# ping-rs 测试

本目录包含 ping-rs 库的测试脚本，使用 pytest 框架组织。

## 测试文件

- `test_basic.py`: 基本功能测试（同步 ping、多次 ping、ping 流）
- `test_async.py`: 异步功能测试（异步 ping、并发 ping）
- `test_timeout.py`: 超时功能测试（不同超时时间的测试）
- `test_concurrent.py`: 并发性能测试（高并发、多目标并发、大规模并发）
- `conftest.py`: pytest 配置和共享夹具
- `pytest.ini`: pytest 配置文件
- `run_all_tests.py`: 运行所有测试的脚本

## 安装测试依赖

```bash
pip install -r tests/requirements.txt
```

## 运行测试

### 运行所有测试

```bash
cd /path/to/ping-rs
python -m tests.run_all_tests
```

或者直接使用 pytest：

```bash
cd /path/to/ping-rs
pytest tests/
```

### 运行单个测试文件

```bash
cd /path/to/ping-rs
pytest tests/test_basic.py
pytest tests/test_async.py
pytest tests/test_timeout.py
pytest tests/test_concurrent.py
```

### 运行特定测试函数

```bash
cd /path/to/ping-rs
pytest tests/test_basic.py::test_ping_once
```

### 使用自定义参数

```bash
cd /path/to/ping-rs
pytest tests/ --target=localhost --timeout=2000
```

## 测试环境要求

- Python 3.7+
- pytest 7.0.0+
- pytest-asyncio 0.18.0+
- ping-rs 库已安装（可以使用 `maturin develop` 安装开发版本）
