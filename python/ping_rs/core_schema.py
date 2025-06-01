from ipaddress import IPv4Address, IPv6Address
from typing import Literal, TypedDict


class PongResult(TypedDict):
    type: Literal["Pong"]
    duration_ms: float
    line: str


class TimeoutResult(TypedDict):
    type: Literal["Timeout"]
    line: str


class UnknownResult(TypedDict):
    type: Literal["Unknown"]
    line: str


class PingExitedResult(TypedDict):
    type: Literal["PingExited"]
    exit_code: int
    stderr: str


PingResultDict = PongResult | TimeoutResult | UnknownResult | PingExitedResult
# 定义 IP 地址类型
TargetType = str | IPv4Address | IPv6Address
