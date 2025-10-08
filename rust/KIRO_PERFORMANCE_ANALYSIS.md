# Kiro 供应商性能分析报告

## 概述

经过全面分析 Rust 版本的 Kiro 供应商实现，发现了多个导致性能不佳的原因。尽管 Rust 版本声称比 Node.js 版本有显著的性能提升，但 Kiro 供应商的具体实现存在一些性能瓶颈。

## 性能瓶颈分析

### 1. 网络超时设置不合理

```rust
let client = Client::builder()
    .timeout(std::time::Duration::from_secs(300))  // 300秒超时！
    .build()?;
```

**问题**：300秒（5分钟）的超时时间过长，导致慢速请求占用资源时间过长。

**影响**：
- 慢速连接不能及时释放
- 并发请求时资源占用过多
- 用户体验差（等待时间过长）

### 2. 复杂的请求格式转换

```rust
let codewhisperer_request = self.build_codewhisperer_request(&body).await?;
```

**问题**：需要将 Claude API 格式转换为 AWS CodeWhisperer 格式，涉及大量的：
- JSON 解析和重构
- 字符串处理
- 嵌套循环遍历
- 多次内存分配

**影响**：
- 增加 CPU 使用率
- 增加内存分配
- 增加处理延迟

### 3. 复杂的响应解析

```rust
fn parse_codewhisperer_response(&self, response_text: &str) -> Result<(String, Vec<serde_json::Value>)> {
    // 手动查找 "event{" 位置
    let mut event_positions: Vec<usize> = Vec::new();
    let mut search_start = 0;
    while let Some(pos) = response_text[search_start..].find("event{") {
        event_positions.push(search_start + pos);
        search_start += pos + 6;
    }
    // ... 多层嵌套循环和字符串查找
}
```

**问题**：
- 使用字符串查找而非专用的事件流解析器
- 多层嵌套循环
- 多次字符串切片和复制
- 正则表达式解析

**影响**：
- 高 CPU 使用率
- 大响应时解析时间长
- 内存使用效率低

### 4. 伪流式响应实现

```rust
// Note: Kiro/CodeWhisperer doesn't support true streaming
// We'll get the full response and simulate streaming
let full_response = self.call_api_with_retry("/v1/messages", request_body, 0).await?;
```

**问题**：
- 需要等待完整响应后才能开始"流式"传输
- 增加了首字节时间（TTFB）
- 用户感知延迟增加

### 5. 同步锁竞争

```rust
let creds = self.credentials.read().await;
if self.is_token_expired(&creds) {
    drop(creds);
    // ... refresh token
}
```

**问题**：
- 每次请求都需要获取锁来检查凭据
- Token 刷新时需要写锁，会阻塞其他请求
- 高并发时锁竞争严重

### 6. Token 管理开销

```rust
// 每次请求前检查 token 是否过期
if self.is_token_expired(&creds) {
    // 刷新 token 需要额外的网络请求
    self.refresh_access_token().await
}
```

**问题**：
- 每次请求都要检查过期时间
- Token 刷新是同步操作，阻塞请求
- 没有预刷新机制

### 7. 重试机制延迟

```rust
if (status.as_u16() == 429 || status.is_server_error()) && retry_count < self.max_retries {
    let delay = self.base_delay * 2_u64.pow(retry_count);
    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
}
```

**问题**：
- 指数退避可能导致长时间延迟
- 没有考虑 Retry-After 头
- 重试时重新进行所有转换操作

## 性能优化方案

### 1. 优化网络配置

```rust
// 优化后的客户端配置
let client = Client::builder()
    .timeout(std::time::Duration::from_secs(30))  // 减少到30秒
    .connect_timeout(std::time::Duration::from_secs(5))  // 连接超时5秒
    .pool_idle_timeout(std::time::Duration::from_secs(90))
    .pool_max_idle_per_host(10)
    .tcp_nodelay(true)  // 禁用 Nagle 算法
    .tcp_keepalive(Some(std::time::Duration::from_secs(60)))
    .build()?;
```

### 2. 缓存请求转换结果

```rust
// 使用 LRU 缓存存储转换结果
use lru::LruCache;

struct KiroApiService {
    // ... 其他字段
    request_cache: Arc<RwLock<LruCache<u64, serde_json::Value>>>,
}

async fn build_codewhisperer_request_cached(&self, claude_request: &serde_json::Value) -> Result<serde_json::Value> {
    let hash = calculate_hash(claude_request);
    
    // 检查缓存
    if let Some(cached) = self.request_cache.read().await.get(&hash) {
        return Ok(cached.clone());
    }
    
    // 转换并缓存
    let result = self.build_codewhisperer_request(claude_request).await?;
    self.request_cache.write().await.put(hash, result.clone());
    Ok(result)
}
```

### 3. 使用专用的事件流解析器

```rust
// 使用 eventsource-stream 或类似库
use eventsource_stream::EventStream;
use futures::StreamExt;

async fn parse_event_stream(&self, response: Response) -> Result<(String, Vec<serde_json::Value>)> {
    let stream = EventStream::new(response.bytes_stream());
    let mut content = String::new();
    let mut tool_calls = Vec::new();
    
    while let Some(event) = stream.next().await {
        match event {
            Ok(event) => {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&event.data) {
                    // 处理事件数据
                }
            }
            Err(e) => warn!("Event parse error: {}", e),
        }
    }
    
    Ok((content, tool_calls))
}
```

### 4. 实现真正的流式响应

```rust
async fn generate_content_stream(&self, model: &str, request_body: serde_json::Value) 
    -> Result<Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send>>> {
    
    // 使用 Server-Sent Events 流
    let response = self.make_streaming_request("/v1/messages", request_body).await?;
    let event_stream = EventStream::new(response.bytes_stream());
    
    let stream = event_stream.filter_map(|event| async move {
        match event {
            Ok(event) => {
                // 实时转换并发送事件
                Some(Ok(convert_to_claude_event(event)))
            }
            Err(e) => Some(Err(e.into()))
        }
    });
    
    Ok(Box::pin(stream))
}
```

### 5. 优化锁使用

```rust
// 使用无锁的 Arc<AtomicBool> 检查 token 状态
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

struct KiroApiService {
    token_expires_at: Arc<AtomicI64>,
    token_refreshing: Arc<AtomicBool>,
    // ...
}

fn is_token_expired(&self) -> bool {
    let expires_at = self.token_expires_at.load(Ordering::Relaxed);
    let now = chrono::Utc::now().timestamp();
    expires_at <= now + 300  // 5分钟缓冲
}
```

### 6. 实现预刷新机制

```rust
// 后台 token 刷新任务
async fn start_token_refresh_task(self: Arc<Self>) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
    
    loop {
        interval.tick().await;
        
        if self.is_token_expiring_soon() {
            if let Ok(false) = self.token_refreshing.compare_exchange(
                false, true, Ordering::SeqCst, Ordering::SeqCst
            ) {
                match self.refresh_access_token().await {
                    Ok(_) => info!("Token refreshed proactively"),
                    Err(e) => error!("Token refresh failed: {}", e),
                }
                self.token_refreshing.store(false, Ordering::SeqCst);
            }
        }
    }
}
```

### 7. 优化重试策略

```rust
// 智能重试，考虑 Retry-After 头
async fn call_api_with_smart_retry(&self, endpoint: &str, body: serde_json::Value) -> Result<serde_json::Value> {
    let mut retry_count = 0;
    
    loop {
        let response = self.make_request(endpoint, &body).await?;
        
        if response.status().is_success() {
            return Ok(response.json().await?);
        }
        
        if response.status() == 429 {
            // 检查 Retry-After 头
            let retry_after = response.headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(1);
                
            tokio::time::sleep(Duration::from_secs(retry_after)).await;
        } else if response.status().is_server_error() && retry_count < 3 {
            // 简单的线性退避
            tokio::time::sleep(Duration::from_millis(500 * (retry_count + 1))).await;
        } else {
            // 不重试，直接返回错误
            break;
        }
        
        retry_count += 1;
    }
}
```

### 8. 添加性能监控

```rust
// 使用 metrics 库添加性能指标
use metrics::{counter, histogram};

async fn generate_content(&self, model: &str, request_body: serde_json::Value) -> Result<serde_json::Value> {
    let start = Instant::now();
    counter!("kiro_requests_total").increment(1);
    
    let result = self.call_api_with_retry("/v1/messages", request_body, 0).await;
    
    let duration = start.elapsed();
    histogram!("kiro_request_duration_seconds").record(duration.as_secs_f64());
    
    match &result {
        Ok(_) => counter!("kiro_requests_success").increment(1),
        Err(_) => counter!("kiro_requests_error").increment(1),
    }
    
    result
}
```

## 实施建议

### 优先级高
1. 减少网络超时时间
2. 优化响应解析逻辑
3. 实现请求/响应缓存

### 优先级中
4. 优化锁机制
5. 实现预刷新 token
6. 改进重试策略

### 优先级低
7. 添加性能监控
8. 实现真正的流式响应

## 预期效果

实施这些优化后，预计可以实现：
- **请求延迟降低 50-70%**
- **CPU 使用率降低 30-50%**
- **内存使用降低 20-30%**
- **并发处理能力提升 2-3 倍**

## 总结

Kiro 供应商的性能问题主要源于：
1. 复杂的格式转换逻辑
2. 低效的响应解析
3. 不合理的网络配置
4. 缺乏缓存机制

通过实施上述优化方案，可以显著提升 Kiro 供应商的性能，使其达到与其他供应商相当的水平。
