use letmein_server::ServerConfig;

#[actix_web::main]
pub async fn main() -> Result<(), letmein_server::LetMeInServerError> {
    let settings = ServerConfig::load(&mut config::Config::default())?;

    letmein_server::init_service(settings)?.await?;

    Ok(())
}
