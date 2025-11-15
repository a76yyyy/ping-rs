# 更新日志

简体中文 | [English](./CHANGELOG.md)

本文档记录了项目的所有重要变更。

文档格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
项目遵循[语义化版本](https://semver.org/lang/zh-CN/)规范。

## [未发布]

### Added

- 增强异步 ping 错误处理和性能测试的全面测试覆盖
- 新增压力测试、内存使用、背压和性能基准测试
- 启用 pytest 并行执行测试，使用 `-n logical` 选项
- 新增全面的异步 ping 测试覆盖
- 新增跨平台架构说明
- 完善 README 中的平台支持详情

### Changed

- 统一所有 ping 操作的超时控制机制
- 优化异步 ping 接收逻辑，移除多余的阻塞操作
- 重构 ICMP ping 超时计算逻辑和结果判断
- 将异步 ping 结果通道从阻塞标准通道迁移到 Tokio 异步通道
- 简化平台适配逻辑，统一使用 `pinger` 库处理所有平台
- 调整测试断言，提高稳健性
- 升级 `pyo3` 及相关库至 0.27 版本
- 升级 `pyo3-log` 至 0.13 版本
- 升级 `pinger` 至 2.1.1 版本
- 升级 pre-commit-hooks 版本
- 调整 Makefile 中 pre-commit 启动命令
- 优化 pyproject.toml 依赖配置和分类信息

### Fixed

- 调整异步多次 ping 测试的超时参数和结果断言，避免测试偶发失败
- 修正 Windows 平台 ping 超时包数计算逻辑
- 修正异步 ping 超时处理 - 超时后先尝试接收剩余结果再停止
- 修正 ping 超时逻辑及跨平台超时计算
- 调整 ping 默认超时时间为 1000 毫秒

### Removed

- 移除 Windows 平台专用 ping 实现及相关工具代码
- 移除 IP 解析和定时相关的 Windows 工具代码

## [1.1.0] - 2025-06-08

### Added

- **AsyncPinger 类**：用于执行异步 ping 操作
- **AsyncPingStream**：支持原生 async/await 的异步迭代
- 为 `create_ping_stream` 函数添加 `count` 参数
- 新增 count 和 timeout 参数的验证函数
- 新增 PingStream 和 AsyncPingStream 的使用示例
- 新增基础使用示例，演示同步和异步 ping 操作
- 新增 PingStream 作为迭代器的全面使用示例
- 新增 AsyncPingStream 异步迭代示例
- 增强异步 ping 操作文档

### Changed

- 改进异步 ping 功能并优化接口
- 重构 PingStream 中的非阻塞接收逻辑
- 重构 sync_stream_example，移除异步元素
- 更新 CI 配置，使用 `pytest-xdist` 并行运行测试
- Windows 平台 IP 函数条件编译优化
- 重构 Windows ping 实现，提高代码清晰度和可维护性
- 移除 pytest-xdist 配置中的 `psutil` 依赖
- 版本更新至 1.1.0

### Fixed

- 将 AsyncPinger 添加到 `__all__` 导出列表
- 改进错误处理和输入验证
- 移除冗余的超时断言条件

## [1.0.0] - 2025-06-02

### Added

- ping-rs 首次正式发布
- 核心 ping 功能，采用 Rust 后端和 Python 绑定
- 同步 ping 操作（`ping_once`、`ping_multiple`）
- 非阻塞 ping 流（`create_ping_stream`、`PingStream`）
- Windows 平台专用 ping 实现，使用原生 ICMP
- 跨平台支持（Linux、macOS、Windows、BSD）
- 完整的 pytest 测试套件
- MIT 许可证
- GitHub Actions CI/CD 流水线
- Codecov 代码覆盖率集成
- **PingResult 类型**：Pong、Timeout、Unknown、PingExited
- **灵活的 API**：支持自定义超时、间隔和网络接口选择
- **IPv4/IPv6 支持**：可选协议选择
- **类型提示**：完整的类型注解支持，包含 `.pyi` 存根文件
- **高性能**：采用 Rust 构建，性能卓越
- 中英文全面的 README 文档
- API 参考文档
- 使用示例
- 架构文档
- PyO3：提供 Python-Rust 绑定
- pinger 库：跨平台 ping 功能
- tokio：异步运行时
- serde：序列化支持

[未发布]: https://github.com/a76yyyy/ping-rs/compare/v2.0.0...HEAD
[2.0.0]: https://github.com/a76yyyy/ping-rs/compare/v1.1.0...v2.0.0
[1.1.0]: https://github.com/a76yyyy/ping-rs/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/a76yyyy/ping-rs/releases/tag/v1.0.0
