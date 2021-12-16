#[actix_rt::test]
async fn health_check_test() -> anyhow::Result<()> {
    letmein::init_service()?.await?;

    let client = reqwest::Client::new();

    let response = client.get("http://127.0.0.1:8000/pulse").send().await?;

    assert!(response.status().is_success());

    Ok(())
}
