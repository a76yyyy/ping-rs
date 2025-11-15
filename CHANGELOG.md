# Changelog

[简体中文](./CHANGELOG_ZH.md) | English

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.2.0] - 2025-11-15

### Features

- Enhanced comprehensive test coverage with async ping error handling and performance tests
- Added stress testing, memory usage, backpressure, and performance benchmark tests

### Changes

- Unified timeout control across all ping operations
- Optimized async ping receiving logic, removed redundant blocking operations
- Refactored ICMP ping timeout calculation logic and result validation
- Migrated async ping result channels from blocking standard channels to Tokio async channels
- Simplified platform adaptation by using `pinger` library for all platforms

### Bug Fixes

- Adjusted async multiple ping test timeout parameters and result assertions to avoid flaky tests
- Fixed Windows platform ping timeout packet count calculation
- Fixed async ping timeout handling - now attempts to receive remaining results before stopping
- Fixed ping timeout logic and cross-platform timeout calculation
- Adjusted ping default timeout to 1000ms

### Removed

- Removed Windows-specific ping implementation and related utility code
- Removed IP resolution and timing-related Windows utility code

### Dependencies

- Upgraded `pyo3` and related libraries to version 0.27
- Upgraded `pyo3-log` to version 0.13
- Upgraded `pinger` to version 2.1.1

### Documentation

- Added cross-platform architecture explanation
- Improved README with platform support details

### Testing

- Enabled pytest parallel execution with `-n logical` option
- Added comprehensive async ping test coverage
- Adjusted test assertions to be more resilient

### Chore

- Upgraded pre-commit-hooks version
- Adjusted Makefile pre-commit startup command
- Optimized pyproject.toml dependency configuration and classification information

## [1.1.0] - 2025-06-08

### New Features

- **AsyncPinger class** for executing asynchronous ping operations
- **AsyncPingStream** for native async/await support with async iteration
- Added `count` parameter to `create_ping_stream` function
- Added validation functions for count and timeout parameters
- Usage examples for PingStream and AsyncPingStream
- Basic usage examples demonstrating synchronous and asynchronous ping operations

### Improvements

- Improved async ping capabilities and refined interfaces
- Refactored non-blocking receiver logic in PingStream
- Refactored sync_stream_example to remove async elements
- Updated CI configuration to run tests in parallel using `pytest-xdist`
- Conditionally compiled IP functions for Windows only
- Refactored Windows ping implementation for better clarity and maintainability
- Updated version to 1.1.0

### Bug Fixes

- Added AsyncPinger to the export list in `__all__`
- Improved error handling and input validation

### Breaking Changes

- Removed `psutil` dependency from pytest-xdist configuration
- Removed redundant timeout assertion conditions

### Documentation Updates

- Added comprehensive usage examples for PingStream as iterator
- Added examples for AsyncPingStream async iteration
- Enhanced documentation for asynchronous ping operations

## [1.0.0] - 2025-06-02

### Initial Release

- Initial release of ping-rs
- Core ping functionality with Rust backend and Python bindings
- Synchronous ping operations (`ping_once`, `ping_multiple`)
- Non-blocking ping stream (`create_ping_stream`, `PingStream`)
- Windows-specific ping implementation using native ICMP
- Cross-platform support (Linux, macOS, Windows, BSD)
- Comprehensive test suite with pytest
- MIT License
- CI/CD pipeline with GitHub Actions
- Code coverage integration with Codecov

### Core Features

- **PingResult types**: Pong, Timeout, Unknown, PingExited
- **Flexible API**: Support for custom timeout, interval, and interface selection
- **IPv4/IPv6 support**: Optional protocol selection
- **Type hints**: Full type annotation support with `.pyi` stub files
- **Performance**: Built with Rust for high performance

### Documentation

- Comprehensive README in English and Chinese
- API reference documentation
- Usage examples
- Architecture documentation

### Technical Stack

- PyO3 for Python-Rust bindings
- pinger library for cross-platform ping functionality
- tokio for async runtime
- serde for serialization

[Unreleased]: https://github.com/a76yyyy/ping-rs/compare/v1.2.0...HEAD
[1.2.0]: https://github.com/a76yyyy/ping-rs/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/a76yyyy/ping-rs/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/a76yyyy/ping-rs/releases/tag/v1.0.0
