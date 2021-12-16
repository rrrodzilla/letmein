use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpResponse, HttpServer};
use thiserror::Error;
use tracing::subscriber::{set_global_default, SetGlobalDefaultError};
use tracing_actix_web::TracingLogger;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::{log_tracer::SetLoggerError, LogTracer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

#[derive(Debug, Error)]
pub enum LetMeInServerError {
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

//#[actix_web::main]
pub async fn init_service(tcp_listener: TcpListener) -> Result<Server, LetMeInServerError> {
    //initialize logging
    LogTracer::init()?;
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("letmein".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber)?;
    //  let span = tracing::debug_span!("init_service");
    //  let _enter = span.enter();

    //set up and return server
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .configure(config_web_app)
    })
    .listen(tcp_listener)?
    .run();

    Ok(server)
}

#[cfg(test)]
mod unit_tests {
    use super::config_web_app;
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
}
