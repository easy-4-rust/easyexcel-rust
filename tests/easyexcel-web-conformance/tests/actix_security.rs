//! Security regressions for the pinned Actix HTTP decoder.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use actix_http::encoding::Decoder;
use actix_http::error::PayloadError;
use actix_http::header::ContentEncoding;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use bytes::Bytes;
use futures_util::{StreamExt, stream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

#[actix_web::test]
async fn brotli_large_window_is_rejected_before_unbounded_allocation() {
    let input = vec![b'a'; 4096];
    let params = brotli::enc::BrotliEncoderParams {
        large_window: true,
        lgwin: 25,
        ..Default::default()
    };
    let mut compressed = Vec::new();
    brotli::BrotliCompress(&mut &input[..], &mut compressed, &params)
        .expect("encode large-window Brotli fixture");
    assert_eq!(
        compressed[0], 0x11,
        "fixture must use the large-window marker"
    );

    let body = stream::iter([Ok::<Bytes, PayloadError>(Bytes::from(compressed))]);
    let mut decoder = Decoder::new(body, ContentEncoding::Brotli);

    assert!(
        matches!(
            decoder.next().await,
            Some(Err(PayloadError::Incomplete(Some(_))))
        ),
        "large-window Brotli must be rejected"
    );
    assert!(
        decoder.next().await.is_none(),
        "decoder must stop after rejection"
    );
}

#[actix_web::test]
async fn compressed_output_limit_is_enforced_by_decoder() {
    let input = vec![b'a'; 64 * 1024];
    let params = brotli::enc::BrotliEncoderParams::default();
    let mut compressed = Vec::new();
    brotli::BrotliCompress(&mut &input[..], &mut compressed, &params)
        .expect("encode compressed-size-limit fixture");

    let body = stream::iter([Ok::<Bytes, PayloadError>(Bytes::from(compressed))]);
    let mut decoder = Decoder::new(body, ContentEncoding::Brotli).limit(1024);

    assert!(
        matches!(decoder.next().await, Some(Err(PayloadError::Overflow))),
        "decoded output above the configured limit must fail"
    );
    assert!(
        decoder.next().await.is_none(),
        "decoder must stop after overflow"
    );
}

async fn record_path(
    request: HttpRequest,
    _body: web::Bytes,
    seen: web::Data<Arc<Mutex<Vec<String>>>>,
) -> HttpResponse {
    seen.lock()
        .expect("request path registry mutex")
        .push(request.path().to_owned());
    HttpResponse::Ok().finish()
}

#[actix_web::test]
async fn malformed_chunked_trailer_cannot_smuggle_a_pipelined_request() {
    let seen = Arc::new(Mutex::new(Vec::<String>::new()));
    let server = HttpServer::new({
        let seen = seen.clone();
        move || {
            App::new()
                .app_data(web::Data::new(seen.clone()))
                .default_service(web::route().to(record_path))
        }
    })
    .bind(("127.0.0.1", 0))
    .expect("bind local Actix test server");

    let address = server.addrs()[0];
    let server = server.run();
    let handle = server.handle();
    let server_task = actix_web::rt::spawn(server);

    let mut stream = TcpStream::connect(address)
        .await
        .expect("connect to local Actix server");

    let smuggle_attempt = b"GET /smuggled-1 HTTP/1.1\r\nHost: localhost\r\n\r\n\
                            GET /smuggled-2 HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let mut request = b"POST /chunked HTTP/1.1\r\n\
                        Host: localhost\r\n\
                        Transfer-Encoding: chunked\r\n\r\n\
                        5\r\nhello\r\n0\r\n\n"
        .to_vec();
    request.extend_from_slice(
        format!(
            "POST /front HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n",
            smuggle_attempt.len()
        )
        .as_bytes(),
    );
    request.extend_from_slice(smuggle_attempt);
    request
        .extend_from_slice(b"GET /third HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");

    stream
        .write_all(&request)
        .await
        .expect("write malformed chunked request sequence");
    let mut response = Vec::new();
    let read_result = timeout(Duration::from_secs(5), stream.read_to_end(&mut response)).await;

    handle.stop(true).await;
    let _ = server_task.await;

    assert!(
        read_result.is_ok(),
        "server did not close/respond within the bounded test window"
    );
    let dispatched = seen.lock().expect("request path registry mutex").clone();
    assert!(
        !dispatched.iter().any(|path| path.starts_with("/smuggled")),
        "malformed chunked input dispatched a smuggled request: {dispatched:?}; response: {}",
        String::from_utf8_lossy(&response)
    );
}
