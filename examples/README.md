# Web framework examples

七个 Web 示例都使用 `easyexcel-web` 的统一运行时，并由各框架适配 crate 暴露原生
`ExcelRequest<T>` 与 `ExcelResponse<T>`：

| Framework | Package | Default port |
|---|---|---:|
| Axum | `easyexcel-demo-axum` | 8080 |
| Actix Web | `easyexcel-demo-actix` | 8081 |
| Hyper | `easyexcel-demo-hyper` | 8082 |
| Poem | `easyexcel-demo-poem` | 8083 |
| Salvo | `easyexcel-demo-salvo` | 8084 |
| Warp | `easyexcel-demo-warp` | 8085 |
| Rocket | `easyexcel-demo-rocket` | 8000 |

每个示例提供 `GET /download` 和 `POST /upload`。上传采用原始 CSV、XLS 或 XLSX 请求体：

```bash
cargo run -p easyexcel-demo-axum
curl -X POST http://127.0.0.1:8080/upload \
  -H 'Content-Type: text/csv' \
  -H 'x-excel-file-name: rows.csv' \
  --data-binary @rows.csv
curl -OJ http://127.0.0.1:8080/download
```

同一传输和行为契约由 `tests/easyexcel-web-conformance` 在七个框架上复用，避免示例与
生产适配器产生不同的资源限制、错误协议或流式语义。
