## 快速目标（给 AI 代理）

这是 ping-rs 仓库的专用工作指南，帮助你快速在本项目中完成改动、编写测试和打包本地 wheel。

要点：

- 项目是一个用 Rust 编写并通过 PyO3 / maturin 暴露给 Python 的高性能 ping 库（混合 Rust + Python）。
- 注意平台差异：Linux/macOS 使用系统 `ping` 命令（输出需解析）；Windows 使用 `winping` 原生 ICMP。

## 常用命令（在仓库根目录运行）

项目使用 `uv` 作为 Python 包管理器和 `Makefile` 作为任务运行器。**优先使用 make 目标**而非直接命令。

### 环境设置

- **完整安装**（首次设置）：`make install`
  自动安装 uv、pre-commit、同步依赖、安装开发版本
- **更新依赖**：`make rebuild-lockfiles`

### 构建变体

- **开发构建**：`make build-dev` 或 `uv run maturin develop --uv`
  快速编译，带调试符号
- **生产构建**：`make build-prod`
  优化构建（--release）
- **性能分析构建**：`make build-profiling`
  带性能分析支持
- **覆盖率构建**：`make build-coverage`
  插桩用于代码覆盖率
- **PGO 构建**：`make build-pgo`
  Profile-Guided Optimization（两阶段构建+测试）

### 测试与验证

- **运行测试**：`make test` 或 `uv run pytest -n logical`
  并行测试（logical core 数）
- **测试覆盖率**：`make testcov`
  生成 Python + Rust 混合覆盖率报告至 `htmlcov/`
- **格式化代码**：`make format`
  自动修复 Python (ruff) + Rust (cargo fmt)
- **Lint 检查**：`make lint`（或分别 `make lint-python` / `make lint-rust`）
  - Python: ruff check/format + griffe + mypy stubtest
  - Rust: cargo fmt check + cargo clippy
- **完整 CI 流程**：`make all`
  = format + build-dev + lint + test

### 运行示例

- 查看 `python/examples/` 下的脚本，使用 `uv run python python/examples/basic_usage.py` 运行

### 清理

- **清理构建产物**：`make clean`
  移除 `__pycache__`、`*.so`、`htmlcov`、`.pytest_cache` 等

## 项目架构 / 关键路径

- **Rust 源码**：`src/`（核心逻辑、协议实现、类型与结果处理）
  - `src/lib.rs`：crate 根，PyO3 绑定入口
  - `src/protocols/icmp/`：ping 协议/平台相关实现
  - `src/types/result.rs`：PingResult 等类型定义与序列化边界
  - `src/utils/conversion.rs`、`src/utils/validation.rs`：辅助工具
- **Python 包装层**：`python/ping_rs/`（公开给 Python 的 API）
  - `python/ping_rs/__init__.py`：Python 可见接口（`ping_once`, `ping_multiple`, `create_ping_stream` 等）
  - `python/_ping_rs.pyi`：类型存根（供 IDE/mypy 使用）
  - `python/ping_rs/core_schema.py`：核心数据模型（如果存在）
  - `python/examples/basic_usage.py`：快速示例与集成测试参考
- **构建配置**：
  - `pyproject.toml`：Python 项目元数据 + maturin 配置
  - `Cargo.toml`：Rust 依赖与 crate 配置
  - `build.rs`：Rust 构建脚本
  - `Makefile`：开发任务运行器（**首选入口**）
- **测试套件**：`tests/`
  - `test_basic.py`, `test_async.py`, `test_timeout.py`, `test_concurrent.py`
  - `run_all_tests.py`：传统测试入口（现推荐 `make test`）
  - `conftest.py`：pytest fixtures 与配置
- **发行物**：`target/wheels/` 存放预构建 wheel

## 开发约定与可观察模式（只记录仓库已显式使用的模式）

- 同步与异步 API 并存：Python 层提供 `ping_once` / `ping_multiple`（同步）与 `ping_once_async` / `ping_multiple_async`（异步）以及 stream 型 API（`create_ping_stream` / `AsyncPingStream`）。修改时同时考虑两套 API 的一致性。
- 错误与结果类型：使用 `PingResult` 的多种子类型（Pong、Timeout、Unknown、PingExited）。在 Python 层通过 `.is_success()`, `.is_timeout()`, `.to_dict()` 等方法判断与转换；修改结果格式时，同时更新 Python 类型提示文件 `python/_ping_rs.pyi` 和 `python/ping_rs/core_schema.py`（如果存在）。
- 平台检测：运行时可能抛出 `RuntimeError: Failed to start ping: Could not detect ping.`，这是因为在 Linux/macOS 未找到系统 `ping` 工具。测试或 CI 中请确保目标环境可访问 `ping`，或在单元测试中模拟/隔离系统调用。

## 编辑/测试/提交建议

- **标准开发流程**（小变更，如修复解析 bug）：

  1. 在 `src/` 或 `python/` 做改动并增加针对性的单元测试（`tests/` 下已有模拟与集成测试）
  2. 运行 `make build-dev` 构建开发版本
  3. 运行 `make test` 验证功能
  4. 若改动影响 API（函数签名、返回 dict 字段等），同时更新 `python/_ping_rs.pyi` 与 README 示例
  5. 提交前运行 `make format` + `make lint` 确保代码风格一致

- **完整 CI 验证**：`make all`
  按顺序执行：format → build-dev → lint → test（模拟 CI 流程）

- **调试技巧**：

  - Rust 层：`cargo test` 或 `cargo clippy`
  - Python 层：`make build-dev` 后用 Python REPL/脚本调用触发边界代码
  - 覆盖率分析：`make testcov` 生成 HTML 报告至 `htmlcov/`

- **性能优化**：
  - 基准测试：`make build-profiling` + 性能分析工具
  - PGO 优化：`make build-pgo`（两阶段构建，测试驱动优化）

## 注意的代码位置（便于快速跳转）

- **Python API 定义和类型提示**：`python/ping_rs/__init__.py`, `python/_ping_rs.pyi`, `python/ping_rs/core_schema.py`
- **测试套件**：`tests/run_all_tests.py`（传统入口），各测试文件：`test_basic.py`, `test_async.py`, `test_timeout.py`, `test_concurrent.py`
- **Rust 协议实现**：`src/protocols/icmp/`（平台相关代码）
- **结果解析与转换**：`src/types/result.rs`, `src/utils/conversion.rs`
- **构建脚本**：`build.rs`（Rust 编译前处理），`Makefile`（开发任务）

## 集成与外部依赖

- **Rust 依赖**：项目使用 `pinger`（crates.io）及在 Windows 上的 `winping`。Python 端通过 PyO3 / maturin 打包。
- **Python 工具链**：使用 `uv` 包管理器、`ruff` 格式化/linting、`pytest` 测试框架、`mypy` 类型检查
- **环境要求**：
  - Python 3.7+，pytest，pytest-asyncio
  - 系统需有 `ping`（Linux/macOS）或在 Windows 下依赖 `winping` crate
  - 开发环境推荐安装 pre-commit hooks（`make install` 自动安装）

## 例子（可直接用于代码生成或补全提示）

- **同步单次 ping**（用于参考测试/示例）：
  ```python
  from ping_rs import ping_once
  result = ping_once("google.com")
  if result.is_success():
      print(f"Latency: {result.duration_ms} ms")
  ```
- **异步流式处理**：参考 `python/examples/basic_usage.py` 和 README 中的 `AsyncPingStream` 示例
- **开发时快速验证**：
  ```bash
  make build-dev && uv run python -c "from ping_rs import ping_once; print(ping_once('8.8.8.8').to_dict())"
  ```
