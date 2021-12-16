use actix_web::{dev::Server, middleware::Logger, web, App, HttpResponse, HttpServer};
use env_logger::Env;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LetMeInServerError {
    #[error(transparent)]
    InitiateServerError(#[from] std::io::Error),
}

fn config_app(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/pulse").route(web::get().to(HttpResponse::Ok)));
}

#[actix_web::main]
pub async fn init_service() -> Result<Server, LetMeInServerError> {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    let server = HttpServer::new(|| App::new().wrap(Logger::default()).configure(config_app))
        .bind("127.0.0.1:8000")?
        .run();
    Ok(server)
}

#[cfg(test)]
mod unit_tests {
    use super::config_app;
    use actix_web::{test, App};

    #[actix_rt::test]
    async fn test_heartbeat_response() {
        let mut app = test::init_service(App::new().configure(config_app)).await;
        let req = test::TestRequest::get().uri("/pulse").to_request();
        let resp = test::call_service(&mut app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_heartbeat_response_length() {
        let mut app = test::init_service(App::new().configure(config_app)).await;
        let req = test::TestRequest::get().uri("/pulse").to_request();
        let bytes = test::read_response(&mut app, req).await;

        assert_eq!(0, bytes.len());
    }
}
