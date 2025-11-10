"""
并发性能测试
"""

import asyncio
import logging
import time
from collections.abc import Coroutine
from ipaddress import ip_address
from typing import Any

import pytest
from ping_rs import AsyncPingStream, PingResult, ping_multiple_async, ping_once_async
from ping_rs.core_schema import TargetType

logger = logging.getLogger(__name__)


async def ping_task(target: TargetType, count: int, interval_ms: int, task_id: int):
    """ping单个目标"""
    logger.info(f"任务 {task_id}: 开始ping {target}")
    start_time = time.time()
    results = await ping_multiple_async(target, count, interval_ms)
    stop_time = time.time()
    logger.info(f"任务 {task_id}: Ping {target} 探测完成，耗时 {stop_time - start_time:.3f}s.")
    return results


@pytest.mark.asyncio
async def test_high_concurrency(target: TargetType, count: int, interval_ms: int):
    """测试高并发ping"""
    # 测试目标 - 使用固定目标多次并发
    targets = [target] * 5  # 使用同一个目标创建10个任务

    logger.info(f"开始执行 {len(targets)} 个并发ping任务")

    # 创建所有ping任务
    tasks = [asyncio.create_task(ping_task(target, count, interval_ms, i)) for i, target in enumerate(targets)]

    # 等待所有任务完成
    start_time = time.time()
    results = await asyncio.gather(*tasks, return_exceptions=True)
    end_time = time.time()

    # 检查结果
    success_count = sum(1 for r in results if not isinstance(r, Exception))
    error_count = sum(1 for r in results if isinstance(r, Exception))

    logger.info(f"所有任务完成，总耗时: {end_time - start_time:.3f}s")
    logger.info(f"成功: {success_count}, 失败: {error_count}")

    # 验证结果
    assert success_count > 0
    assert success_count + error_count == len(tasks)
    expected_time = (count + 2) * interval_ms / 1000
    assert end_time - start_time < expected_time  # 允许一些误差


@pytest.mark.asyncio
async def test_multiple_targets_concurrency():
    """测试多目标并发ping"""
    # 测试目标 - 使用一些公共DNS服务器
    targets = [
        "8.8.8.8",  # Google DNS
        "1.1.1.1",  # Cloudflare DNS
        "9.9.9.9",  # Quad9 DNS
        "208.67.222.222",  # OpenDNS
        "114.114.114.114",  # 114DNS
        "223.5.5.5",  # AliDNS
        "119.29.29.29",  # DNSPod
        "180.76.76.76",  # Baidu DNS
    ]

    count = 3  # 每个目标ping的次数
    interval_ms = 1000  # ping间隔（毫秒）

    logger.info(f"开始执行 {len(targets)} 个并发ping任务")

    # 创建所有ping任务
    tasks = [
        asyncio.create_task(ping_task(ip_address(target), count, interval_ms, i)) for i, target in enumerate(targets)
    ]

    # 等待所有任务完成
    start_time = time.time()
    results = await asyncio.gather(*tasks, return_exceptions=True)
    end_time = time.time()

    # 检查结果
    success_count = sum(1 for r in results if not isinstance(r, Exception))
    error_count = sum(1 for r in results if isinstance(r, Exception))

    logger.info(f"所有任务完成，总耗时: {end_time - start_time:.3f}s")
    logger.info(f"成功: {success_count}, 失败: {error_count}")

    # 验证结果
    assert success_count > 0
    assert success_count + error_count == len(tasks)

    # 验证总耗时应该接近单个任务的耗时，而不是所有任务耗时的总和
    # 这表明任务是并行执行的
    expected_time = (count + 2) * interval_ms / 1000  # 预期单个任务的耗时（秒）
    assert end_time - start_time < expected_time  # 允许一些误差


@pytest.mark.asyncio
async def test_massive_concurrency():
    """测试大规模并发ping"""
    # 使用单个目标但创建大量并发任务
    target = "127.0.0.1"
    count = 3  # 每个目标ping的次数
    interval_ms = 1000  # ping间隔（毫秒）
    concurrency = 20  # 并发任务数

    logger.info(f"开始执行 {concurrency} 个并发ping任务")

    # 创建所有ping任务
    tasks = [asyncio.create_task(ping_task(ip_address(target), count, interval_ms, i)) for i in range(concurrency)]

    # 等待所有任务完成
    start_time = time.time()
    results = await asyncio.gather(*tasks, return_exceptions=True)
    end_time = time.time()

    # 检查结果
    success_count = sum(1 for r in results if not isinstance(r, Exception))
    error_count = sum(1 for r in results if isinstance(r, Exception))

    logger.info(f"所有任务完成，总耗时: {end_time - start_time:.3f}s")
    logger.info(f"成功: {success_count}, 失败: {error_count}")

    # 验证结果
    assert success_count > 0
    assert success_count + error_count == concurrency

    # 验证总耗时应该接近单个任务的耗时，而不是所有任务耗时的总和
    expected_time = (count + 2) * interval_ms / 1000  # 预期单个任务的耗时（秒）
    assert end_time - start_time < expected_time  # 允许一些误差


@pytest.mark.asyncio
async def test_error_handling():
    """测试错误处理"""
    # 使用无效目标测试错误处理
    invalid_target = "invalid.host.that.does.not.exist"
    count = 1
    interval_ms = 100

    # 创建ping任务
    task = ping_task(invalid_target, count, interval_ms, 999)

    # 等待任务完成
    result = await task

    # 验证结果 - 即使目标无效，也应该返回结果而不是抛出异常
    assert isinstance(result, list)

    # 检查结果中的失败情况
    success_count = sum(1 for r in result if r.is_success())
    failure_count = sum(1 for r in result if not r.is_success())

    logger.info(f"无效目标测试完成，成功: {success_count}, 失败: {failure_count}")

    # 大多数情况下应该失败
    assert success_count == 0
    assert failure_count == 1


# ============================================================================
# 性能和压力测试
# ============================================================================


@pytest.mark.asyncio
async def test_async_stream_performance() -> None:
    """
    测试异步流的性能

    验证：异步流能否高效处理大量结果
    """
    target = "127.0.0.1"
    max_count = 100

    stream = AsyncPingStream(target, interval_ms=100, max_count=max_count)

    start_time = time.time()
    results: list[PingResult] = []

    async for result in stream:
        results.append(result)

    elapsed = time.time() - start_time

    # 验证结果
    assert len(results) == max_count

    logger.info(f"异步流测试: {len(results)} 个结果, 耗时={elapsed:.2f}s")

    # 性能断言
    if elapsed < 15:  # 100 * 0.1s = 10s + 5s 缓冲
        logger.info("✅ 异步流性能良好")
    else:
        logger.warning(f"⚠️ 异步流性能较差 ({elapsed:.2f}s)")


# ============================================================================
# 内存和背压测试
# ============================================================================


@pytest.mark.asyncio
async def test_memory_usage_unbounded_channel():
    """
    测试无界通道的内存使用

    验证：无界通道在背压场景下是否会导致内存溢出
    注意：这个测试只能间接验证，真正的内存测试需要监控工具
    """
    target = "127.0.0.1"

    # 创建多个长时间运行的流 (interval_ms 最小为 100ms)
    streams = [AsyncPingStream(target, interval_ms=100, max_count=100) for _ in range(10)]

    # 慢速消费
    async def slow_consumer(stream: AsyncPingStream):
        results: list[PingResult] = []
        count = 0
        async for result in stream:
            results.append(result)
            count += 1
            if count % 100 == 0:
                await asyncio.sleep(0.1)  # 模拟慢速消费
        return results

    start_time = time.time()

    # 并发运行所有流
    tasks = [slow_consumer(stream) for stream in streams]
    results = await asyncio.gather(*tasks)

    elapsed = time.time() - start_time

    # 验证结果
    total_results = sum(len(r) for r in results)
    logger.info(f"内存测试: {total_results} 个结果, 耗时={elapsed:.2f}s")

    # 应该能正常完成 (10个流 × 100个结果 = 1000)
    assert total_results == 1000, f"结果数量不对: {total_results}"

    logger.info("✅ 内存测试通过，无界通道能正常处理")


@pytest.mark.asyncio
async def test_backpressure_handling() -> None:
    """
    测试背压处理

    验证：生产速度 > 消费速度时，无界通道是否会导致内存问题
    预期：应该能正常处理，但可能会有内存增长
    """
    target = "127.0.0.1"

    # 创建一个快速生产的流 (interval_ms 最小为 100ms)
    stream = AsyncPingStream(target, interval_ms=100, max_count=100)

    results: list[PingResult] = []
    start_time = time.time()

    # 慢速消费
    async for result in stream:
        results.append(result)
        if len(results) % 50 == 0:
            await asyncio.sleep(0.5)  # 每 50 个结果暂停一下

    elapsed = time.time() - start_time

    logger.info(f"背压测试: {len(results)} 个结果, 耗时={elapsed:.2f}s")

    # 验证结果完整
    assert len(results) == 100

    # 如果有背压，总时间应该受消费速度影响
    # 100 个结果，每 50 个暂停 0.5s，至少需要 1s (暂停时间) + 10s (ping时间)
    if elapsed >= 10.5:
        logger.info("✅ 背压机制可能正常工作")
    else:
        logger.warning(f"⚠️ 背压机制可能不完善 ({elapsed:.2f}s)")


# ============================================================================
# 综合压力测试
# ============================================================================


@pytest.mark.asyncio
async def test_comprehensive_stress_test() -> None:
    """
    综合压力测试

    测试多种操作在高压力场景下的表现
    """
    target = "127.0.0.1"

    # 创建多种类型的任务
    tasks: list[Coroutine[Any, Any, PingResult] | Coroutine[Any, Any, list[PingResult]]] = []  # pyright: ignore[reportExplicitAny]

    # 1. 多个 ping_once
    tasks.extend([ping_once_async(target, timeout_ms=5000) for _ in range(50)])

    # 2. 多个 ping_multiple
    tasks.extend([ping_multiple_async(target, count=20, interval_ms=100) for _ in range(10)])

    # 3. 异步流
    async def consume_stream() -> list[PingResult]:
        stream = AsyncPingStream(target, interval_ms=100, max_count=30)
        results: list[PingResult] = []
        async for result in stream:
            results.append(result)
        return results

    tasks.extend([consume_stream() for _ in range(5)])

    start_time = time.time()

    # 执行所有任务
    results = await asyncio.gather(*tasks, return_exceptions=True)

    elapsed = time.time() - start_time

    # 统计结果
    success_count = sum(1 for r in results if not isinstance(r, Exception))
    error_count = sum(1 for r in results if isinstance(r, Exception))

    logger.info(f"综合压力测试: 成功={success_count}, 失败={error_count}, 耗时={elapsed:.2f}s")

    # 验证大部分任务成功
    assert success_count > 60, f"成功率过低: {success_count}/{len(tasks)}"

    # 性能评估
    if elapsed < 30:
        logger.info("✅ 综合性能良好")
    else:
        logger.warning(f"⚠️ 综合性能较差 ({elapsed:.2f}s)")


# ============================================================================
# 性能基准测试
# ============================================================================


@pytest.mark.asyncio
async def test_performance_baseline() -> None:
    """
    性能基准测试

    用于对比不同场景下的性能差异

    注意: 使用 time.perf_counter() 而不是 time.time() 以获得更高精度
    """
    target = "127.0.0.1"

    # 测试 1: 单个 ping 延迟 (使用高精度计时器)
    start = time.perf_counter()
    _ = [await ping_once_async(target, timeout_ms=5000) for _ in range(100)]
    single_ping_latency = (time.perf_counter() - start) * 1000 / 100  # 转换为毫秒

    # 测试 2: 100 个 ping 的总时间
    start = time.perf_counter()
    _ = await ping_multiple_async(target, count=100, interval_ms=100)
    multiple_ping_time = time.perf_counter() - start

    # 测试 3: 并发性能
    start = time.perf_counter()
    _ = await asyncio.gather(*[ping_once_async(target, timeout_ms=5000) for _ in range(100)])
    concurrent_time = time.perf_counter() - start

    # 输出基准数据
    logger.info("=" * 60)
    logger.info("性能基准测试结果:")
    logger.info(f"  单个 ping 延迟: {single_ping_latency:.2f}ms")
    logger.info(f"  100 个 ping 总时间: {multiple_ping_time:.2f}s")
    logger.info(f"  100 个并发 ping 时间: {concurrent_time:.2f}s")
    logger.info("=" * 60)

    # 性能评估
    if single_ping_latency < 1.0:
        logger.info("✅ 单个 ping 延迟优秀 (<1ms)")
    elif single_ping_latency < 5.0:
        logger.info("⚠️ 单个 ping 延迟一般 (1-5ms)")
    else:
        logger.warning(f"❌ 单个 ping 延迟较差 ({single_ping_latency:.2f}ms)")

    if multiple_ping_time < 15:
        logger.info("✅ 批量 ping 性能优秀 (<15s)")
    elif multiple_ping_time < 30:
        logger.info("⚠️ 批量 ping 性能一般 (15-30s)")
    else:
        logger.warning(f"❌ 批量 ping 性能较差 ({multiple_ping_time:.2f}s)")

    if concurrent_time < 5:
        logger.info("✅ 并发性能优秀 (<5s)")
    elif concurrent_time < 15:
        logger.info("⚠️ 并发性能一般 (5-15s)")
    else:
        logger.warning(f"❌ 并发性能较差 ({concurrent_time:.2f}s)")


if __name__ == "__main__":
    # 可以直接运行此文件进行测试
    _ = pytest.main(["-xvs", __file__])
