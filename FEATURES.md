# VPN 应用新功能说明

## 功能概览

本应用现已支持以下核心功能：

### 1. 订阅管理功能

#### 订阅地址管理

- **添加订阅**：支持添加多个订阅源
- **订阅更新**：支持手动更新订阅，自动获取最新的节点列表
- **代理设置**：可选择是否通过代理更新订阅
- **自动更新间隔**：可设置自动更新的时间间隔（小时为单位）

#### 订阅功能特点

- 支持 Base64 编码的订阅内容
- 自动解析多种协议（Shadowsocks、ShadowsocksR、VMess、V2Ray、Trojan）
- 显示订阅状态、节点数量、最后更新时间
- 支持启用/禁用订阅

### 2. 连接测试功能

#### 单节点测试

- **延迟测试**：测试单个节点的延迟（通过 TCP 连接测试）
- **连通性测试**：检查节点是否可连接
- **测试参数**：可自定义测试次数和超时时间
- **实时反馈**：显示测试进度和结果

#### 批量测试

- **批量延迟测试**：一键测试所有节点的延迟
- **并发测试**：提高测试效率
- **结果展示**：以颜色标识延迟等级
  - 绿色：< 100ms（优秀）
  - 黄色：100-300ms（良好）
  - 红色：> 300ms（较慢）
  - 灰色：测试失败或未测试

### 3. HTTP 连接测试

- 支持通过 HTTP 请求测试连接（可选）
- 支持代理设置
- 可自定义测试 URL

## 使用方法

### 订阅管理

1. **打开订阅管理**
   - 点击主界面顶部的「订阅管理」按钮
2. **添加订阅**

   - 在订阅管理对话框中填写：
     - 订阅名称：例如"机场 A"
     - 订阅地址：订阅链接 URL
     - 更新间隔：多少小时更新一次
     - 是否使用代理更新：勾选后将通过当前连接的代理更新
   - 点击「添加订阅」按钮

3. **更新订阅**

   - 在订阅列表中找到要更新的订阅
   - 点击「更新」按钮
   - 等待更新完成，系统会显示获取到的节点数量
   - 更新后的节点会自动添加到配置列表

4. **删除订阅**
   - 在订阅列表中找到要删除的订阅
   - 点击「删除」按钮
   - 确认删除

### 连接测试

1. **单节点测试**

   - 在配置列表中找到要测试的节点
   - 点击该节点对应的「测试」按钮
   - 等待测试完成，结果会显示在「延迟」列

2. **批量测试**

   - 点击主界面顶部的「批量测试」按钮
   - 系统会自动测试所有节点
   - 测试完成后，每个节点的延迟会显示在列表中
   - 系统会显示可用节点数量

3. **查看测试结果**
   - 延迟列会显示各节点的测试结果：
     - 绿色标签：延迟 < 100ms（推荐使用）
     - 黄色标签：延迟 100-300ms（可以使用）
     - 红色标签：延迟 > 300ms（较慢）
     - 红色"失败"：无法连接
     - 灰色"未测试"：尚未测试

## 技术实现

### 后端（Rust）

#### 模块结构

- `subscription.rs`：订阅管理模块
  - 订阅的增删改查
  - 从订阅地址获取配置
  - Base64 解码和内容解析
- `network_test.rs`：网络测试模块
  - TCP 连接测试
  - 延迟测量
  - HTTP 连接测试
  - 批量测试支持

#### 核心 API

```rust
// 订阅管理
load_subscriptions_command()
add_subscription_command(name, url, use_proxy, update_interval)
update_subscription_command(id, ...)
delete_subscription_command(id)
refresh_subscription_command(id, proxy_url)

// 网络测试
test_connection_command(server, port, timeout_secs)
test_latency_command(server, port, count, timeout_secs)
test_http_connection_command(url, timeout_secs, use_proxy, proxy_url)
batch_test_connections_command(configs, timeout_secs)
batch_test_latencies_command(configs, count, timeout_secs)
```

### 前端（Vue + TypeScript）

#### 组件

- `SubscriptionDialog.vue`：订阅管理对话框
- `MainInterface.vue`：主界面（已集成测试功能）

#### 特性

- 使用 Naive UI 组件库
- 响应式设计
- 实时状态更新
- 用户友好的交互体验

## 依赖

### Rust 依赖

- `reqwest`：HTTP 客户端，用于获取订阅内容
- `tokio`：异步运行时
- `serde/serde_json`：JSON 序列化
- `base64`：Base64 编解码
- `chrono`：时间处理

### 前端依赖

- `Vue 3`
- `Naive UI`
- `@tauri-apps/api`

## 配置文件

### subscriptions.json

存储订阅列表的配置文件

```json
{
  "subscriptions": [
    {
      "id": "uuid",
      "name": "订阅名称",
      "url": "订阅地址",
      "enabled": true,
      "use_proxy": false,
      "update_interval": 24,
      "last_update": "2024-01-01T00:00:00Z",
      "config_count": 10,
      "created_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

## 注意事项

1. **订阅更新**

   - 首次添加订阅后需要手动点击「更新」按钮获取节点
   - 使用代理更新时需要确保当前已连接到代理
   - 订阅地址必须返回有效的节点配置

2. **连接测试**

   - 测试只检查 TCP 连接，不代表实际使用效果
   - 批量测试可能需要较长时间，请耐心等待
   - 测试超时时间默认为 5 秒，可根据网络情况调整

3. **性能考虑**
   - 批量测试是顺序执行的，节点较多时可能需要较长时间
   - 建议定期测试以获取最新的延迟信息
   - 测试结果会保存在内存中，刷新页面后需要重新测试

## 未来计划

- [ ] 自动定时更新订阅
- [ ] 导入/导出订阅列表
- [ ] 更智能的节点筛选（按延迟、地区等）
- [ ] 测试结果持久化存储
- [ ] 并发批量测试（提高测试速度）
- [ ] 支持更多订阅格式
- [ ] 节点分组管理
