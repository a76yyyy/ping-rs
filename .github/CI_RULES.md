# CI 触发规则

## 快速参考

| 场景                              | 测试 Jobs | 构建 Jobs | 说明           |
| --------------------------------- | --------- | --------- | -------------- |
| PR: 仅修改文档 (`.md`, `LICENSE`) | ❌        | ❌        | 跳过所有       |
| PR: 修改代码/测试/依赖/CI 配置    | ✅        | ❌        | 仅运行测试     |
| PR: 添加 `Full Build` 标签        | ✅        | ✅        | 强制完整构建   |
| Push 到 main: 仅修改文档          | ❌        | ❌        | 跳过所有       |
| Push 到 main: 修改代码/依赖       | ✅        | ✅        | 完整测试和构建 |
| Tag push (发版)                   | ✅        | ✅        | 所有 + Release |

## 详细说明

### 测试 Jobs

- `coverage`, `test-python`, `test-os`, `test-msrv`, `test-debug`, `lint`
- **触发条件**: 代码/依赖/CI 配置变更，或 tag push

### 构建 Jobs

- `build`, `build-pgo`, `build-sdist`
- **触发条件**:
  - Tag push (发版)
  - PR 带 `Full Build` 标签
  - Push 到 main **且** 有代码/依赖变更

### Release Job

- 仅在 tag push 时触发
- 自动发布到 PyPI 和创建 GitHub Release（带双语 Release Notes）

## 文件变更分类

- **代码**: `src/**`, `python/**`, `tests/**`, `build.rs`
- **依赖**: `Cargo.toml`, `Cargo.lock`, `pyproject.toml`
- **文档**: `**.md`, `LICENSE`
- **CI**: `.github/workflows/**`, `Makefile`

## Changelog 工作流

### 自动流程（发版）

```bash
git tag v1.2.0
git push origin v1.2.0
# ✅ ci.yml 自动运行：测试 → 构建 → 发布到 PyPI → 创建 GitHub Release
```

### 手动更新 Release Notes

如需补充或更新已有版本的 Release Notes：

1. 修改 `CHANGELOG.md` 和 `CHANGELOG_ZH.md`
2. 在 GitHub Actions 页面手动触发 "Changelog Release Sync"
3. 输入版本号（如 `1.2.0`，不带 `v`）

### Changelog 格式检查

- **自动触发**: PR 或 push 修改 changelog 文件时
- **手动触发**: Actions 页面运行 "Changelog Check"，可选指定版本号
