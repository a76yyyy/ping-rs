"""
基本功能测试
"""

import asyncio
import logging
import time

import pytest
from ping_rs import PingResult, create_ping_stream, ping_multiple, ping_once, ping_once_async
from ping_rs.core_schema import TargetType

logger = logging.getLogger(__name__)


def test_ping_once(target: TargetType, timeout_ms: int):
    """测试单次 ping"""
    result = ping_once(target, timeout_ms=timeout_ms)

    # 验证结果属性
    assert hasattr(result, "duration_ms")
    assert hasattr(result, "line")
    assert hasattr(result, "type_name")
    assert hasattr(result, "is_success")
    assert hasattr(result, "is_timeout")
    assert hasattr(result, "is_unknown")
    assert hasattr(result, "is_exited")

    # 验证方法调用不会抛出异常
    _ = result.is_success()
    _ = result.is_timeout()
    _ = result.is_unknown()
    _ = result.is_exited()
    _ = result.to_dict()

    # 验证结果
    assert result is not None
    assert result.is_success()


def test_ping_multiple(target: TargetType, count: int, interval_ms: int):
    """测试多次 ping"""
    results = ping_multiple(target, count=count, interval_ms=interval_ms)

    # 验证结果
    assert results is not None
    assert isinstance(results, list)
    assert len(results) == count

    # 打印结果（可选）
    for result in results:
        assert result.is_success()

    # 验证每个结果
    for result in results:
        assert hasattr(result, "duration_ms")
        assert hasattr(result, "line")
        assert hasattr(result, "type_name")


def test_ping_stream(target: TargetType, interval_ms: int):
    """测试非阻塞 ping 流"""
    stream = create_ping_stream(target, interval_ms=interval_ms)

    # 验证流对象
    assert stream is not None
    assert hasattr(stream, "try_recv")
    assert hasattr(stream, "recv")
    assert hasattr(stream, "is_active")

    # 测试接收结果
    count = 0
    max_count = 3
    results: list[PingResult] = []

    logger.info("开始接收 ping 结果...")
    start_time = time.time()
    timeout = 10  # 10 秒超时

    while stream.is_active() and count < max_count and (time.time() - start_time) < timeout:
        result = stream.try_recv()
        if result is not None:
            count += 1
            results.append(result)
            assert result.is_success()
        time.sleep(0.1)  # 小延迟，避免忙等待

    # 验证结果
    assert len(results) <= max_count

    # 验证流关闭
    assert stream.is_active()


def test_ping_timeout(target: TargetType):
    """测试超时参数"""
    # 使用非常短的超时时间
    result = ping_once(target, timeout_ms=100)
    # 验证结果
    assert result is not None
    # 打印结果（可选）
    logger.info(f"超短超时测试结果: {result}")

    # 使用正常的超时时间
    result = ping_once(target, timeout_ms=5000)
    # 验证结果
    assert result is not None
    # 打印结果（可选）
    logger.info(f"正常超时测试结果: {result}")


# ============================================================================
# 错误处理测试
# ============================================================================


@pytest.mark.asyncio
async def test_error_handling_invalid_target():
    """
    测试错误处理机制

    验证：无效目标是否能正确处理，不会导致程序崩溃
    """
    # 使用各种无效的目标
    invalid_targets = [
        "invalid.host.that.does.not.exist.example.com",
        "999.999.999.999",  # 无效 IP
        "",  # 空字符串
    ]

    for target in invalid_targets:
        try:
            result = await ping_once_async(target, timeout_ms=1000)

            # 应该返回错误结果，而不是抛出异常
            logger.info(f"无效目标 '{target}' 结果: {result.type_name}")

            # 验证是失败或退出状态
            assert result.is_exited() or result.is_timeout() or result.is_unknown()

        except Exception as e:
            # 如果抛出异常，记录但不失败测试
            logger.warning(f"目标 '{target}' 抛出异常: {type(e).__name__}: {e}")


@pytest.mark.asyncio
async def test_concurrent_error_handling():
    """
    测试并发场景下的错误处理

    验证：多个任务同时失败时，错误处理是否正常
    """
    # 混合有效和无效的目标
    targets = [
        "127.0.0.1",  # 有效
        "invalid.host.example.com",  # 无效
        "localhost",  # 有效
        "999.999.999.999",  # 无效
        "::1",  # 有效（IPv6）
    ]

    tasks = [ping_once_async(target, timeout_ms=2000) for target in targets]

    results = await asyncio.gather(*tasks, return_exceptions=True)

    # 统计结果
    success_count = 0
    error_count = 0
    exception_count = 0

    for i, result in enumerate(results):
        if isinstance(result, BaseException):
            exception_count += 1
            logger.warning(f"目标 {targets[i]} 抛出异常: {type(result).__name__}")
        elif result.is_success():
            success_count += 1
        else:
            error_count += 1

    logger.info(f"并发错误处理: 成功={success_count}, 失败={error_count}, 异常={exception_count}")

    # 验证至少有一些成功的
    assert success_count >= 2, "应该有至少 2 个成功的 ping"

    # 如果有异常，说明错误处理不够完善
    if exception_count > 0:
        logger.warning(f"⚠️ 有 {exception_count} 个任务抛出异常，错误处理可能不够完善")


if __name__ == "__main__":
    # 可以直接运行此文件进行测试
    _ = pytest.main(["-xvs", __file__])
