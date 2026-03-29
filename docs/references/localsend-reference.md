# LocalSend 技术参考

> 来源：localsend-main/ 项目分析
> 用途：优化 Thru 设计参考

---

## 1. 核心架构洞察

### HTTP 上传进度（关键发现）

**错误设计**（我们之前）：
```
HTTP 层读取文件 → 更新进度 → 发送
```

**正确设计**（LocalSend）：
```
应用层读取文件 → 更新进度 → 发送到 channel
                          ↓
HTTP 层只接收 stream → 发送到网络
```

**参考代码** (core/src/http/client/v2.rs:227-230):
```rust
pub async fn upload(binary: mpsc::Receiver<Vec<u8>>) {
    let stream = ReceiverStream::new(binary).map(Ok::<Vec<u8>, anyhow::Error>);
    let body = reqwest::Body::wrap_stream(stream);
    client.post(url).body(body).send().await?
}
```

**依赖**：
- `tokio_stream = "0.7"`
- `futures = "0.3"`

---

## 2. 设备发现三层机制

```
Multicast Discovery   → UDP 224.0.0.167:53317
HTTP Scan Discovery   → 50 并发扫描子网
HTTP Target Discovery → 单 IP 探测
```

**TaskRunner 并发控制**：
```dart
class TaskRunner<T> {
    final int concurrency;  // 50 个并发
    final Queue<FutureFunction<T>> _queue;
}
```

---

## 3. 协议版本

| V2 | V3 |
|----|----|
| POST /register | POST /nonce (新增) |
| POST /prepare-upload | POST /register |
| POST /upload | POST /prepare-upload |
| POST /cancel | POST /upload |
| | POST /cancel |

**V3 新增 nonce 交换**：防止重放攻击

---

## 4. 可借鉴设计

| 设计 | 用途 | Thru 应用 |
|------|------|----------|
| `mpsc::Receiver<Vec<u8>>` | 流式上传 | 修复进度条问题 |
| `LruCache` | nonce 缓存 | 会话管理 |
| `TaskRunner` | 并发控制 | 多文件并发上传 |
| IPv4/IPv6 双栈 | 兼容性 | 添加 IPv6 支持 |
| Token + IP 白名单 | 安全 | HTTP 服务认证 |

---

## 5. 关键文件索引

| 文件 | 内容 |
|------|------|
| `core/src/http/client/v2.rs` | HTTP 客户端实现 |
| `core/src/http/server/mod.rs` | HTTP 服务端 |
| `core/src/crypto/nonce.rs` | Nonce 生成验证 |
| `app/lib/provider/network/send_provider.dart` | 发送逻辑 + 进度更新 |
| `app/lib/provider/progress_provider.dart` | 进度管理 |