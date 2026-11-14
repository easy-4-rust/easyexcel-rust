# Proposal: Dependency Security Remediation

## Why

PR #5 已将 ZIP 依赖从被 yanked 的 8.6.0 迁移到 ZIP 9.1.3，但 2026-11-08 的 Security Audit 仍报告 9 项漏洞和 11 项 unsoundness 警告。当前依赖链覆盖 HTTP 请求解析/解压、OLE2 文件解析、HTTP/2 frame 管理、Zstandard 流式内存安全和格式缓存。

继续让多个 Agent 各自升级依赖会造成重复版本、MSRV 漂移、框架 feature 被误删和 Cargo.lock 互相覆盖。必须先建立单一审计事实来源与共享设计决策，再逐项修复。

## Scope

- 修复 docs/audit/2026-11-08-dependency-security-audit.md 列出的全部 9 项漏洞和 11 项 unsoundness。
- 保留 ZIP 9.1.3 迁移和所有现有功能边界。
- 保留 Actix、Salvo、Rocket 集成；保留 HTTP/2、Brotli/gzip/deflate/zstd 能力。
- 经后续决策 D7，将第一方 workspace/core/adapter/example/test 的 MSRV 统一提升到 Rust 1.95，以直接使用 ref-swap 0.1.3 上游安全修复而无需维护本地 fork。
- 建立恶意输入、并发/资源耗尽和格式兼容回归测试。
- 保持 cargo audit / cargo deny 无豁免策略。

## Out of Scope

- 不重构 Web adapter 的公共 API。
- 不移除框架或压缩/HTTP2 feature。
- 不进行与本审计无关的全 workspace 依赖清理。
- 不通过 ignore、allow、漏洞数据库配置或缩小 CI 审计范围来消除告警。
- 不在未评估 API/MSRV 的情况下升级框架主版本。

## Success Criteria

1. 所有审计 ID 均关闭，有明确源码/依赖链和验证证据。
2. cargo audit --no-fetch --deny warnings 与 cargo deny check advisories bans licenses sources 均通过。
3. 全部 workspace、适配器、Java golden、XLSX round-trip 和安全回归通过。
4. Cargo.lock 不包含报告中的受影响版本。
5. PR #5 从 Draft 转为 Ready 并通过完整 CI 后才允许合并。
