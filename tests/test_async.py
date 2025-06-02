"""
异步功能测试
"""

import asyncio
import logging

import pytest
from ping_rs import ping_multiple_async, ping_once_async
from ping_rs.core_schema import TargetType

logger = logging.getLogger(__name__)


@pytest.mark.asyncio
async def test_ping_once_async(
    target: TargetType,
    timeout_ms: int,
):
    """测试异步单次 ping"""
    result = await ping_once_async(target, timeout_ms=timeout_ms)

    # 验证结果属性
    assert hasattr(result, "duration_ms")
    assert hasattr(result, "line")
    assert hasattr(result, "type_name")
    assert hasattr(result, "is_success")
    assert hasattr(result, "is_timeout")
    assert hasattr(result, "is_unknown")
    assert hasattr(result, "is_exited")

    assert result.is_success()


@pytest.mark.asyncio
async def test_ping_multiple_async(target: TargetType, count: int, interval_ms: int):
    """测试异步多次 ping"""
    results = await ping_multiple_async(target, count=count, interval_ms=interval_ms)

    # 验证结果
    assert results is not None
    assert isinstance(results, list)
    assert len(results) == count

    # 验证每个结果
    for result in results:
        assert hasattr(result, "duration_ms")
        assert hasattr(result, "line")
        assert hasattr(result, "type_name")

        assert result.is_success()


@pytest.mark.asyncio
async def test_concurrent_pings(target: TargetType, timeout_ms: int):
    """测试并发 ping"""
    # 并发执行多个异步 ping
    tasks = [
        ping_once_async(target, timeout_ms=timeout_ms),
        ping_once_async(target, timeout_ms=timeout_ms),
        ping_once_async(target, timeout_ms=timeout_ms),
        ping_once_async(target, timeout_ms=timeout_ms),
    ]

    results = await asyncio.gather(*tasks)

    # 验证结果
    assert len(results) == 4

    # 打印结果（可选）
    for result in results:
        assert result.is_success()


@pytest.mark.asyncio
async def test_ping_timeout_async(target: TargetType):
    """测试异步超时参数"""
    # 使用非常短的超时时间
    with pytest.raises(ValueError):
        result = await ping_once_async(target, timeout_ms=1)

    # 使用正常的超时时间
    result = await ping_once_async(target, timeout_ms=5000)

    # 验证结果
    assert result is not None
    assert result.is_success()


# 添加边界条件测试
@pytest.mark.asyncio
async def test_async_edge_cases():
    """测试异步边界条件"""
    # 测试无效主机名
    result = await ping_once_async("invalid.host.that.does.not.exist")
    assert not result.is_success()
    assert result.is_exited()

    # 测试零次 ping
    with pytest.raises(ValueError):
        _ = await ping_multiple_async("127.0.0.1", count=0)

    # 测试负数 ping 次数
    with pytest.raises(ValueError):
        _ = await ping_multiple_async("127.0.0.1", count=-1)


# 添加异常情况测试
@pytest.mark.asyncio
async def test_async_exception_handling():
    """测试异步异常处理"""
    # 测试无效 IP 地址格式
    with pytest.raises(TypeError):
        _ = await ping_once_async(1)  # type: ignore[reportArgumentType]

    # 测试极小的间隔时间
    results = await ping_multiple_async("127.0.0.1", count=3, interval_ms=100)
    assert len(results) == 3

    # 测试超大 ping 次数，但使用合理的值
    results = await ping_multiple_async("127.0.0.1", count=10, interval_ms=1000)
    assert len(results) == 10


# 测试并发错误处理
@pytest.mark.asyncio
async def test_concurrent_error_handling():
    """测试并发错误处理"""
    # 混合有效和无效目标
    tasks = [
        ping_once_async("127.0.0.1"),  # 有效
        ping_once_async("invalid.host.that.does.not.exist"),  # 无效
        ping_once_async("localhost"),  # 有效
    ]

    # 使用 gather 并设置 return_exceptions=True 来捕获异常
    results = await asyncio.gather(*tasks, return_exceptions=True)

    # 验证结果
    assert len(results) == 3

    # 检查结果中的成功和失败情况
    success_count = sum(1 for r in results if not isinstance(r, BaseException) and r.is_success())
    failure_count = sum(1 for r in results if not isinstance(r, BaseException) and not r.is_success())
    exception_count = sum(1 for r in results if isinstance(r, BaseException))

    logger.info(f"并发错误处理测试: 成功={success_count}, 失败={failure_count}, 异常={exception_count}")

    assert success_count == 2
    assert failure_count == 1
    assert exception_count == 0


if __name__ == "__main__":
    # 可以直接运行此文件进行测试
    _ = pytest.main(["-xvs", __file__])
