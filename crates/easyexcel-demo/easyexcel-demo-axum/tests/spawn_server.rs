//! 启动 `easyexcel-demo-axum` 服务进程，用原生 HTTP 请求验证
//! 下载 / 失败降级 / 上传三个端点（对应 Java WebTest 的三个接口）。
//!
//! 端口经 PORT 环境变量配置（对应 Java 可配置 server.port），测试使用随机空闲端口。

use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};

use chrono::NaiveDateTime;
use easyexcel::{EasyExcel, ExcelRow};

/// 上传数据行（与 demo 相同的列序映射）。
#[derive(Debug, Clone, ExcelRow)]
struct UploadData {
    #[excel(index = 0)]
    string: String,
    #[excel(index = 1)]
    date: NaiveDateTime,
    #[excel(index = 2)]
    double_data: f64,
}

const BOUNDARY: &str = "----easyexcel-test-boundary";

/// 获取一个空闲端口（绑定后立即释放）。
fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("bind port 0");
    listener.local_addr().expect("local addr").port()
}

/// 子进程守卫：无论断言成败都确保服务进程被终止。
struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        // 先 SIGTERM 优雅退出（刷新覆盖率 profraw），超时再 SIGKILL
        stop_gracefully(&mut self.0);
    }
}

/// 发送 SIGTERM 等待优雅退出；超时后 SIGKILL 兜底。
fn stop_gracefully(child: &mut Child) {
    // 通过 `kill` 命令发送 SIGTERM（workspace 禁止 unsafe，故不直接调 libc）
    let _ = Command::new("kill")
        .arg("-TERM")
        .arg(child.id().to_string())
        .status();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(_status) = child.try_wait().expect("try_wait") {
            return;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
}

/// 发起一个 HTTP/1.1 请求并返回 (status_line, headers, body)。
///
/// 有界读取：先读到 `\r\n\r\n` 头结束，再按 `Content-Length` 精确读取响应体，
/// 避免 keep-alive 连接上阻塞等待 EOF。
#[allow(clippy::too_many_arguments)]
fn http_request(
    address: &str,
    method: &str,
    target: &str,
    headers: &str,
    body: &[u8],
) -> (String, String, Vec<u8>) {
    let mut stream = TcpStream::connect(address).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("timeout");
    let mut request = format!("{method} {target} HTTP/1.1\r\nHost: {address}\r\n");
    request.push_str(headers);
    request.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
    let mut request = request.into_bytes();
    request.extend_from_slice(body);
    stream.write_all(&request).expect("write request");

    // 读响应头（最多 64KB）：查找 `\r\n\r\n` 终止符（可能在缓冲区中间，
    // 因为头与 body 可能同包到达；固定 Content-Length 响应不以 `\r\n\r\n` 结尾）
    let mut head = Vec::new();
    let mut buf = [0u8; 4096];
    let mut split = None;
    while split.is_none() && head.len() < 65536 {
        let n = stream.read(&mut buf).expect("read head");
        if n == 0 {
            break;
        }
        head.extend_from_slice(&buf[..n]);
        split = head.windows(4).position(|window| window == b"\r\n\r\n");
    }
    let split = split.expect("header terminator");
    let head_text = String::from_utf8_lossy(&head[..split]).into_owned();
    let status = head_text.lines().next().unwrap_or_default().to_owned();

    // 按 Content-Length 读取响应体（有界）
    let content_length = head_text
        .lines()
        .filter_map(|line| {
            let line = line.trim().to_ascii_lowercase();
            line.strip_prefix("content-length:")
                .map(|value| value.trim().parse::<usize>())
        })
        .flatten()
        .next()
        .unwrap_or(0);
    let mut payload = head[split + 4..].to_vec();
    while payload.len() < content_length {
        let n = stream.read(&mut buf).expect("read body");
        if n == 0 {
            break;
        }
        payload.extend_from_slice(&buf[..n]);
    }
    (status, head_text, payload)
}

/// 等待服务端口就绪。
fn wait_ready(child: &mut Child, address: &str) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().expect("try_wait") {
            panic!("demo server exited early: {status:?}");
        }
        if TcpStream::connect(address).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
    panic!("demo server did not become ready within 20s");
}

#[test]
fn demo_axum_serves_download_upload_and_error_endpoints() {
    // 随机空闲端口（对应 Java 可配置 server.port），避免与残留进程冲突
    let address = format!("127.0.0.1:{}", free_port());
    assert!(
        TcpStream::connect(&address).is_err(),
        "port {address} already in use by another process"
    );
    let port = address.rsplit(':').next().expect("port").to_owned();
    let child = Command::new(env!("CARGO_BIN_EXE_easyexcel-demo-axum"))
        .env("PORT", port)
        .spawn()
        .expect("spawn");
    let mut guard = ChildGuard(child);
    wait_ready(&mut guard.0, &address);
    // 服务刚就绪时 hyper 的 accept/连接任务可能仍在预热；等待一个固定窗口
    thread::sleep(Duration::from_millis(500));

    // GET /download → XLSX 附件（Java WebTest.download）
    let (status, head, body) = http_request(&address, "GET", "/download", "", b"");
    assert!(status.starts_with("HTTP/1.1 200"), "{status}");
    assert!(
        head.to_lowercase()
            .contains("content-disposition: attachment;filename*=utf-8''"),
        "{head}"
    );
    assert_eq!(&body[..2], b"PK", "not an xlsx");

    // GET /downloadFailedUsingJson → 成功时仍返回 XLSX（Java downloadFailedUsingJson）
    let (status, _, body) = http_request(&address, "GET", "/downloadFailedUsingJson", "", b"");
    assert!(status.starts_with("HTTP/1.1 200"), "{status}");
    assert_eq!(&body[..2], b"PK", "not an xlsx");

    // POST /upload → multipart 上传 6 行 XLSX：触发监听器 batch>=5 批量刷新 + 返回 success
    let date = NaiveDateTime::parse_from_str("2020-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
        .expect("valid date");
    let rows: Vec<UploadData> = (0..6)
        .map(|index| UploadData {
            string: format!("上传数据{index}"),
            date,
            double_data: f64::from(index) + 0.25,
        })
        .collect();
    let mut buffer = Vec::new();
    EasyExcel::write::<UploadData>("upload.xlsx")
        .sheet("上传")
        .to_writer(&mut buffer)
        .do_write(rows)
        .expect("write xlsx");

    let multipart = format!(
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"upload.xlsx\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    );
    let mut payload = multipart.into_bytes();
    payload.extend_from_slice(&buffer);
    payload.extend_from_slice(format!("\r\n--{BOUNDARY}--\r\n").as_bytes());
    let (status, _, body) = http_request(
        &address,
        "POST",
        "/upload",
        &format!("Content-Type: multipart/form-data; boundary={BOUNDARY}\r\n"),
        &payload,
    );
    assert!(status.starts_with("HTTP/1.1 200"), "{status}");
    assert_eq!(body, b"success", "unexpected upload response");

    // POST /upload → 垃圾字节：读取解析失败走 500（对应 Java 上传解析异常）
    let garbage = format!(
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"bad.xlsx\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    );
    let mut payload = garbage.into_bytes();
    payload.extend_from_slice(b"not-an-excel");
    payload.extend_from_slice(format!("\r\n--{BOUNDARY}--\r\n").as_bytes());
    let (status, _, body) = http_request(
        &address,
        "POST",
        "/upload",
        &format!("Content-Type: multipart/form-data; boundary={BOUNDARY}\r\n"),
        &payload,
    );
    // 对应 Java：上传解析异常 → 500（非 200）
    assert!(status.starts_with("HTTP/1.1 500"), "{status}");
    assert!(
        String::from_utf8_lossy(&body).contains("excel format error"),
        "body: {:?}",
        String::from_utf8_lossy(&body)
    );

    // POST /upload → 畸形 multipart（缺结束边界）：multipart 解析错误 → 400
    let malformed = format!(
        "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"trunc.xlsx\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    );
    let (status, _, _) = http_request(
        &address,
        "POST",
        "/upload",
        &format!("Content-Type: multipart/form-data; boundary={BOUNDARY}\r\n"),
        malformed.as_bytes(),
    );
    assert!(status.starts_with("HTTP/1.1 400"), "{status}");

    // POST /upload → 空 multipart（无字段）：走"未收到上传文件"分支 → 400
    let (status, _, body) = http_request(
        &address,
        "POST",
        "/upload",
        &format!("Content-Type: multipart/form-data; boundary={BOUNDARY}\r\n"),
        format!("--{BOUNDARY}--\r\n").as_bytes(),
    );
    assert!(status.starts_with("HTTP/1.1 400"), "{status}");
    assert!(
        String::from_utf8_lossy(&body).contains("未收到上传文件"),
        "body: {:?}",
        String::from_utf8_lossy(&body)
    );
}
