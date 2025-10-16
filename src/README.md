# 项目代码结构说明

本项目采用模块化架构，将代码按职责拆分为不同的模块，以提高可维护性和可读性。

## 📁 目录结构

```
src/
├── types/              # 类型定义
│   └── vpn.ts         # VPN 相关的所有类型定义
├── config/            # 配置文件
│   └── table-columns.tsx  # 表格列配置和渲染逻辑
├── composables/       # 组合式函数（Composables）
│   ├── useConfigManagement.ts  # 配置管理逻辑
│   ├── useVpnConnection.ts     # VPN 连接管理逻辑
│   └── useNetworkTest.ts       # 网络测试逻辑
└── components/        # Vue 组件
    ├── MainInterface.vue       # 主界面组件
    ├── ConnectionStatus.vue    # 连接状态组件
    ├── ImportDialog.vue        # 导入配置对话框
    ├── ExportDialog.vue        # 导出配置对话框
    ├── EditConfigDialog.vue    # 编辑配置对话框
    └── SubscriptionDialog.vue  # 订阅管理对话框
```

## 🎯 模块说明

### 1. 类型定义 (`types/vpn.ts`)

集中管理所有 VPN 相关的类型定义，包括：

- `VpnConfig`: VPN 配置接口
- `VpnProtocol`: VPN 协议类型
- `ConnectionStatus`: 连接状态类型
- `LatencyTestResult`: 延迟测试结果
- `BatchTestResult`: 批量测试结果
- `ConnectionInfo`: 连接信息
- 等等...

**使用示例**：

```typescript
import type { VpnConfig, ConnectionStatus } from '../types/vpn'

const config: VpnConfig = {
  id: '1',
  name: 'My VPN',
  protocol: 'Shadowsocks'
  // ...
}
```

### 2. 表格列配置 (`config/table-columns.tsx`)

定义和管理表格列的配置和渲染逻辑。

**导出内容**：

- `protocolTagMap`: 协议标签映射
- `statusTagMap`: 状态标签映射
- `createTableColumns()`: 创建表格列配置的工厂函数
- `TableActions`: 表格操作接口

**使用示例**：

```typescript
import { createTableColumns, type TableActions } from '../config/table-columns'

const tableActions: TableActions = {
  connectConfig,
  disconnectConfig,
  editConfig,
  testSingleConfig,
  exportConfig,
  deleteConfig
}

const columns = createTableColumns(testResults, connectionStatus, currentConfig, tableActions)
```

### 3. Composables（组合式函数）

#### 3.1 配置管理 (`useConfigManagement.ts`)

处理配置的加载、保存、删除等操作。

**导出内容**：

- `configs`: 配置列表（响应式）
- `loading`: 加载状态（响应式）
- `loadConfigs()`: 加载配置列表
- `refreshConfigs()`: 刷新配置列表
- `deleteConfig()`: 删除配置
- `updateConfig()`: 更新配置
- `addConfig()`: 添加配置

**使用示例**：

```typescript
import { useConfigManagement } from '../composables/useConfigManagement'

const { configs, loading, loadConfigs, refreshConfigs, deleteConfig, updateConfig, addConfig } =
  useConfigManagement({ message })
```

#### 3.2 VPN 连接管理 (`useVpnConnection.ts`)

处理 VPN 的连接、断开等操作。

**导出内容**：

- `connectionStatus`: 连接状态（响应式）
- `currentConfig`: 当前连接的配置（响应式）
- `connectConfig()`: 连接到指定配置
- `disconnectConfig()`: 断开当前连接
- `checkConnectionStatus()`: 检查连接状态

**使用示例**：

```typescript
import { useVpnConnection } from '../composables/useVpnConnection'

const { connectionStatus, currentConfig, connectConfig, disconnectConfig, checkConnectionStatus } =
  useVpnConnection({ message, configs })
```

#### 3.3 网络测试 (`useNetworkTest.ts`)

处理单个和批量的网络延迟测试。

**导出内容**：

- `testResults`: 测试结果集合（响应式）
- `testSingleConfig()`: 测试单个配置的延迟
- `handleBatchTest()`: 批量测试所有配置
- `clearTestResults()`: 清除测试结果

**使用示例**：

```typescript
import { useNetworkTest } from '../composables/useNetworkTest'

const { testResults, testSingleConfig, handleBatchTest, clearTestResults } = useNetworkTest({
  message,
  configs
})
```

## 🔧 最佳实践

### 1. 类型安全

所有的类型定义都集中在 `types/vpn.ts` 中，确保类型一致性：

```typescript
// ✅ 推荐：使用统一的类型定义
import type { VpnConfig } from '../types/vpn'

// ❌ 避免：在组件中重复定义类型
interface VpnConfig { ... }
```

### 2. 逻辑复用

将业务逻辑抽取到 Composables 中，提高代码复用性：

```typescript
// ✅ 推荐：使用 Composables
const { loadConfigs, deleteConfig } = useConfigManagement({ message })

// ❌ 避免：在组件中直接写业务逻辑
const loadConfigs = async () => {
  // 大量逻辑代码...
}
```

### 3. 关注点分离

将 UI 渲染逻辑（如表格列配置）与业务逻辑分离：

```typescript
// ✅ 推荐：表格列配置独立
import { createTableColumns } from '../config/table-columns'

// ❌ 避免：在组件中定义大量渲染逻辑
const columns = [
  {
    /* 100+ 行的渲染逻辑 */
  }
]
```

### 4. Props 和 Emits 类型

始终为组件的 Props 和 Emits 提供明确的类型：

```typescript
const props = defineProps<{
  show: boolean
  config: VpnConfig | null
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
  saved: [config: VpnConfig]
}>()
```

## 🚀 扩展建议

### 添加新功能

1. **添加新的类型**：在 `types/vpn.ts` 中定义
2. **添加新的业务逻辑**：创建新的 Composable
3. **添加新的 UI 组件**：在 `components/` 目录下创建
4. **添加新的配置**：在 `config/` 目录下创建

### 重构建议

如果某个 Composable 变得过于复杂（超过 200 行），考虑进一步拆分：

```typescript
// 拆分前
useVpnConnection.ts (300+ 行)

// 拆分后
useVpnConnection.ts (主要逻辑)
└── utils/
    ├── connectionHandler.ts (连接处理)
    └── statusChecker.ts (状态检查)
```

## 📝 维护指南

### 修改现有功能

1. **修改类型定义**：更新 `types/vpn.ts`
2. **修改业务逻辑**：找到对应的 Composable 并修改
3. **修改 UI**：找到对应的组件并修改

### 代码审查要点

- ✅ 类型定义是否完整和准确
- ✅ 是否正确使用了 TypeScript 类型系统
- ✅ 业务逻辑是否在 Composables 中
- ✅ 组件是否只关注 UI 渲染
- ✅ 是否有重复的代码可以抽取

## 🎨 代码风格

- 使用 TypeScript 严格模式
- 使用 ESLint 和 Prettier 保持代码风格一致
- 为所有公开的函数和接口添加 JSDoc 注释
- 使用有意义的变量和函数名称

## 🔗 相关资源

- [Vue 3 组合式 API 文档](https://cn.vuejs.org/guide/extras/composition-api-faq.html)
- [TypeScript 类型体操](https://github.com/type-challenges/type-challenges)
- [Naive UI 组件库](https://www.naiveui.com/)
