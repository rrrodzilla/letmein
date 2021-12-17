mod server_config;
pub use server_config::{ServerConfig, ServerConfigError};

use actix_web::{dev::Server, middleware, web, App, HttpResponse, HttpServer};
use std::sync::Once;
use thiserror::Error;
use tracing::{
    info,
    subscriber::{set_global_default, SetGlobalDefaultError},
};
use tracing_actix_web::TracingLogger;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::{log_tracer::SetLoggerError, LogTracer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

#[derive(Debug, Error)]
pub enum LetMeInServerError {
    #[error(transparent)]
    ServerConfigError(#[from] ServerConfigError),
    #[error(transparent)]
    InitiateServerError(#[from] std::io::Error),
    #[error(transparent)]
    FromLoggerError(#[from] SetGlobalDefaultError),
    #[error(transparent)]
    SetLoggerError(#[from] SetLoggerError),
}

#[tracing::instrument(skip(cfg))]
fn config_web_app(cfg: &mut web::ServiceConfig) {
    //build all the routes
    cfg.service(web::resource("/pulse").route(web::get().to(HttpResponse::Ok)));
}

static INIT: Once = Once::new();

pub fn init_service(settings: ServerConfig) -> Result<Server, LetMeInServerError> {
    //initialize logging
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(settings.log_level()));
    let formatting_layer = BunyanFormattingLayer::new("letmein".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    // can only call init once so make sure that's the case, especially when we're creating
    // multiple instances in different threads via integration tests (or any other process)
    INIT.call_once(|| {
        LogTracer::init().unwrap();
        set_global_default(subscriber).unwrap();
    });

    let service_address = format!("{}:{}", settings.host(), settings.port());
    //set up and return server
    let server = HttpServer::new(|| {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(TracingLogger::default())
            .configure(config_web_app)
    })
    .listen(settings.tcp_listener)?
    .run();

    info!("Listening at: http://{}", service_address);
    Ok(server)
}

#[cfg(test)]
mod unit_tests {
    use super::{config_web_app, ServerConfig};
    use actix_web::{test, App};

    #[actix_rt::test]
    async fn test_heartbeat_response() {
        let mut app = test::init_service(App::new().configure(config_web_app)).await;
        let req = test::TestRequest::get().uri("/pulse").to_request();
        let resp = test::call_service(&mut app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_heartbeat_response_length() {
        let mut app = test::init_service(App::new().configure(config_web_app)).await;
        let req = test::TestRequest::get().uri("/pulse").to_request();
        let bytes = test::read_response(&mut app, req).await;

        assert_eq!(0, bytes.len());
    }

    #[actix_rt::test]
    async fn test_config() -> anyhow::Result<()> {
        let settings = ServerConfig::load(&mut config::Config::default())?;
        assert_eq!(settings.host(), "127.0.0.1");

        Ok(())
    }
}
