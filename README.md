# ping-rs

[简体中文](./README_ZH.md) | English

A high-performance network ping library built with Rust and exposed to Python.

This package provides fast and reliable ping functionality with both synchronous and asynchronous interfaces. By leveraging Rust's performance and safety guarantees, `ping-rs` offers an efficient alternative to traditional Python ping implementations.

## Installation

```bash
uv add ping-rs
```

## Usage

> **Note:** If you encounter the error `RuntimeError: Failed to start ping: Could not detect ping.`,
> please install the ping utility first:
>
> ```bash
> # On Debian/Ubuntu
> sudo apt-get install iputils-ping
> ```

### Basic Usage (Synchronous)

```python
from ping_rs import ping_once

# Simple ping (synchronous)
result = ping_once("google.com")
if result.is_success():
    print(f"Ping successful! Latency: {result.duration_ms} ms")
else:
    print("Ping failed")
```

### Asynchronous Usage

```python
import asyncio
from ping_rs import ping_once_async, ping_multiple_async

async def ping_test():
    # Single ping asynchronously
    result = await ping_once_async("google.com")
    if result.is_success():
        print(f"Ping successful! Latency: {result.duration_ms} ms")
    else:
        print("Ping failed")

    # Multiple pings asynchronously
    results = await ping_multiple_async("google.com", count=5)
    for i, result in enumerate(results):
        if result.is_success():
            print(f"Ping {i+1}: {result.duration_ms} ms")
        else:
            print(f"Ping {i+1}: Failed")

# Run the async function
asyncio.run(ping_test())
```

### Multiple Pings (Synchronous)

```python
from ping_rs import ping_multiple

# Multiple pings (synchronous)
results = ping_multiple("google.com", count=5)
for i, result in enumerate(results):
    if result.is_success():
        print(f"Ping {i+1}: {result.duration_ms} ms")
    else:
        print(f"Ping {i+1}: Failed")
```

### Using Timeout

```python
from ping_rs import ping_multiple

# Multiple pings with timeout (will stop after 3 seconds)
results = ping_multiple("google.com", count=10, timeout_ms=3000)
print(f"Received {len(results)} results before timeout")
```

### Non-blocking Stream

```python
import time
from ping_rs import create_ping_stream

# Create a non-blocking ping stream
stream = create_ping_stream("google.com")

# Process results as they arrive
while stream.is_active():
    result = stream.try_recv()
    if result is not None:
        if result.is_success():
            print(f"Ping: {result.duration_ms} ms")
        else:
            print("Ping failed")
    time.sleep(0.1)  # Small delay to avoid busy waiting
```

## API Reference

### Functions

- `ping_once(target, timeout_ms=5000, interface=None, ipv4=False, ipv6=False)`: Execute a single ping operation synchronously
- `ping_once_async(target, timeout_ms=5000, interface=None, ipv4=False, ipv6=False)`: Execute a single ping operation asynchronously
- `ping_multiple(target, count=4, interval_ms=1000, timeout_ms=None, interface=None, ipv4=False, ipv6=False)`: Execute multiple pings synchronously
- `ping_multiple_async(target, count=4, interval_ms=1000, timeout_ms=None, interface=None, ipv4=False, ipv6=False)`: Execute multiple pings asynchronously
- `create_ping_stream(target, interval_ms=1000, interface=None, ipv4=False, ipv6=False)`: Create a non-blocking ping stream

### Classes

#### PingResult

Represents the result of a ping operation.

- `duration_ms`: Get the ping duration in milliseconds (None if not successful)
- `line`: Get the raw output line from the ping command
- `exit_code`: Get the exit code if this is a PingExited result, or None otherwise
- `stderr`: Get the stderr output if this is a PingExited result, or None otherwise
- `type_name`: Get the type name of this PingResult (Pong, Timeout, Unknown, or PingExited)
- `is_success()`: Check if this is a successful ping result
- `is_timeout()`: Check if this is a timeout result
- `is_unknown()`: Check if this is an unknown result
- `is_exited()`: Check if this is a ping process exit result
- `to_dict()`: Convert this PingResult to a dictionary

#### Pinger

High-level ping interface.

- `__init__(target, interval_ms=1000, interface=None, ipv4=False, ipv6=False)`: Initialize a Pinger
- `ping_once()`: Execute a single ping synchronously
- `ping_stream(count=None)`: Execute multiple pings asynchronously

#### PingStream

Non-blocking ping stream processor.

- `try_recv()`: Try to receive the next ping result without blocking
- `recv()`: Receive the next ping result, blocking if necessary
- `is_active()`: Check if the stream is still active

## Development

### Advanced Usage Examples

#### Working with PingResult Types

```python
from ping_rs import ping_once

# Using pattern matching (Python 3.10+)
result = ping_once("google.com")
match result:
    case result if result.is_success():
        print(f"Success: {result.duration_ms} ms")
    case result if result.is_timeout():
        print("Timeout")
    case result if result.is_unknown():
        print(f"Unknown response: {result.line}")
    case result if result.is_exited():
        print(f"Ping process exited with code {result.exit_code}")
        print(f"Error message: {result.stderr}")
    case _:
        print("Unexpected result type")

# Converting results to dictionaries for data processing
result = ping_once("google.com")
result_dict = result.to_dict()
print(result_dict)  # {'type': 'Pong', 'duration_ms': 15.2, 'line': 'Reply from...'}
```

#### PingResult Types

PingResult can be one of the following types:

1. **Pong** - Successful ping response

   - `duration_ms` - Ping duration in milliseconds
   - `line` - Raw output line from ping command

2. **Timeout** - Ping timeout

   - `line` - Raw output line with timeout information

3. **Unknown** - Unrecognized ping response

   - `line` - Raw output line that couldn't be parsed

4. **PingExited** - Ping process exited unexpectedly
   - `exit_code` - Exit code of the ping process
   - `stderr` - Error output from the ping process

### Running Tests

The package includes a comprehensive test suite in the `tests` directory. To run the tests:

```bash
# Run all tests
cd /path/to/ping-rs
python -m tests.run_all_tests
```

### Building from Source

To build the package from source:

```bash
cd /path/to/ping-rs
maturin develop
```

## Acknowledgements

This package uses the following Rust libraries:

- [pinger](https://crates.io/crates/pinger): Provides a cross-platform way to execute ping commands and parse their output. Currently developed as part of the [gping](https://github.com/orf/gping) project.
- [winping](https://crates.io/crates/winping): Enables native ICMP ping functionality on Windows platforms without relying on external commands.

## License

MIT License
