"""
ping-rs: Fast ping implementation using Rust with Python bindings

This package provides high-performance ping functionality with both synchronous
and asynchronous interfaces, leveraging Rust's performance and safety.
"""

from ping_rs._ping_rs import (
    AsyncPingStream,
    Pinger,
    PingResult,
    PingStream,
    __version__,
    create_ping_stream,
    ping_multiple,
    ping_multiple_async,
    ping_once,
    ping_once_async,
)
from ping_rs.core_schema import (
    PingExitedResult,
    PingResultDict,
    PongResult,
    TargetType,
    TimeoutResult,
    UnknownResult,
)

__all__ = [
    # 从 Rust 核心导出的类和函数
    "AsyncPingStream",
    "PingResult",
    "Pinger",
    "PingStream",
    "__version__",
    "create_ping_stream",
    "ping_once",
    "ping_once_async",
    "ping_multiple",
    "ping_multiple_async",
    # 从 core_schema 导出的类型定义，便于静态类型检查
    "PongResult",
    "TimeoutResult",
    "UnknownResult",
    "PingExitedResult",
    "PingResultDict",
    "TargetType",
]
