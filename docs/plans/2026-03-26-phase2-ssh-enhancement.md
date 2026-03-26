# Phase 2: SSH 完善设计文档

> 创建日期: 2026-03-26
> 状态: 设计完成，待实现

---

## 一、目标

让 SSH 方案完整可用：
- 支持免密登录配置
- 支持远程拉取
- 优化传输体验（rsync + 进度显示）
- 全命令 --json 输出

---

## 二、新增命令

### 2.1 thru init

初始化向导，引导用户完成首次配置。

```bash
# 交互模式（默认）
thru init
  ? 手机 IP: 100.118.195.54
  ? SSH 端口 [8022]: 
  ? 用户名: u0_a406
  ? 是否配置免密登录? (Y/n)
  ✓ 配置完成！

# 参数模式（AI Agent 友好）
thru init --ip 100.118.195.54 --port 8022 --user u0_a406

# JSON 输出
thru init --ip 100.118.195.54 --json
```

**JSON 输出格式：**
```json
{
  "success": true,
  "config_path": "C:\\Users\\xxx\\.thru\\config.toml",
  "device": {
    "ip": "100.118.195.54",
    "port": 8022,
    "user": "u0_a406"
  }
}
```

### 2.2 thru config keygen

生成 SSH 密钥对。

```bash
thru config keygen
  ✓ 密钥已生成: ~/.thru/id_ed25519
  ✓ 公钥位置: ~/.thru/id_ed25519.pub
```

**密钥存储位置：** `~/.thru/id_ed25519`

**JSON 输出：**
```json
{
  "success": true,
  "private_key": "~/.thru/id_ed25519",
  "public_key": "~/.thru/id_ed25519.pub",
  "public_key_content": "ssh-ed25519 AAAA..."
}
```

### 2.3 thru config key-copy

部署公钥到手机。

```bash
thru config key-copy

# 方式 1：显示公钥 + 手动引导
  请在手机上执行以下命令：
  ─────────────────────────────────
  echo "ssh-ed25519 AAAA... thru@pc" >> ~/.ssh/authorized_keys
  ─────────────────────────────────

# 方式 2：自动推送（需要输入一次密码）
  ? 输入手机 SSH 密码: ****
  ✓ 公钥已部署，现在可以免密登录！
```

**JSON 输出：**
```json
{
  "success": true,
  "method": "manual",
  "public_key": "ssh-ed25519 AAAA...",
  "instructions": "echo \"ssh-ed25519 AAAA...\" >> ~/.ssh/authorized_keys"
}
```

### 2.4 thru pull

从手机拉取文件到电脑。

```bash
# 拉取单个文件
thru pull photo.jpg

# 列出手机 Thru 目录文件
thru pull --list

# 拉取全部
thru pull --all

# 指定保存目录
thru pull photo.jpg --output ~/Pictures/
```

**JSON 输出 (--list)：**
```json
{
  "files": [
    {"name": "photo.jpg", "size": 2411724, "modified": "2026-03-26T10:30:00Z"},
    {"name": "document.pdf", "size": 3072000, "modified": "2026-03-25T15:20:00Z"}
  ],
  "total": 2,
  "total_size": 5483724
}
```

**JSON 输出 (传输完成)：**
```json
{
  "success": true,
  "file": {
    "name": "photo.jpg",
    "size": 2411724,
    "local_path": "C:\\Users\\xxx\\Downloads\\Thru\\photo.jpg"
  }
}
```

### 2.5 thru send（改造）

使用 rsync 替代 scp，支持进度显示。

```bash
# 自动选择（rsync 优先）
thru send file.jpg

# 强制使用 rsync
thru send file.jpg --rsync

# 强制使用 scp（降级）
thru send file.jpg --scp
```

**进度显示：**
```
📤 Sending document.pdf...
[████████████████████░░░░] 80%  2.4/3.0 MB  1.2 MB/s
```

**JSON 输出：**
```json
{
  "success": true,
  "method": "rsync",
  "file": {
    "name": "document.pdf",
    "size": 3072000
  },
  "duration_ms": 2500,
  "speed_bps": 1228800
}
```

---

## 三、实现计划（3 周）

### Week 1: 配置体验

| 任务 | 文件 | 说明 |
|------|------|------|
| init 命令 | `src/commands/init.rs` | 交互式 + 参数模式 |
| keygen 子命令 | `src/commands/config.rs` | 生成 ed25519 密钥 |
| key-copy 子命令 | `src/commands/config.rs` | 显示公钥 + 引导 |
| SSH 密钥管理 | `src/core/ssh_key.rs` | 新增模块 |

### Week 2: 传输优化

| 任务 | 文件 | 说明 |
|------|------|------|
| rsync 传输 | `src/core/transfer.rs` | 替代 scp |
| 进度解析 | `src/core/transfer.rs` | 解析 rsync --progress |
| 进度显示 | `src/core/transfer.rs` | indicatif 进度条 |
| rsync 检测 | `src/core/transfer.rs` | 检测可用性，降级 scp |

### Week 3: 远程拉取

| 任务 | 文件 | 说明 |
|------|------|------|
| pull 命令 | `src/commands/pull.rs` | 新增 |
| pull --list | `src/commands/pull.rs` | 列出远程文件 |
| --json 支持 | 所有命令 | 统一 JSON 输出 |

---

## 四、新增依赖

```toml
[dependencies]
indicatif = "0.17"    # 进度条显示
inquire = "0.7"       # 交互式问答
```

---

## 五、文件结构

```
src/
├── commands/
│   ├── mod.rs
│   ├── init.rs       # 新增：初始化向导
│   ├── pull.rs       # 新增：远程拉取
│   ├── send.rs       # 改造：rsync + 进度
│   ├── config.rs     # 改造：新增 keygen/key-copy
│   └── ...
├── core/
│   ├── mod.rs
│   ├── transfer.rs   # 改造：rsync + 进度解析
│   ├── ssh_key.rs    # 新增：密钥管理
│   └── ...
└── main.rs
```

---

## 六、错误处理

### 6.1 init 命令

| 错误场景 | 处理方式 |
|----------|----------|
| IP 格式无效 | 提示重新输入 |
| 端口超出范围 | 提示重新输入 |
| 配置文件已存在 | 询问是否覆盖 |

### 6.2 keygen 命令

| 错误场景 | 处理方式 |
|----------|----------|
| 密钥已存在 | 询问是否覆盖 |
| 权限不足 | 提示检查目录权限 |

### 6.3 key-copy 命令

| 错误场景 | 处理方式 |
|----------|----------|
| SSH 连接失败 | 显示公钥，引导手动配置 |
| 密码错误 | 允许重试 3 次 |

### 6.4 pull 命令

| 错误场景 | 处理方式 |
|----------|----------|
| 远程文件不存在 | 提示文件列表 |
| 本地空间不足 | 提示错误并退出 |
| 权限不足 | 提示错误并退出 |

### 6.5 send 命令（rsync 检测）

| 错误场景 | 处理方式 |
|----------|----------|
| 本地无 rsync | 自动降级到 scp |
| 远程无 rsync | 自动降级到 scp |
| 传输中断 | 显示已传输大小，提示重试 |

---

## 七、兼容性设计

### 7.1 Windows rsync 问题

Windows 原生没有 rsync，解决方案：

```
检测流程：
1. 检查 rsync 是否可用
2. 可用 → 使用 rsync
3. 不可用 → 降级到 scp（Windows 自带）
```

### 7.2 手机端 rsync

Termux 可安装 rsync：

```bash
pkg install rsync
```

检测流程：
```
1. 先尝试 rsync 传输
2. 失败 → 检测是否缺少 rsync
3. 缺少 → 提示安装命令，降级到 scp
```

### 7.3 降级策略

```
rsync (优先) → 有进度、可续传、压缩
    ↓ 不可用
scp (兜底) → 基础传输，无进度
```

---

## 八、安全设计

| 项目 | 设计 |
|------|------|
| 密钥存储 | `~/.thru/id_ed25519`，权限 600 |
| pull 路径限制 | 默认只允许 `~/storage/downloads/Thru/` |
| JSON 输出 | 不包含私钥内容 |
| 密码输入 | 使用 inquire 隐藏显示 |

---

## 九、相关文档

- [产品路线图](./2026-03-26-thru-roadmap.md)
- [总体设计](../thru-design.md)
- [Phase 1 实现](./2026-03-25-thru-cli-implementation.md)

---

## 十、状态

- 设计：✅ 完成
- 实现：📅 待开始