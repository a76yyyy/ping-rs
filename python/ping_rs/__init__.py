"""
ping-rs: Fast ping implementation using Rust with Python bindings

This package provides high-performance ping functionality with both synchronous
and asynchronous interfaces, leveraging Rust's performance and safety.
"""

from ping_rs._ping_rs import (
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

__all__ = [
    "PingResult",
    "Pinger",
    "PingStream",
    "__version__",
    "create_ping_stream",
    "ping_once",
    "ping_once_async",
    "ping_multiple",
    "ping_multiple_async",
]
