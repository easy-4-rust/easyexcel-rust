//! End-to-end HTTP/2 bridge regression using h2 0.4.
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use tokio::net::TcpStream;

#[actix_web::test]
async fn actix_http2_uses_h2_04_http_type_bridge() {
    let http_server = HttpServer::new(|| {
        App::new().route(
            "/h2-bridge",
            web::get().to(|request: HttpRequest| async move {
                let probe = request
                    .headers()
                    .get("x-h2-probe")
                    .expect("probe header")
                    .to_str()
                    .expect("valid probe header")
                    .to_owned();

                HttpResponse::Ok()
                    .insert_header(("x-h2-method", request.method().as_str()))
                    .body(format!("{probe}:ok"))
            }),
        )
    })
    .bind(("127.0.0.1", 0))
    .expect("bind h2c test server")
    .auto_h2c(true);

    let address = http_server.addrs()[0];
    let server = http_server.run();
    let handle = server.handle();
    let server_task = actix_web::rt::spawn(server);

    let socket = TcpStream::connect(address)
        .await
        .expect("connect h2c client");
    let (mut client, connection) = h2::client::handshake(socket)
        .await
        .expect("h2 0.4 client handshake");
    let connection_task = tokio::spawn(async move {
        connection.await.expect("h2 connection");
    });

    let request = http::Request::builder()
        .method(http::Method::GET)
        .uri(format!("http://{address}/h2-bridge"))
        .header("x-h2-probe", "h2-04")
        .body(())
        .expect("build h2 request");

    let (response, _send_stream) = client.send_request(request, true).expect("send h2 request");
    let response = response.await.expect("receive h2 response");

    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(response.version(), http::Version::HTTP_2);
    assert_eq!(response.headers().get("x-h2-method").unwrap(), "GET");

    let mut body = response.into_body();
    let mut bytes = Vec::new();
    while let Some(chunk) = body.data().await {
        let chunk = chunk.expect("read h2 response body");
        let len = chunk.len();
        bytes.extend_from_slice(&chunk);
        body.flow_control()
            .release_capacity(len)
            .expect("release h2 receive capacity");
    }
    assert_eq!(bytes, b"h2-04:ok");

    handle.stop(true).await;
    let _ = server_task.await;
    let _ = connection_task.await;
}
