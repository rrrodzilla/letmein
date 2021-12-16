use std::net::TcpListener;

#[actix_rt::test]
async fn health_check_test() -> anyhow::Result<()> {
    let service_address = spawn_server().await?;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/pulse", service_address))
        .send()
        .await?;

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());

    Ok(())
}

async fn spawn_server() -> anyhow::Result<String> {
    let listener = TcpListener::bind("127.0.0.1:0")?;

    let service_address = format!("http://127.0.0.1:{}", listener.local_addr()?.port());

    let server = letmein::init_service(listener);
    //let _server = server.await?;
    let _ = tokio::spawn(async { server.await.unwrap().await });

    Ok(service_address)
}
