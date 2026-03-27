# Help 改进设计

> **日期**: 2026-03-27
> **状态**: 已批准

## 背景

用户反馈：
1. `thru --help` 没有快速入门指南
2. 没有显示所有命令（`init` 还没实现）
3. 缺少详细参数说明

## 设计决策

### 命令结构

```
thru --help          → 简洁命令列表（现有，保持不变）
thru intro           → 新增：快速入门指南
thru <cmd> --help    → 详细参数（clap 自动生成）
```

### 行业惯例参考

| 命令 | 用途 | 案例 |
|------|------|------|
| `--help` | 简洁帮助 | git, npm, cargo |
| `--verbose` | 详细日志（不是帮助） | 通用 |
| `intro` / `tutorial` | 入门教程 | 作为子命令 |

### `thru intro` 输出内容

```
🚀 Thru 快速入门

1. 配置手机信息
   thru init --ip <手机IP> --user <用户名>

2. 生成 SSH 密钥（免密登录）
   thru config keygen
   thru config key-copy   # 按提示在手机执行命令

3. 发送文件到手机
   thru send 文件路径

4. 从手机拉取文件
   thru pull --list       # 查看手机上的文件
   thru pull 文件名       # 拉取文件

详细帮助: thru <命令> --help
```

## 实现要点

1. 在 `main.rs` 添加 `Intro` 命令
2. 创建 `src/commands/intro.rs`
3. 输出静态文本即可

## 相关问题修复

### 路径引号问题

所有涉及用户路径的地方需要加引号，防止空格问题：
- `send.rs` - SSH/SCP 命令
- `transfer.rs` - rsync 命令
- `pull.rs` - 路径拼接

### 传输逻辑改进

目录降级策略：
```
~/storage/downloads/Thru/ 不存在
    ↓
自动改发到 ~/storage/downloads/
    ↓
再失败才报错
```