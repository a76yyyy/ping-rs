"""
基本功能测试
"""

import logging
import time

import pytest
from ping_rs import PingResult, create_ping_stream, ping_multiple, ping_once
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


if __name__ == "__main__":
    # 可以直接运行此文件进行测试
    _ = pytest.main(["-xvs", __file__])
