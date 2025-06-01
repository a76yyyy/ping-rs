"""Type stubs for ping_rs Rust extension module."""

from typing import final, override

from ping_rs.core_schema import PingResultDict, TargetType

__version__: str

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

class PingResult:
    """Represents the result of a ping operation."""

    @final
    class Pong:
        """Successful ping result."""

        __match_args__ = ("duration_ms", "line")
        duration_ms: float
        line: str
        def __init__(self, duration_ms: float, line: str) -> None: ...

    @final
    class Timeout:
        """Timeout ping result."""

        __match_args__ = ("line",)
        line: str
        def __init__(self, line: str) -> None: ...

    @final
    class Unknown:
        """Unknown ping result."""

        __match_args__ = ("line",)
        line: str
        def __init__(self, line: str) -> None: ...

    @final
    class PingExited:
        """Ping process exited result."""

        __match_args__ = ("exit_code", "stderr")
        exit_code: int
        stderr: str
        def __init__(self, exit_code: int, stderr: str) -> None: ...

    @override
    def __repr__(self) -> str: ...
    @override
    def __str__(self) -> str: ...
    @property
    def duration_ms(self) -> float | None:
        """Get the ping duration in milliseconds, or None if not a successful ping."""
        ...

    @property
    def line(self) -> str:
        """Get the raw output line from the ping command."""
        ...

    @property
    def exit_code(self) -> int | None:
        """Get the exit code if this is a PingExited result, or None otherwise."""
        ...

    @property
    def stderr(self) -> str | None:
        """Get the stderr output if this is a PingExited result, or None otherwise."""
        ...

    @property
    def type_name(self) -> str:
        """Get the type name of this PingResult (Pong, Timeout, Unknown, or PingExited)."""
        ...

    def is_success(self) -> bool:
        """Check if this is a successful ping result."""
        ...

    def is_timeout(self) -> bool:
        """Check if this is a timeout result."""
        ...

    def is_unknown(self) -> bool:
        """Check if this is an unknown result."""
        ...

    def is_exited(self) -> bool:
        """Check if this is a ping process exit result."""
        ...

    def to_dict(self) -> PingResultDict:
        """Convert this PingResult to a dictionary."""
        ...

@final
class Pinger:
    """High-level ping interface."""

    def __init__(
        self,
        target: TargetType,
        interval_ms: int = 1000,
        interface: str | None = None,
        ipv4: bool = False,
        ipv6: bool = False,
    ) -> None: ...
    def ping_once(self) -> PingResult:
        """Execute a single ping synchronously."""
        ...

    async def ping_stream(self, count: int | None = None) -> list[PingResult]:
        """Execute multiple pings asynchronously."""
        ...

    @override
    def __repr__(self) -> str: ...

@final
class PingStream:
    """Non-blocking ping stream processor."""

    def try_recv(self) -> PingResult | None:
        """Try to receive the next ping result without blocking."""
        ...

    def recv(self) -> PingResult | None:
        """Receive the next ping result, blocking if necessary."""
        ...

    def is_active(self) -> bool:
        """Check if the stream is still active."""
        ...

def ping_once(
    target: TargetType,
    timeout_ms: int = 5000,
    interface: str | None = None,
    ipv4: bool = False,
    ipv6: bool = False,
) -> PingResult:
    """Execute a single ping operation synchronously."""
    ...

async def ping_once_async(
    target: TargetType,
    timeout_ms: int = 5000,
    interface: str | None = None,
    ipv4: bool = False,
    ipv6: bool = False,
) -> PingResult:
    """Execute a single ping operation asynchronously."""
    ...

def ping_multiple(
    target: TargetType,
    count: int = 4,
    interval_ms: int = 1000,
    timeout_ms: int | None = None,
    interface: str | None = None,
    ipv4: bool = False,
    ipv6: bool = False,
) -> list[PingResult]:
    """Execute multiple ping operations synchronously."""
    ...

async def ping_multiple_async(
    target: TargetType,
    count: int = 4,
    interval_ms: int = 1000,
    timeout_ms: int | None = None,
    interface: str | None = None,
    ipv4: bool = False,
    ipv6: bool = False,
) -> list[PingResult]:
    """Execute multiple ping operations asynchronously."""
    ...

def create_ping_stream(
    target: TargetType,
    interval_ms: int = 1000,
    interface: str | None = None,
    ipv4: bool = False,
    ipv6: bool = False,
) -> PingStream:
    """Create a non-blocking ping stream."""
    ...
