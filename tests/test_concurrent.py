"""
并发性能测试
"""

import asyncio
import logging
import time
from ipaddress import ip_address

import pytest
from ping_rs import ping_multiple_async
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


if __name__ == "__main__":
    # 可以直接运行此文件进行测试
    _ = pytest.main(["-xvs", __file__])
