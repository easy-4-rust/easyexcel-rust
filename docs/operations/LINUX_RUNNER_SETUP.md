# Linux Benchmark Runner -- 运维配置指南

本文档指导运维人员配置自托管 GitHub Actions runner，用于运行 release profile
benchmark（1M rows, cross_runtime gate）。所有 release baseline 必须在此固定
runner 上生成；本地 macOS 运行 **不** 被接受为 release 证据。

---

## 1. 机型规格

| 属性 | 要求 | 原因 |
|------|------|------|
| 架构 | 裸金属 x86_64（非共享 VM） | 消除 noisy-neighbour 干扰 |
| CPU | >= 8 物理核，禁用 turbo boost | 稳定频率，减少方差 |
| 内存 | >= 32 GB | Java 堆 4g + Rust + OS 缓存 |
| 磁盘 | NVMe SSD, `/tmp` 分区 >= 50 GB 可用 | benchmark 临时文件 |
| 操作系统 | Ubuntu 22.04 LTS 或 24.04 LTS x86_64 | 长期支持，稳定内核 |
| 内核 | 发行版原生内核，无自定义 `isolcpus`/`cgroup` | 与 CI 环境一致 |

### CPU 频率调控

benchmark 期间 CPU governor 必须为 `performance`，禁用频率缩放：

```bash
# 查看当前 governor
cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor

# 设置为 performance（所有核心）
for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
  echo performance | sudo tee "$cpu"
done

# 持久化：安装 cpufrequtils
sudo apt-get install -y cpufrequtils
echo 'GOVERNOR="performance"' | sudo tee /etc/default/cpufrequtils
sudo systemctl restart cpufrequtils
```

### 后台服务

benchmark 运行期间（release profile 约 50-60 分钟）禁止以下服务消耗显著
CPU 或 I/O：

- 系统更新 (`unattended-upgrades`)
- 日志轮转
- 其他 CI runner 任务
- 数据库/缓存服务

建议在 cron 中于 benchmark 窗口前暂停这些服务。

---

## 2. 基础软件安装

### 2.1 系统依赖

```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  curl \
  git \
  python3 \
  python3-pip \
  unzip \
  locales \
  tzdata

# 设置 locale
sudo locale-gen en_US.UTF-8
sudo update-locale LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8

# 设置时区
sudo timedatectl set-timezone UTC
```

### 2.2 Rust 1.97.1（pinned）

```bash
# 安装 rustup（如尚未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/bin/env"

# 安装 pinned 版本
rustup toolchain install 1.97.1
rustup default 1.97.1

# 验证
rustc --version   # 应输出 rustc 1.97.1 (xxxx-xx-xx)
cargo --version
```

**重要**：不要运行 `rustup update`。benchmark attestation 记录了 `rustc` 二进制的
SHA-256 指纹，toolchain 变更会导致 attestation 校验失败。

### 2.3 Temurin Java 17

```bash
# 使用 Adoptium APT 仓库
sudo apt-get install -y wget apt-transport-https gpg
wget -qO - https://packages.adoptium.net/artifactory/api/gpg/key/public | \
  sudo gpg --dearmor -o /usr/share/keyrings/adoptium.gpg
echo "deb [signed-by=/usr/share/keyrings/adoptium.gpg] \
  https://packages.adoptium.net/artifactory/deb $(lsb_release -cs) main" | \
  sudo tee /etc/apt/sources.list.d/adoptium.list
sudo apt-get update
sudo apt-get install -y temurin-17-jdk

# 设置 JAVA_HOME
export JAVA_HOME="/usr/lib/jvm/temurin-17-jdk-amd64"
echo "export JAVA_HOME=\"$JAVA_HOME\"" >> "$HOME/.bashrc"

# 验证
java -version   # 应输出 openjdk version "17.x.x"
```

### 2.4 Java easyexcel v4.0.3 源码

```bash
# 克隆到固定路径（runner 使用）
git clone --branch v4.0.3 --depth 1 \
  https://github.com/easy-4-java/easyexcel.git \
  /opt/easyexcel-java

# 验证
cd /opt/easyexcel-java && git status && git log --oneline -1
# 应显示 tag v4.0.3 对应的 commit
```

### 2.5 Maven

bundled `mvnw` wrapper 足够，无需单独安装 Maven。如需系统级 Maven：

```bash
sudo apt-get install -y maven
mvn --version
```

### 2.6 Python 3.10+

Ubuntu 22.04/24.04 自带 Python 3.10+。验证：

```bash
python3 --version   # >= 3.10
```

---

## 3. GitHub Actions Self-Hosted Runner 注册

### 3.1 创建 runner

1. 进入 GitHub 仓库 Settings > Actions > Runners
2. 点击 "New self-hosted runner"
3. 选择 Linux x86_64
4. 按照页面指示下载并配置 runner

```bash
# 创建 runner 目录
mkdir -p /opt/actions-runner && cd /opt/actions-runner

# 下载 runner 包（版本号以 GitHub 页面为准）
curl -o actions-runner-linux-x64.tar.gz -L \
  https://github.com/actions/runner/releases/download/v2.319.1/actions-runner-linux-x64-2.319.1.tar.gz
tar xzf actions-runner-linux-x64.tar.gz

# 配置（使用仓库提供的 token）
./config.sh \
  --url https://github.com/<OWNER>/easyexcel-rust \
  --token <REGISTRATION_TOKEN> \
  --labels linux-benchmark \
  --name "easyexcel-benchmark-linux-01" \
  --work /tmp/actions-runner-work
```

**关键**：`--labels linux-benchmark` 是 workflow 文件中 `runs-on` 匹配的标签。

### 3.2 以 systemd 服务运行

```bash
sudo ./svc.sh install
sudo ./svc.sh start
sudo ./svc.sh status
```

### 3.3 Runner 标签验证

在 GitHub 仓库 Settings > Actions > Runners 页面确认 runner 状态为
"Online"，标签包含 `linux-benchmark`。

---

## 4. 触发 Release Benchmark Workflow

### 4.1 手动触发（推荐首次使用）

1. 进入 GitHub 仓库 Actions 页
2. 选择 "Linux Release Baseline Benchmark" workflow
3. 点击 "Run workflow"
4. 参数：
   - `generate_baseline`: 勾选（默认 true）
   - `java_repo_ref`: `v4.0.3`（默认）
   - `reviewer`: 填写操作者姓名
5. 点击绿色 "Run workflow" 按钮

### 4.2 自动触发（GitHub Release）

创建并发布 GitHub Release 时自动触发：

```bash
# 通过 CLI 创建 release
gh release create v0.1.0 --title "v0.1.0" --notes "First release"
```

workflow 将在 self-hosted runner 上自动运行 release profile benchmark。

### 4.3 监控运行

1. 进入 Actions 页，点击正在运行的 workflow
2. 展开各 step 查看实时日志
3. release profile 预计运行 50-60 分钟（含 soak）
4. 运行完成后下载 artifact `release-benchmark-linux-<run_id>`

---

## 5. 解读 Baseline Candidate

### 5.1 Artifact 结构

```
release-benchmark-linux-<run_id>/
  artifacts/
    release-runner-artifact.json    # attestation manifest
    release/
      matrix/
        raw-results.jsonl           # benchmark 原始数据
        environment-manifest.json   # runner 环境快照
      soak/
        raw-results.jsonl           # soak 原始数据
        soak-manifest.json
      report.json                   # 汇总报告（passed/failures）
  benchmarks/baselines/
    release-ubuntu-x64.json         # baseline candidate（如生成）
```

### 5.2 检查报告

```bash
# 下载 artifact 后解压
unzip release-benchmark-linux-*.zip

# 查看报告
python3 -c "
import json
r = json.load(open('artifacts/release/report.json'))
print(f'Passed:  {r[\"passed\"]}')
print(f'Samples: {r[\"valid_sample_count\"]}/{r[\"sample_count\"]}')
for f in r.get('failures', []):
    print(f'  FAIL: {f}')
"
```

### 5.3 cross_runtime 门禁

报告中的 `cross_runtime_ratios` 字段包含 Rust vs Java 吞吐比：

- **低并发** (workers 1, 2, 4): `median_ratio >= 1.00` 且 `confidence_lower_bound >= 0.95`
- **高并发** (workers 8, 16): `median_ratio >= 0.90`

任何一项未达标，`passed` 将为 `false`。

---

## 6. Approve Baseline 流程

baseline candidate 通过所有门禁后，需要人工 review 并 approve。

### 6.1 自动方式（workflow 内置）

如果 workflow 运行时 `generate_baseline=true`，baseline candidate 会自动生成
并作为 artifact 上传。运维只需：

1. 下载 `release-baseline-candidate-<run_id>` artifact
2. 验证 `release-ubuntu-x64.json` 内容
3. 将文件复制到仓库 `benchmarks/baselines/release-ubuntu-x64.json`
4. 提交 PR

### 6.2 手动方式（更精细控制）

```bash
# 在 runner 上或本地（需要相同的原始数据）
python3 benchmarks/scripts/approve_benchmark_baseline.py \
  --candidate-report artifacts/release/report.json \
  --spec benchmarks/spec/benchmark-suite-v1.json \
  --result artifacts/release/matrix/raw-results.jsonl \
  --result artifacts/release/soak/raw-results.jsonl \
  --soak-manifest artifacts/release/soak/soak-manifest.json \
  --reviewer "your-name" \
  --review-notes "Release baseline on fixed Linux runner, YYYY-MM-DD" \
  --output benchmarks/baselines/release-ubuntu-x64.json
```

### 6.3 Approve 脚本校验项

`approve_benchmark_baseline.py` 自动验证：

- candidate report 必须通过所有非回归门禁（`passed=true`, `failures=[]`）
- spec SHA 匹配
- Java/Rust 源码 Git SHA 存在且有效
- 输出路径在 `benchmarks/baselines/` 下且命名为 `{profile}-ubuntu-x64.json`
- 输出文件不存在（如存在需先删除 stub）

### 6.4 提交 Baseline

```bash
cd /path/to/easyexcel-rust
git add benchmarks/baselines/release-ubuntu-x64.json
git commit -m "baseline: add reviewed release baseline from Linux runner

Runner: easyexcel-benchmark-linux-01
Profile: release (1M rows, 3 warmups, 7 measurements)
cross_runtime gate: passed
Reviewer: <name>"
git push
```

---

## 7. 运维常见问题

### Q: workflow 运行失败，提示 locale 不是 UTF-8

在 runner 上执行：
```bash
sudo locale-gen en_US.UTF-8
export LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8
```

### Q: attestation 校验失败（Rust toolchain mismatch）

不要运行 `rustup update`。重新 pin 到 1.97.1：
```bash
rustup default 1.97.1
```

### Q: cross_runtime gate 失败，Rust/Java ratio < 1.00

1. 检查 runner 是否为裸金属（非 VM）
2. 检查 CPU governor 是否为 `performance`
3. 检查是否有后台进程干扰
4. 检查环境温度（thermal throttling）

### Q: 如何替换已有的 release baseline？

`approve_benchmark_baseline.py` 拒绝覆盖已有文件。需要：

1. 手动删除旧的 `release-ubuntu-x64.json`
2. 重新运行 approve 脚本
3. 在 PR 中说明替换原因

### Q: runner 磁盘空间不足

benchmark release profile 生成大量临时文件。确保 `/tmp` 有 >= 50 GB 可用：
```bash
df -h /tmp
```

---

## 8. 参考文档

| 文档 | 路径 |
|------|------|
| Baseline schema | `benchmarks/baselines/baseline.schema.json` |
| Baseline runbook | `benchmarks/baselines/README.md` |
| cross_runtime gate runbook | `docs/performance/CROSS_RUNTIME_RUNBOOK.md` |
| Benchmark spec | `benchmarks/spec/benchmark-suite-v1.json` |
| Approve script | `benchmarks/scripts/approve_benchmark_baseline.py` |
| Compare script | `benchmarks/scripts/compare_results.py` |
| Release workflow | `.github/workflows/release-benchmark.yml` |
