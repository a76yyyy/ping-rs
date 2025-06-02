"""
超时功能测试
"""

import asyncio
import logging
import time

import pytest
from ping_rs import ping_multiple, ping_multiple_async, ping_once, ping_once_async
from ping_rs.core_schema import TargetType

logger = logging.getLogger(__name__)


@pytest.mark.parametrize("target", ["33.254.254.254"])
@pytest.mark.parametrize("timeout_ms", [1000, 2000, 3000])
def test_ping_once_timeout(target: TargetType, timeout_ms: int):
    """测试同步单次 ping 的超时功能"""
    start_time = time.time()

    # 使用不可达的 IP 地址进行测试
    result = ping_once(target, timeout_ms=timeout_ms)

    end_time = time.time()
    elapsed_time = (end_time - start_time) * 1000  # 转换为毫秒

    # 验证结果
    assert result is not None
    assert result.is_timeout()  # 应该是超时结果

    # 验证执行时间接近超时时间
    assert elapsed_time >= timeout_ms * 0.75  # 允许一些误差
    assert elapsed_time <= timeout_ms * 1.25  # 允许一些额外时间用于处理

    # 打印结果（可选）
    logger.info(f"超时时间 {timeout_ms} ms, 实际耗时: {elapsed_time:.2f} ms")


@pytest.mark.asyncio
@pytest.mark.parametrize("target", ["33.254.254.254"])
@pytest.mark.parametrize("timeout_ms", [1000, 2000, 3000])
async def test_ping_once_async_timeout(target: TargetType, timeout_ms: int):
    """测试异步单次 ping 的超时功能"""
    start_time = time.time()

    # 使用不可达的 IP 地址进行测试
    result = await ping_once_async(target, timeout_ms=timeout_ms)

    end_time = time.time()
    elapsed_time = (end_time - start_time) * 1000  # 转换为毫秒

    # 验证结果
    assert result is not None
    assert result.is_timeout()  # 应该是超时结果

    # 验证执行时间接近超时时间
    assert elapsed_time >= timeout_ms * 0.75  # 允许一些误差
    assert elapsed_time <= timeout_ms * 1.25  # 允许一些额外时间用于处理

    # 打印结果（可选）
    logger.info(f"异步超时时间 {timeout_ms} ms, 实际耗时: {elapsed_time:.2f} ms")


@pytest.mark.asyncio
@pytest.mark.parametrize("target", ["33.254.254.254"])
@pytest.mark.parametrize("timeout_ms", [1000, 2000, 3000])
async def test_ping_multiple_async_timeout(target: TargetType, timeout_ms: int):
    """测试异步多次 ping 的超时功能"""
    count = 10
    timeout_ms = 3000
    interval_ms = 500

    start_time = time.time()

    # 设置超时时间为 3 秒，但请求 10 个结果
    # 由于每个 ping 间隔为 1 秒，所以应该在超时前只能获取到约 3 个结果
    results = await ping_multiple_async(target, count=count, interval_ms=interval_ms, timeout_ms=timeout_ms)

    end_time = time.time()
    elapsed_time = (end_time - start_time) * 1000  # 转换为毫秒

    # 验证结果
    assert results is not None
    assert isinstance(results, list)
    assert len(results) < count  # 由于超时，应该获取不到所有结果

    # 验证执行时间接近超时时间
    assert elapsed_time >= timeout_ms * 0.75  # 允许一些误差
    assert elapsed_time <= timeout_ms + interval_ms * 2  # 允许一些额外时间用于处理

    # 打印结果（可选）
    logger.info(f"请求了 {count} 个结果，但由于 {timeout_ms} 毫秒超时，实际获取到 {len(results)} 个结果")
    logger.info(f"实际耗时: {elapsed_time:.2f} ms")


@pytest.mark.parametrize("target", ["33.254.254.254"])
@pytest.mark.parametrize("timeout_ms", [1000, 2000, 3000])
def test_ping_multiple_timeouts(target: TargetType, timeout_ms: int):
    """测试不同超时时间的同步多次 ping"""
    count = 10
    interval_ms = 500

    start_time = time.time()

    results = ping_multiple(target, count=count, interval_ms=interval_ms, timeout_ms=timeout_ms)

    end_time = time.time()
    elapsed_time = (end_time - start_time) * 1000  # 转换为毫秒

    # 验证结果
    assert results is not None
    assert isinstance(results, list)

    # 验证执行时间接近超时时间
    assert elapsed_time >= timeout_ms * 0.75  # 允许一些误差
    assert elapsed_time <= timeout_ms + interval_ms * 2  # 允许一些额外时间用于处理

    # 打印结果（可选）
    logger.info(f"超时时间 {timeout_ms} ms, 实际耗时: {elapsed_time:.2f} ms")


# 添加 ping_once 边界条件测试
def test_ping_once_edge_timeout_cases():
    """测试 ping_once 超时边界条件"""
    # 测试零超时
    with pytest.raises(ValueError):
        _ = ping_once("33.254.254.254", timeout_ms=0)

    # 测试负数超时
    with pytest.raises(ValueError):
        _ = ping_once("33.254.254.254", timeout_ms=-1)

    # 测试极小超时
    result = ping_once("33.254.254.254", timeout_ms=100)
    assert result is not None
    assert result.is_timeout()  # 应该是超时结果


# 添加 ping_once_async 边界条件测试
@pytest.mark.asyncio
async def test_ping_once_async_timeout_edge_cases():
    """测试 ping_once_async 超时边界条件"""
    # 测试零超时
    with pytest.raises(ValueError):
        _ = await ping_once_async("33.254.254.254", timeout_ms=0)

    # 测试负数超时
    with pytest.raises(ValueError):
        _ = await ping_once_async("33.254.254.254", timeout_ms=-1)

    # 测试极小超时
    result = await ping_once_async("33.254.254.254", timeout_ms=100)
    assert result is not None
    assert result.is_timeout()  # 应该是超时结果


# 添加边界条件测试
def test_ping_multiple_timeout_edge_cases():
    """测试超时边界条件"""
    # 测试零超时
    with pytest.raises(ValueError):
        results = ping_multiple("33.254.254.254", count=1, timeout_ms=0)

    # 测试负数超时
    with pytest.raises(ValueError):
        results = ping_multiple("33.254.254.254", count=1, timeout_ms=-1)

    # 测试极小超时
    results = ping_multiple("33.254.254.254", count=1, interval_ms=100, timeout_ms=100)
    assert len(results) == 1

    # 测试合理的超时值
    results = ping_multiple("33.254.254.254", count=1, timeout_ms=1000)
    assert len(results) == 1


# 添加异步边界条件测试
@pytest.mark.asyncio
async def test_async_timeout_edge_cases():
    """测试异步超时边界条件"""
    # 测试零超时
    with pytest.raises(ValueError):
        _ = await ping_multiple_async("33.254.254.254", count=1, timeout_ms=0)

    # 测试负数超时
    with pytest.raises(ValueError):
        _ = await ping_multiple_async("33.254.254.254", count=1, timeout_ms=-1)

    # 测试极小超时
    with pytest.raises(ValueError):
        _ = await ping_multiple_async("33.254.254.254", count=1, timeout_ms=1)


# 测试超时与并发
@pytest.mark.asyncio
async def test_timeout_with_concurrency():
    """测试超时与并发"""
    # 创建多个并发任务，每个都有不同的超时
    start = time.perf_counter()
    tasks = [
        ping_multiple_async("33.254.254.254", count=3, interval_ms=500, timeout_ms=1500),
        ping_multiple_async("33.254.254.254", count=3, interval_ms=1000, timeout_ms=3000),
        ping_multiple_async("33.254.254.254", count=3, interval_ms=3000, timeout_ms=9000),
    ]

    # 等待所有任务完成
    results = await asyncio.gather(*tasks)
    end = time.perf_counter()

    # 验证结果
    assert end - start < 12
    assert len(results) == 3
    for result_list in results:
        assert len(result_list) == 3

    start = time.perf_counter()
    tasks = [
        ping_once_async("33.254.254.254", timeout_ms=3000),
        ping_once_async("33.254.254.254", timeout_ms=3000),
        ping_once_async("33.254.254.254", timeout_ms=3000),
    ]
    _ = await asyncio.gather(*tasks)
    end = time.perf_counter()
    assert end - start < 6


if __name__ == "__main__":
    # 可以直接运行此文件进行测试
    _ = pytest.main(["-xvs", __file__])
