# Changelog

[简体中文](./CHANGELOG_ZH.md) | English

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.1] - 2025-11-15

### Fixed

- Ensure typing override compatibility on Python < 3.12 by importing `override` from `typing_extensions` in stubs
- Align `PingStream` iterator protocol in stubs: make `__next__` synchronous and add constructor parameters to match Rust implementation
- Remove duplicate top-level stub to avoid drift; keep authoritative stub at `ping_rs/_ping_rs.pyi`
- Correct `PingResult` stubs: convert nested variant constructors to `__new__` and annotate base with `@disjoint_base` to improve typing and pattern matching

### Changed

- Updated development/lint dependencies; add `typing_extensions` and refresh dependency set
- CI: enable workflow concurrency cancellation to avoid duplicate runs on rapid pushes
- Release: derive changelog version by stripping leading `v` from tag for changelog reader action
- GitHub Release: treat tags containing `rc` as prereleases (in addition to `alpha`/`beta`)
- CI: unify continue-on-error settings to `false` for stability
- CI: simplify test/build matrices; temporarily drop free-threaded 3.13t and CPython 3.14 from testing

## [2.0.0] - 2025-11-15

### Added

- Enhanced comprehensive test coverage with async ping error handling and performance tests
- Added stress testing, memory usage, backpressure, and performance benchmark tests
- Enabled pytest parallel execution with `-n logical` option
- Added comprehensive async ping test coverage
- Added cross-platform architecture explanation
- Improved README with platform support details

### Changed

- Unified timeout control across all ping operations
- Optimized async ping receiving logic, removed redundant blocking operations
- Refactored ICMP ping timeout calculation logic and result validation
- Migrated async ping result channels from blocking standard channels to Tokio async channels
- Simplified platform adaptation by using `pinger` library for all platforms
- Adjusted test assertions to be more resilient
- Upgraded `pyo3` and related libraries to version 0.27
- Upgraded `pyo3-log` to version 0.13
- Upgraded `pinger` to version 2.1.1
- Upgraded pre-commit-hooks version
- Adjusted Makefile pre-commit startup command
- Optimized pyproject.toml dependency configuration and classification information

### Fixed

- Adjusted async multiple ping test timeout parameters and result assertions to avoid flaky tests
- Fixed Windows platform ping timeout packet count calculation
- Fixed async ping timeout handling - now attempts to receive remaining results before stopping
- Fixed ping timeout logic and cross-platform timeout calculation
- Adjusted ping default timeout to 1000ms

### Removed

- Removed Windows-specific ping implementation and related utility code
- Removed IP resolution and timing-related Windows utility code

## [1.1.0] - 2025-06-08

### Added

- **AsyncPinger class** for executing asynchronous ping operations
- **AsyncPingStream** for native async/await support with async iteration
- Added `count` parameter to `create_ping_stream` function
- Added validation functions for count and timeout parameters
- Usage examples for PingStream and AsyncPingStream
- Basic usage examples demonstrating synchronous and asynchronous ping operations
- Added comprehensive usage examples for PingStream as iterator
- Added examples for AsyncPingStream async iteration
- Enhanced documentation for asynchronous ping operations

### Changed

- Improved async ping capabilities and refined interfaces
- Refactored non-blocking receiver logic in PingStream
- Refactored sync_stream_example to remove async elements
- Updated CI configuration to run tests in parallel using `pytest-xdist`
- Conditionally compiled IP functions for Windows only
- Refactored Windows ping implementation for better clarity and maintainability
- Removed `psutil` dependency from pytest-xdist configuration
- Updated version to 1.1.0

### Fixed

- Added AsyncPinger to the export list in `__all__`
- Improved error handling and input validation
- Removed redundant timeout assertion conditions

## [1.0.0] - 2025-06-02

### Added

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
- **PingResult types**: Pong, Timeout, Unknown, PingExited
- **Flexible API**: Support for custom timeout, interval, and interface selection
- **IPv4/IPv6 support**: Optional protocol selection
- **Type hints**: Full type annotation support with `.pyi` stub files
- **Performance**: Built with Rust for high performance
- Comprehensive README in English and Chinese
- API reference documentation
- Usage examples
- Architecture documentation
- PyO3 for Python-Rust bindings
- pinger library for cross-platform ping functionality
- tokio for async runtime
- serde for serialization

[Unreleased]: https://github.com/a76yyyy/ping-rs/compare/v2.0.1...HEAD
[2.0.1]: https://github.com/a76yyyy/ping-rs/compare/v2.0.0...v2.0.1
[2.0.0]: https://github.com/a76yyyy/ping-rs/compare/v1.1.0...v2.0.0
[1.1.0]: https://github.com/a76yyyy/ping-rs/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/a76yyyy/ping-rs/releases/tag/v1.0.0
