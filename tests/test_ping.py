#!/usr/bin/env python3
"""
测试 ping-rs 包的基本功能
"""

import ipaddress
import logging
import time

import pytest
from ping_rs import create_ping_stream, ping_multiple, ping_multiple_async, ping_once, ping_once_async

logger = logging.getLogger(__name__)


def test_ping_once():
    """测试单次 ping"""
    logger.info("\n=== 测试单次 ping ===")
    # 测试字符串参数
    result = ping_once("localhost")
    assert result.is_success()
    logger.info(f"Ping 成功! 延迟: {result.duration_ms} ms")
    logger.info(f"原始输出: {result.line}")

    # 测试 IPv4Address 参数
    ip = ipaddress.IPv4Address("127.0.0.1")
    result = ping_once(ip)
    assert result.is_success()
    logger.info(f"Ping IPv4 成功! 延迟: {result.duration_ms} ms")
    logger.info(f"原始输出: {result.line}")


@pytest.mark.asyncio
async def test_ping_once_async():
    """测试异步单次 ping"""
    logger.info("\n=== 测试异步单次 ping ===")
    # 测试字符串参数
    result = await ping_once_async("localhost")
    assert result.is_success()
    logger.info(f"Ping 成功! 延迟: {result.duration_ms} ms")
    logger.info(f"原始输出: {result.line}")

    # 测试 IPv4Address 参数
    ip = ipaddress.IPv4Address("127.0.0.1")
    result = await ping_once_async(ip)
    assert result.is_success()
    logger.info(f"Ping IPv4 成功! 延迟: {result.duration_ms} ms")
    logger.info(f"原始输出: {result.line}")


def test_ping_multiple():
    logger.info("\n=== 测试多次 ping ===")
    # 测试字符串参数
    results = ping_multiple("localhost", count=3)
    for i, result in enumerate(results):
        assert result.is_success()
        logger.info(f"Ping {i + 1} 成功! 延迟: {result.duration_ms} ms")
        logger.info(f"原始输出: {result.line}")

    # 测试 IPv4Address 参数
    ip = ipaddress.IPv4Address("127.0.0.1")
    results = ping_multiple(ip, count=3)
    for i, result in enumerate(results):
        assert result.is_success()
        logger.info(f"Ping IPv4 {i + 1} 成功! 延迟: {result.duration_ms} ms")
        logger.info(f"原始输出: {result.line}")


@pytest.mark.asyncio
async def test_ping_multiple_async():
    """测试异步多次 ping"""
    logger.info("\n=== 测试异步多次 ping ===")

    # 测试字符串参数
    start = time.perf_counter()
    results = await ping_multiple_async("localhost", count=3, interval_ms=5000)
    for i, result in enumerate(results):
        assert result.is_success()
        logger.info(f"Ping {i + 1} 成功! 延迟: {result.duration_ms} ms")
        logger.info(f"原始输出: {result.line}")
    logger.info(f"耗时: {(time.perf_counter() - start) * 1000} ms")

    # 测试 IPv4Address 参数
    ip = ipaddress.IPv4Address("127.0.0.1")
    start = time.perf_counter()
    results = await ping_multiple_async(ip, count=3, interval_ms=5000)
    for result in results:
        assert result.is_success()
    logger.info(f"耗时 IPv4: {(time.perf_counter() - start) * 1000} ms")


def test_ping_stream():
    """测试非阻塞 ping 流"""
    logger.info("\n=== 测试非阻塞 ping 流 ===")

    # 测试字符串参数
    stream = create_ping_stream("localhost")
    count = 0
    max_count = 3

    logger.info("开始接收 ping 结果...")
    while stream.is_active() and count < max_count:
        result = stream.try_recv()
        if result is not None:
            count += 1
            assert result.is_success()
            logger.info(f"Ping {count} 成功! 延迟: {result.duration_ms} ms")
            logger.info(f"原始输出: {result.line}")
        time.sleep(0.1)  # 小延迟，避免忙等待

    # 测试 IPv4Address 参数
    ip = ipaddress.IPv4Address("127.0.0.1")
    stream = create_ping_stream(ip)
    count = 0
    max_count = 3

    logger.info("开始接收 IPv4 ping 结果...")
    while stream.is_active() and count < max_count:
        result = stream.try_recv()
        if result is not None:
            count += 1
            assert result.is_success()
            logger.info(f"Ping IPv4 {count} 成功! 延迟: {result.duration_ms} ms")
            logger.info(f"原始输出: {result.line}")
        time.sleep(0.1)  # 小延迟，避免忙等待


# 添加边界条件测试
def test_ping_edge_cases():
    """测试边界条件"""
    # 测试无效主机名
    result = ping_once("invalid.host.that.does.not.exist")
    assert not result.is_success()
    assert result.is_exited()

    # 测试零次 ping
    with pytest.raises(ValueError):
        _ = ping_multiple("127.0.0.1", count=0)

    # 测试负数 ping 次数
    with pytest.raises(ValueError):
        _ = ping_multiple("127.0.0.1", count=-1)


# 添加异常情况测试
@pytest.mark.asyncio
async def test_ping_exception_handling():
    """测试异常处理"""
    # 测试无效 IP 地址格式
    result = ping_once("not.an.ip.address/with/invalid/chars")
    assert result.is_exited()

    # 测试极小的间隔时间
    with pytest.raises(ValueError):  # 或更具体的异常类型
        _ = await ping_multiple_async("127.0.0.1", count=3, interval_ms=1)

    with pytest.raises(ValueError):
        _ = await ping_multiple_async("127.0.0.1", count=3, interval_ms=101)


if __name__ == "__main__":
    # 可以直接运行此文件进行测试
    _ = pytest.main(["-xvs", __file__])
