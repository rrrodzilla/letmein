use std::time::Duration;

use letmein_server::ServerConfig;

#[actix_rt::test]
async fn health_check_test() -> anyhow::Result<()> {
    let service_address = spawn_server()?;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/pulse", service_address))
        .timeout(Duration::from_secs(2))
        .send()
        .await?;

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());

    Ok(())
}

#[actix_rt::test]
async fn not_found_test() -> anyhow::Result<()> {
    let service_address = spawn_server()?;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}", service_address))
        .timeout(Duration::from_secs(2))
        .send()
        .await?;

    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
    assert_eq!(Some(0), response.content_length());

    Ok(())
}

fn spawn_server() -> anyhow::Result<String> {
    let settings = ServerConfig::load(&mut config::Config::default())?;
    let service_address = format!("http://{}:{}", settings.host(), settings.port());

    let _ = tokio::spawn(letmein_server::init_service(settings)?);

    Ok(service_address)
}
