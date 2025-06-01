#!/usr/bin/env python3
"""
运行所有 ping-rs 测试
"""

import os
import sys

import pytest

# 添加项目根目录到 Python 路径
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))


def main():
    """运行所有测试"""
    # 运行当前目录下的所有测试
    return pytest.main(["-xvs", os.path.dirname(__file__)])


if __name__ == "__main__":
    sys.exit(main())
