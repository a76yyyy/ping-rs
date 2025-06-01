"""
pytest 配置文件，提供共享的测试配置和夹具
"""

import pytest


def pytest_addoption(parser: pytest.Parser):
    """添加命令行选项"""
    parser.addoption("--target", default="127.0.0.1", help="目标主机名或 IP 地址")
    parser.addoption("--timeout", type=int, default=5000, help="超时时间（毫秒）")
    parser.addoption("--interval", type=int, default=1000, help="ping 间隔（毫秒）")
    parser.addoption("--count", type=int, default=3, help="ping 次数")


@pytest.fixture
def target(request: pytest.FixtureRequest):
    """目标主机名或 IP 地址"""
    return request.config.getoption("--target")


@pytest.fixture
def timeout_ms(request: pytest.FixtureRequest):
    """超时时间（毫秒）"""
    return request.config.getoption("--timeout")


@pytest.fixture
def interval_ms(request: pytest.FixtureRequest):
    """ping 间隔（毫秒）"""
    return request.config.getoption("--interval")


@pytest.fixture
def count(request: pytest.FixtureRequest):
    """ping 次数"""
    return request.config.getoption("--count")
