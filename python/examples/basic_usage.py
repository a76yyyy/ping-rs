"""
Basic usage examples for ping-rs Python bindings
"""

import asyncio
import time
from ipaddress import IPv4Address

from ping_rs import (
    AsyncPinger,
    AsyncPingStream,
    Pinger,
    create_ping_stream,
    ping_multiple,
    ping_multiple_async,
    ping_once,
    ping_once_async,
)


def sync_examples():
    """同步 API 示例"""
    # 单次 ping
    result = ping_once("8.8.8.8")
    print(f"Sync ping result: {result}")

    # IPv4 地址对象
    ip_addr = IPv4Address("8.8.8.8")
    result = ping_once(ip_addr)
    print(f"Sync ping with IPv4Address: {result}")

    # 多次 ping
    results = ping_multiple("8.8.8.8", count=3, interval_ms=500)
    print(f"Multiple pings: {len(results)} results")
    for idx, res in enumerate(results):
        print(f"  Result {idx + 1}: {res}")

    # 使用 Pinger 类
    pinger = Pinger("8.8.8.8", interval_ms=1000)
    result = pinger.ping_once()
    print(f"Pinger result: {result}")

    # 仅 IPv4
    pinger = Pinger("8.8.8.8", ipv4=True)
    result = pinger.ping_once()
    print(f"IPv4-only ping: {result}")


async def async_examples():
    """异步 API 示例"""
    # 单次异步 ping
    result = await ping_once_async("8.8.8.8")
    print(f"Async ping result: {result}")

    # 多次异步 ping
    start = time.time()
    results = await ping_multiple_async("8.8.8.8", count=3, interval_ms=500)
    print(f"Async multiple pings: {len(results)} results")
    print(f"Async multiple pings took: {time.time() - start:.2f} seconds")

    # 使用 Pinger 类的异步方法
    pinger = AsyncPinger("8.8.8.8")
    result = await pinger.ping_once()
    print(f"Async pinger result: {result}")


def stream_example():
    """Ping 流示例"""

    # 创建 ping 流
    stream = create_ping_stream("8.8.8.8", interval_ms=1000)

    # 尝试接收几个结果
    for i in range(5):
        result = stream.recv()
        if result is None:
            print("Stream ended")
            break
        print(f"Stream result {i + 1}: {result}")

        time.sleep(0.5)  # 短暂延迟


def sync_stream_example():
    """同步 Ping 流示例"""

    # 创建 ping 流
    stream = create_ping_stream("8.8.8.8", interval_ms=1000, count=5)

    # 同步接收几个结果
    i = 0
    for result in stream:
        print(f"Sync stream result {i + 1}: {result}")
        i += 1


async def async_ping_stream_example():
    """原生异步 Ping 流示例"""

    # 创建异步 ping 流
    stream = AsyncPingStream("8.8.8.8", interval_ms=1000, max_count=5)

    # 使用异步迭代器接收结果
    i = 0
    async for result in stream:
        print(f"Native async stream result {i + 1}: {result}")
        i += 1


if __name__ == "__main__":
    # 运行同步示例
    print("=== 运行同步示例 ===")
    sync_examples()

    # 运行异步示例
    print("\n=== 运行异步示例 ===")
    asyncio.run(async_examples())

    # 运行流示例
    print("\n=== 运行流示例 ===")
    stream_example()

    # 运行异步流示例
    print("\n=== 运行迭代器形式流示例 ===")
    sync_stream_example()

    # 运行原生异步流示例
    print("\n=== 运行原生异步流示例 ===")
    asyncio.run(async_ping_stream_example())
