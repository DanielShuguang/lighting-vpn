# VPN 客户端

一个基于 Tauri + Vue 3 + Naive UI 开发的功能完整的跨平台 VPN 客户端，支持订阅管理、连接测试和真实 VPN 连接。

## ✨ 核心功能

### 🔌 真实 VPN 连接

- ✅ **完整的连接功能**: 支持实际的 VPN 连接（通过 V2Ray/Xray 核心）
- ✅ **自动系统代理**: 连接时自动设置系统代理，断开时自动恢复
- ✅ **进程管理**: 智能管理 V2Ray/Xray 进程
- ✅ **跨平台支持**: Windows、macOS、Linux

### 📡 订阅管理

- ✅ **订阅地址管理**: 添加、更新、删除订阅源
- ✅ **自动更新节点**: 从订阅地址自动获取最新节点列表
- ✅ **代理更新**: 支持通过代理或直连更新订阅
- ✅ **订阅统计**: 显示节点数量、最后更新时间等

### 🧪 连接测试

- ✅ **延迟测试**: 测试单个或批量节点的延迟
- ✅ **连通性检测**: 检查节点是否可用
- ✅ **可视化结果**: 颜色标识延迟等级（绿色优秀、黄色良好、红色较慢）
- ✅ **批量测试**: 一键测试所有节点

### 🔧 支持的协议

- ✅ **VMess**: V2Ray 原生协议
- ✅ **Shadowsocks (SS)**: 轻量级加密代理
- ✅ **Trojan**: 模拟 HTTPS 流量
- ✅ **VLESS**: V2Ray 最新协议
- ⚠️ **ShadowsocksR (SSR)**: 仅支持导入，不支持连接

### 📥 配置导入

- **URL 导入**: 直接粘贴配置链接
- **二维码扫描**: 使用摄像头扫描二维码
- **剪贴板导入**: 从剪贴板自动读取配置

### 📤 配置导出

- **生成配置链接**: 导出为标准的配置 URL
- **生成二维码**: 生成配置二维码图片
- **复制到剪贴板**: 一键复制配置到剪贴板

### 🛠️ 配置管理

- **添加配置**: 通过 URL、二维码、剪贴板或订阅导入
- **编辑配置**: 修改现有配置参数
- **删除配置**: 安全删除不需要的配置
- **连接管理**: 真实的 VPN 连接/断开功能
- **状态显示**: 实时显示连接状态和延迟信息

### 💾 数据持久化

- 配置自动保存到本地文件
- 应用重启后自动加载配置
- 支持配置的增删改查

## 技术栈

### 前端

- **Vue 3** - 渐进式 JavaScript 框架
- **TypeScript** - 类型安全的 JavaScript
- **Naive UI** - Vue 3 组件库
- **UnoCSS** - 原子化 CSS 引擎
- **QR Scanner** - 二维码扫描库
- **QRCode** - 二维码生成库

### 后端

- **Rust** - 系统编程语言
- **Tauri** - 构建桌面应用的框架
- **Serde** - 序列化/反序列化
- **Tokio** - 异步运行时
- **Reqwest** - HTTP 客户端
- **V2Ray/Xray** - 代理核心程序（需单独安装）

## 开发环境

### 前置要求

- Node.js 18+
- pnpm
- Rust 1.70+
- Tauri CLI
- **V2Ray 或 Xray** (用于实际连接功能)

### 安装 V2Ray/Xray

**Windows**:

- 下载: [V2Ray](https://github.com/v2fly/v2ray-core/releases) 或 [Xray](https://github.com/XTLS/Xray-core/releases)
- 将 `v2ray.exe` 或 `xray.exe` 放在程序目录或系统 PATH 中

**Linux/macOS**:

```bash
# V2Ray
sudo bash -c "$(curl -L https://raw.githubusercontent.com/v2fly/fhs-install-v2ray/master/install-release.sh)"

# 或 Xray
sudo bash -c "$(curl -L https://github.com/XTLS/Xray-install/raw/main/install-release.sh)"
```

### 安装依赖

```bash
# 安装前端依赖
pnpm install

# 安装 Rust 依赖（自动执行）
```

### 开发模式

```bash
# 启动开发服务器
pnpm tauri dev
```

### 构建应用

```bash
# 构建生产版本
pnpm tauri build
```

## 📖 使用说明

详细使用说明请查看：

- **[连接功能指南](CONNECTION_GUIDE.md)** - VPN 连接详细说明
- **[功能特性文档](FEATURES.md)** - 订阅管理和测试功能

### 快速开始

#### 1. 安装 V2Ray/Xray

按照上面的说明安装 V2Ray 或 Xray 核心程序

#### 2. 添加节点配置

**方式 A：通过订阅**

1. 点击「订阅管理」按钮
2. 添加订阅源（输入名称和 URL）
3. 点击「更新」获取节点列表

**方式 B：手动导入**

1. **URL 导入**

   - 点击"导入配置"按钮
   - 选择"URL 导入"
   - 粘贴配置链接（如：`ss://...`, `vmess://...` 等）
   - 点击"解析并导入"

2. **二维码扫描**

   - 选择"二维码扫描"
   - 点击"开始扫描二维码"
   - 将二维码对准摄像头
   - 自动解析并导入配置

3. **剪贴板导入**
   - 选择"剪贴板导入"
   - 确保剪贴板中有配置链接
   - 点击"从剪贴板导入"

#### 3. 测试节点（推荐）

1. 点击「批量测试」按钮测试所有节点
2. 或点击单个节点的「测试」按钮
3. 查看延迟结果，选择低延迟节点

#### 4. 连接 VPN

1. 找到要连接的节点
2. 点击「连接」按钮
3. 等待连接成功（状态变为「已连接」）
4. 系统代理自动设置完成

#### 5. 断开连接

1. 点击已连接节点的「断开」按钮
2. 系统代理自动恢复

### 其他功能

**编辑配置**

- 点击「编辑」按钮修改节点信息

**导出配置**

- 点击「导出」按钮生成分享链接或二维码

**删除配置**

- 点击「删除」按钮移除节点

## 配置格式

### Shadowsocks

```
ss://base64(method:password)@server:port#name
```

### VMess

```
vmess://base64(json_config)
```

### Trojan

```
trojan://password@server:port?remarks=name
```

## 项目结构

```
vpn/
├── src/                           # 前端源码
│   ├── components/                # Vue 组件
│   │   ├── MainInterface.vue      # 主界面
│   │   ├── ImportDialog.vue       # 导入配置对话框
│   │   ├── ExportDialog.vue       # 导出配置对话框
│   │   ├── EditConfigDialog.vue   # 编辑配置对话框
│   │   ├── SubscriptionDialog.vue # 订阅管理对话框
│   │   └── ConnectionStatus.vue   # 连接状态组件
│   ├── App.vue
│   └── main.ts
├── src-tauri/                     # Rust 后端
│   ├── src/
│   │   ├── lib.rs                 # 主入口
│   │   ├── main.rs                # 程序入口
│   │   ├── commands.rs            # Tauri 命令
│   │   ├── vpn_config.rs          # VPN 配置解析
│   │   ├── vpn_manager.rs         # VPN 连接管理
│   │   ├── proxy_manager.rs       # 系统代理管理
│   │   ├── v2ray_config.rs        # V2Ray 配置生成
│   │   ├── subscription.rs        # 订阅管理
│   │   ├── network_test.rs        # 网络测试
│   │   └── storage.rs             # 数据存储
│   └── Cargo.toml
├── configs/                       # V2Ray 配置文件目录（自动生成）
├── vpn_configs.json               # 节点配置数据
├── subscriptions.json             # 订阅配置数据
├── package.json
├── README.md
├── FEATURES.md                    # 功能特性文档
└── CONNECTION_GUIDE.md            # 连接功能指南
```

## 🎯 功能完成度

- [x] 配置导入导出（URL、二维码、剪贴板）
- [x] 订阅管理（添加、更新、删除）
- [x] 连接测试（单个、批量、延迟测试）
- [x] **真实 VPN 连接**（VMess、SS、Trojan、VLESS）
- [x] **自动系统代理设置**
- [x] **进程管理**
- [x] 跨平台支持（Windows、macOS、Linux）
- [ ] 系统托盘支持
- [ ] 自动重连功能
- [ ] 配置分组管理
- [ ] 流量统计
- [ ] 日志记录功能
- [ ] 主题切换
- [ ] 多语言支持
- [ ] 规则管理（PAC/路由）

## ⚠️ 注意事项

1. **需要 V2Ray/Xray**: 连接功能依赖外部核心程序
2. **管理员权限**: Windows 上设置系统代理可能需要管理员权限
3. **防火墙**: 确保防火墙允许 V2Ray/Xray 访问网络
4. **SSR 不支持**: ShadowsocksR 协议不支持连接（只能导入）
5. **连接前测试**: 建议先测试节点延迟再连接

## 🐛 故障排除

### 提示"未找到 V2Ray/Xray"

- 确保已安装 V2Ray 或 Xray
- 将可执行文件放在程序目录或系统 PATH 中

### 连接失败

- 先测试节点延迟确认可用
- 检查防火墙设置
- 查看错误提示信息

### 浏览器无法使用代理

- Chrome/Edge 默认使用系统代理，应该可以
- Firefox 需要在设置中选择"使用系统代理设置"
- 或手动配置代理：`127.0.0.1:10809` (HTTP) 或 `127.0.0.1:10808` (SOCKS5)

详细故障排除请参考 [连接功能指南](CONNECTION_GUIDE.md)

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！
#   l i g h t i n g - v p n  
 