<template>
  <div class="h-full flex flex-col bg-gray-50">
    <!-- 头部工具栏 -->
    <div class="bg-white shadow-sm border-b border-gray-200 px-3 sm:px-4 md:px-6 py-2 sm:py-3">
      <div class="flex flex-col gap-2">
        <!-- 第一行：状态和代理模式 -->
        <div class="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 w-full">
          <ConnectionStatus
            :status="connectionStatus"
            :current-config="currentConfig"
            @disconnect="disconnectConfig"
            class="flex-shrink-0" />
          <ProxyModeSelector :disabled="connectionStatus === 'connecting'" class="flex-shrink-0" />
        </div>

        <!-- 第二行：操作按钮 -->
        <div class="flex flex-wrap items-center gap-1.5 md:gap-2">
          <!-- 自动连接按钮 -->
          <n-popover
            trigger="hover"
            :disabled="isAutoConnecting || connectionStatus === 'connected'">
            <template #trigger>
              <n-button
                v-if="!isAutoConnecting"
                type="success"
                @click="handleAutoConnect"
                :disabled="configs.length === 0 || connectionStatus === 'connected'"
                size="small">
                <template #icon>
                  <n-icon><FlashIcon /></n-icon>
                </template>
                <span class="hidden md:inline">自动连接</span>
                <span class="md:hidden">自动</span>
              </n-button>
              <n-button v-else type="warning" @click="cancelAutoConnect" size="small">
                <template #icon>
                  <n-icon><PauseIcon /></n-icon>
                </template>
                <span class="hidden md:inline">取消自动连接</span>
                <span class="md:hidden">取消</span>
              </n-button>
            </template>
            <div class="max-w-xs space-y-2">
              <div class="font-semibold text-sm">🚀 自动连接功能</div>
              <div class="text-xs text-gray-600 space-y-1">
                <p>自动依次测试所有配置，找到第一个可用的服务并连接</p>
                <p class="text-blue-600">• 优先测试之前成功的配置</p>
                <p class="text-blue-600">• 自动连接延迟 &lt; 3秒的服务</p>
                <p class="text-blue-600">• 连接成功后立即停止测试</p>
              </div>
            </div>
          </n-popover>

          <n-button type="primary" @click="showImportDialog = true" size="small">
            <template #icon>
              <n-icon><AddIcon /></n-icon>
            </template>
            <span class="hidden md:inline">导入配置</span>
            <span class="md:hidden">导入</span>
          </n-button>
          <n-button type="info" @click="showSubscriptionDialog = true" size="small">
            <template #icon>
              <n-icon><CloudDownloadIcon /></n-icon>
            </template>
            <span class="hidden md:inline">订阅管理</span>
            <span class="md:hidden">订阅</span>
          </n-button>
          <n-button
            v-if="!isBatchTesting"
            @click="handleBatchTest"
            :disabled="configs.length === 0"
            size="small">
            <template #icon>
              <n-icon><SpeedometerIcon /></n-icon>
            </template>
            <span class="hidden md:inline">批量测试</span>
            <span class="md:hidden">测速</span>
          </n-button>
          <n-button v-else type="warning" @click="cancelBatchTest" size="small">
            <template #icon>
              <n-icon><PauseIcon /></n-icon>
            </template>
            <span class="hidden md:inline">取消测试</span>
            <span class="md:hidden">取消</span>
          </n-button>
          <n-button @click="refreshConfigs" size="small">
            <template #icon>
              <n-icon><RefreshIcon /></n-icon>
            </template>
            <span class="hidden md:inline">刷新</span>
          </n-button>
          <n-button type="success" @click="showCoreDialog = true" size="small">
            <template #icon>
              <n-icon><CoreIcon /></n-icon>
            </template>
            <span class="hidden md:inline">核心管理</span>
            <span class="md:hidden">核心</span>
          </n-button>
          <n-popconfirm
            @positive-click="handleResetProxy"
            positive-text="确定重置"
            negative-text="取消">
            <template #trigger>
              <n-button type="warning" size="small">
                <template #icon>
                  <n-icon><RefreshIcon /></n-icon>
                </template>
                <span class="hidden md:inline">重置代理</span>
                <span class="md:hidden">重置</span>
              </n-button>
            </template>
            确定要重置系统代理设置吗？这将清除所有代理配置。
          </n-popconfirm>
        </div>
      </div>
    </div>

    <!-- 主要内容区域 -->
    <div class="flex-1 flex overflow-hidden">
      <!-- 配置列表 -->
      <div
        class="flex-1 bg-white m-2 sm:m-4 rounded-lg shadow-sm border border-gray-200 flex flex-col overflow-hidden">
        <div class="p-3 sm:p-4 border-b border-gray-200 space-y-3">
          <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-2">
            <h2 class="text-base sm:text-lg font-semibold text-gray-900">配置列表</h2>
            <div
              v-if="checkedRowKeys.length > 0"
              class="flex flex-wrap items-center gap-2 w-full sm:w-auto">
              <span class="text-xs sm:text-sm text-gray-600">
                已选择 {{ checkedRowKeys.length }} 项
              </span>
              <n-button size="small" type="error" @click="handleBatchDelete">
                <template #icon>
                  <n-icon><TrashIcon /></n-icon>
                </template>
                批量删除
              </n-button>
              <n-button size="small" @click="checkedRowKeys = []">取消选择</n-button>
            </div>
          </div>

          <!-- 过滤器 -->
          <ConfigFilter
            v-model:search-text="filterText"
            v-model:selected-subscription="filterSubscription"
            v-model:selected-protocol="filterProtocol"
            :configs="configs"
            :subscriptions="subscriptions" />
        </div>

        <div class="flex-1 overflow-auto p-2 sm:p-4">
          <n-data-table
            :columns="columns"
            :data="filteredConfigs"
            :loading="loading"
            :pagination="false"
            :bordered="false"
            :single-line="false"
            :scroll-x="1200"
            :row-key="(row: VpnConfig) => row.id"
            v-model:checked-row-keys="checkedRowKeys" />
        </div>
      </div>
    </div>

    <!-- 导入配置对话框 -->
    <ImportDialog v-model:show="showImportDialog" @imported="handleConfigImported" />

    <!-- 导出配置对话框 -->
    <ExportDialog v-model:show="showExportDialog" :config="selectedConfig" />

    <!-- 编辑配置对话框 -->
    <EditConfigDialog
      v-model:show="showEditDialog"
      :config="selectedConfig"
      @saved="handleConfigSaved" />

    <!-- 订阅管理对话框 -->
    <SubscriptionDialog
      v-model:show="showSubscriptionDialog"
      @subscription-updated="handleSubscriptionUpdated" />

    <!-- 核心管理对话框 -->
    <CoreDownloadDialog v-model:show="showCoreDialog" />
  </div>
</template>

<script lang="ts" setup>
import { ref, onMounted, computed } from 'vue'
import { NButton, NIcon, NDataTable, NPopover, NPopconfirm, useMessage } from 'naive-ui'
import {
  Add as AddIcon,
  Refresh as RefreshIcon,
  CloudDownload as CloudDownloadIcon,
  Speedometer as SpeedometerIcon,
  Pause as PauseIcon,
  Trash as TrashIcon,
  Cube as CoreIcon,
  Flash as FlashIcon
} from '@vicons/ionicons5'
import ImportDialog from './ImportDialog.vue'
import ExportDialog from './ExportDialog.vue'
import EditConfigDialog from './EditConfigDialog.vue'
import ConnectionStatus from './ConnectionStatus.vue'
import ProxyModeSelector from './ProxyModeSelector.vue'
import ConfigFilter from './ConfigFilter.vue'
import SubscriptionDialog from './SubscriptionDialog.vue'
import CoreDownloadDialog from './CoreDownloadDialog.vue'
import type { VpnConfig } from '../types/vpn'
import { createTableColumns, type TableActions } from '../config/table-columns'
import { useConfigManagement } from '../composables/useConfigManagement'
import { useVpnConnection } from '../composables/useVpnConnection'
import { useNetworkTest } from '../composables/useNetworkTest'
import { useProxyMode } from '../composables/useProxyMode'
import { invoke } from '@tauri-apps/api/core'

// 初始化消息提示
const message = useMessage()

// 对话框状态
const showImportDialog = ref(false)
const showExportDialog = ref(false)
const showEditDialog = ref(false)
const showSubscriptionDialog = ref(false)
const showCoreDialog = ref(false)
const selectedConfig = ref<VpnConfig | null>(null)

// 批量选择状态
const checkedRowKeys = ref<string[]>([])

// 过滤状态
const filterText = ref('')
const filterSubscription = ref<string | null>(null)
const filterProtocol = ref<string | null>(null)

// 订阅列表
const subscriptions = ref<Array<{ id: string; name: string }>>([])

// 过滤后的配置列表
const filteredConfigs = computed(() => {
  return configs.value.filter(config => {
    const matchName =
      !filterText.value ||
      config.name.toLowerCase().includes(filterText.value.toLowerCase()) ||
      (config.server && config.server.toLowerCase().includes(filterText.value.toLowerCase()))

    const matchSubscription =
      !filterSubscription.value ||
      config.subscription_id === filterSubscription.value ||
      (!config.subscription_id && filterSubscription.value === null)

    const matchProtocol = !filterProtocol.value || config.protocol === filterProtocol.value

    return matchName && matchSubscription && matchProtocol
  })
})

// 配置管理
const {
  configs,
  loading,
  loadConfigs,
  refreshConfigs,
  deleteConfig,
  batchDeleteConfigs,
  updateConfig,
  addConfig
} = useConfigManagement({ message })

// VPN 连接管理
const { connectionStatus, currentConfig, connectConfig, disconnectConfig, checkConnectionStatus } =
  useVpnConnection({ message, configs })

// 网络测试
const {
  testResults,
  testSingleConfig,
  handleBatchTest,
  cancelBatchTest,
  handleAutoConnect,
  cancelAutoConnect,
  isConfigTesting,
  isBatchTesting,
  isAutoConnecting
} = useNetworkTest({ message, configs })

// 编辑配置
const editConfig = (config: VpnConfig): void => {
  selectedConfig.value = config
  showEditDialog.value = true
}

// 导出配置
const exportConfig = (config: VpnConfig): void => {
  selectedConfig.value = config
  showExportDialog.value = true
}

// 处理配置导入
const handleConfigImported = (newConfig: VpnConfig): void => {
  addConfig(newConfig)
  message.success('配置导入成功')
}

// 处理配置保存
const handleConfigSaved = (updatedConfig: VpnConfig): void => {
  updateConfig(updatedConfig)
  message.success('配置更新成功')
}

// 处理订阅更新
const handleSubscriptionUpdated = async (): Promise<void> => {
  await loadConfigs()
  await loadSubscriptions()
}

// 处理批量删除
const handleBatchDelete = async (): Promise<void> => {
  if (checkedRowKeys.value.length === 0) {
    message.warning('请先选择要删除的配置')
    return
  }

  const count = checkedRowKeys.value.length
  const confirmed = confirm(`确定要删除选中的 ${count} 个配置吗？`)
  if (!confirmed) {
    return
  }

  await batchDeleteConfigs(checkedRowKeys.value)
  checkedRowKeys.value = []
  message.success(`成功删除 ${count} 个配置`)
}

// 重置代理设置
const handleResetProxy = async () => {
  try {
    const result = await invoke<string>('reset_proxy_command')
    message.success(result)
    // 刷新连接状态
    await checkConnectionStatus()
  } catch (error) {
    message.error(`重置代理失败: ${error}`)
  }
}

// 表格操作
const tableActions: TableActions = {
  connectConfig,
  disconnectConfig,
  editConfig,
  testSingleConfig,
  exportConfig,
  deleteConfig,
  isConfigTesting
}

// 表格列配置
const columns = computed(() =>
  createTableColumns(testResults, connectionStatus, currentConfig, tableActions, true)
)

// 组件挂载时初始化
// 代理模式管理
const { loadProxyMode } = useProxyMode()

// 加载订阅列表
const loadSubscriptions = async () => {
  try {
    const result = await invoke('load_subscriptions_command')
    const subs = (result as any).subscriptions || []
    subscriptions.value = subs.map((s: any) => ({
      id: s.id,
      name: s.name
    }))
  } catch (error) {
    console.error('加载订阅列表失败:', error)
  }
}

onMounted(async () => {
  await loadConfigs()
  await checkConnectionStatus()
  await loadSubscriptions()
  // 加载代理模式
  await loadProxyMode()
})
</script>
