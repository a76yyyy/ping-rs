# ping-rs

简体中文 | [English](./README.md)

一个用 Rust 构建并提供 Python 接口的高性能网络 ping 库。

本库提供了快速可靠的 ping 功能，同时支持同步和异步接口。通过利用 Rust 的性能和安全保证，`ping-rs` 为传统的 Python ping 实现提供了一个高效的替代方案。

## 安装

```bash
uv add ping-rs
```

## 使用方法

> **注意:** 如果遇到错误 `RuntimeError: Failed to start ping: Could not detect ping.`，
> 请先安装 ping 工具：
>
> ```bash
> # Debian/Ubuntu 系统
> sudo apt-get install iputils-ping
> ```

### 基本用法（同步）

```python
from ping_rs import ping_once

# 简单的 ping（同步）
result = ping_once("google.com")
if result.is_success():
    print(f"Ping 成功！延迟: {result.duration_ms} ms")
else:
    print("Ping 失败")
```

### 异步用法

```python
import asyncio
from ping_rs import ping_once_async, ping_multiple_async

async def ping_test():
    # 单次异步 ping
    result = await ping_once_async("google.com")
    if result.is_success():
        print(f"Ping 成功！延迟: {result.duration_ms} ms")
    else:
        print("Ping 失败")

    # 多次异步 ping
    results = await ping_multiple_async("google.com", count=5)
    for i, result in enumerate(results):
        if result.is_success():
            print(f"Ping {i+1}: {result.duration_ms} ms")
        else:
            print(f"Ping {i+1}: 失败")

# 运行异步函数
asyncio.run(ping_test())
```

### 多次 Ping（同步）

```python
from ping_rs import ping_multiple

# 多次 ping（同步）
results = ping_multiple("google.com", count=5)
for i, result in enumerate(results):
    if result.is_success():
        print(f"Ping {i+1}: {result.duration_ms} ms")
    else:
        print(f"Ping {i+1}: 失败")
```

### 使用超时

```python
from ping_rs import ping_multiple

# 带超时的多次 ping（将在 3 秒后停止）
results = ping_multiple("google.com", count=10, timeout_ms=3000)
print(f"超时前收到 {len(results)} 个结果")
```

### 非阻塞流

```python
import time
from ping_rs import create_ping_stream

# 创建非阻塞 ping 流
stream = create_ping_stream("google.com")

# 处理结果
while stream.is_active():
    result = stream.try_recv()
    if result is not None:
        if result.is_success():
            print(f"Ping: {result.duration_ms} ms")
        else:
            print("Ping 失败")
    time.sleep(0.1)  # 小延迟，避免忙等待
```

## API 参考

### 函数

- `ping_once(target, timeout_ms=5000, interface=None, ipv4=False, ipv6=False)`: 同步执行单次 ping 操作
- `ping_once_async(target, timeout_ms=5000, interface=None, ipv4=False, ipv6=False)`: 异步执行单次 ping 操作
- `ping_multiple(target, count=4, interval_ms=1000, timeout_ms=None, interface=None, ipv4=False, ipv6=False)`: 同步执行多次 ping 操作
- `ping_multiple_async(target, count=4, interval_ms=1000, timeout_ms=None, interface=None, ipv4=False, ipv6=False)`: 异步执行多次 ping 操作
- `create_ping_stream(target, interval_ms=1000, interface=None, ipv4=False, ipv6=False)`: 创建非阻塞 ping 流

### 类

#### PingResult

表示 ping 操作的结果。

- `duration_ms`: 获取 ping 延迟（毫秒）（如果不成功则为 None）
- `line`: 获取来自 ping 命令的原始输出行
- `exit_code`: 如果这是 PingExited 结果，则获取退出代码，否则为 None
- `stderr`: 如果这是 PingExited 结果，则获取标准错误输出，否则为 None
- `type_name`: 获取 PingResult 的类型名称（Pong、Timeout、Unknown 或 PingExited）
- `is_success()`: 检查这是否是成功的 ping 结果
- `is_timeout()`: 检查这是否是超时结果
- `is_unknown()`: 检查这是否是未知结果
- `is_exited()`: 检查这是否是 ping 进程退出结果
- `to_dict()`: 将 PingResult 转换为字典

#### Pinger

高级 ping 接口。

- `__init__(target, interval_ms=1000, interface=None, ipv4=False, ipv6=False)`: 初始化 Pinger
- `ping_once()`: 同步执行单次 ping
- `ping_stream(count=None)`: 异步执行多次 ping

#### PingStream

非阻塞 ping 流处理器。

- `try_recv()`: 尝试接收下一个 ping 结果，不阻塞
- `recv()`: 接收下一个 ping 结果，如有必要则阻塞
- `is_active()`: 检查流是否仍处于活动状态

## 开发

### 高级用法示例

#### 处理 PingResult 类型

```python
from ping_rs import ping_once

# 使用模式匹配（Python 3.10+）
result = ping_once("google.com")
match result:
    case result if result.is_success():
        print(f"成功: {result.duration_ms} ms")
    case result if result.is_timeout():
        print("超时")
    case result if result.is_unknown():
        print(f"未知响应: {result.line}")
    case result if result.is_exited():
        print(f"Ping 进程退出，退出代码 {result.exit_code}")
        print(f"错误信息: {result.stderr}")
    case _:
        print("意外的结果类型")

# 将结果转换为字典以进行数据处理
result = ping_once("google.com")
result_dict = result.to_dict()
print(result_dict)  # {'type': 'Pong', 'duration_ms': 15.2, 'line': 'Reply from...'}
```

#### PingResult 类型

PingResult 可以是以下类型之一：

1. **Pong** - 成功的 ping 响应

   - `duration_ms` - Ping 延迟（毫秒）
   - `line` - 来自 ping 命令的原始输出行

2. **Timeout** - Ping 超时

   - `line` - 包含超时信息的原始输出行

3. **Unknown** - 无法识别的 ping 响应

   - `line` - 无法解析的原始输出行

4. **PingExited** - Ping 进程意外退出
   - `exit_code` - Ping 进程的退出代码
   - `stderr` - Ping 进程的错误输出

### 运行测试

该包在 `tests` 目录中包含全面的测试套件。要运行测试：

```bash
# 运行所有测试
cd /path/to/ping-rs
python -m tests.run_all_tests
```

### 从源码构建

要从源代码构建包：

```bash
cd /path/to/ping-rs
maturin develop
```

## 致谢

本库使用了以下 Rust 库：

- [pinger](https://crates.io/crates/pinger)：提供跨平台方式执行 ping 命令并解析输出。当前作为 [gping](https://github.com/orf/gping) 项目的一部分开发。
- [winping](https://crates.io/crates/winping)：在 Windows 平台上启用原生 ICMP ping 功能，无需依赖外部命令。

## 许可证

MIT 许可证
