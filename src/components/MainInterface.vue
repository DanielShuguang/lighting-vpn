<template>
  <div class="h-full flex flex-col bg-gray-50">
    <!-- 头部工具栏 -->
    <div class="bg-white shadow-sm border-b border-gray-200 px-6 py-4">
      <div class="flex items-center justify-between">
        <h1 class="text-2xl font-bold text-gray-900">VPN 客户端</h1>
        <div class="flex items-center space-x-3">
          <ConnectionStatus :status="connectionStatus" :current-config="currentConfig" />
          <n-button type="primary" @click="showImportDialog = true">
            <template #icon>
              <n-icon><AddIcon /></n-icon>
            </template>
            导入配置
          </n-button>
          <n-button type="info" @click="showSubscriptionDialog = true">
            <template #icon>
              <n-icon><CloudDownloadIcon /></n-icon>
            </template>
            订阅管理
          </n-button>
          <n-button @click="handleBatchTest">
            <template #icon>
              <n-icon><SpeedometerIcon /></n-icon>
            </template>
            批量测试
          </n-button>
          <n-button @click="refreshConfigs">
            <template #icon>
              <n-icon><RefreshIcon /></n-icon>
            </template>
            刷新
          </n-button>
        </div>
      </div>
    </div>

    <!-- 主要内容区域 -->
    <div class="flex-1 flex overflow-hidden">
      <!-- 配置列表 -->
      <div class="flex-1 bg-white m-4 rounded-lg shadow-sm border border-gray-200">
        <div class="p-4 border-b border-gray-200">
          <h2 class="text-lg font-semibold text-gray-900">配置列表</h2>
        </div>

        <div class="p-4">
          <n-data-table
            :columns="columns"
            :data="configs"
            :loading="loading"
            :pagination="false"
            :bordered="false"
            :single-line="false" />
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
  </div>
</template>

<script lang="ts" setup>
import { ref, onMounted, computed } from 'vue'
import { NButton, NIcon, NDataTable, useMessage } from 'naive-ui'
import {
  Add as AddIcon,
  Refresh as RefreshIcon,
  CloudDownload as CloudDownloadIcon,
  Speedometer as SpeedometerIcon
} from '@vicons/ionicons5'
import ImportDialog from './ImportDialog.vue'
import ExportDialog from './ExportDialog.vue'
import EditConfigDialog from './EditConfigDialog.vue'
import ConnectionStatus from './ConnectionStatus.vue'
import SubscriptionDialog from './SubscriptionDialog.vue'
import type { VpnConfig } from '../types/vpn'
import { createTableColumns, type TableActions } from '../config/table-columns'
import { useConfigManagement } from '../composables/useConfigManagement'
import { useVpnConnection } from '../composables/useVpnConnection'
import { useNetworkTest } from '../composables/useNetworkTest'

// 初始化消息提示
const message = useMessage()

// 对话框状态
const showImportDialog = ref(false)
const showExportDialog = ref(false)
const showEditDialog = ref(false)
const showSubscriptionDialog = ref(false)
const selectedConfig = ref<VpnConfig | null>(null)

// 配置管理
const { configs, loading, loadConfigs, refreshConfigs, deleteConfig, updateConfig, addConfig } =
  useConfigManagement({ message })

// VPN 连接管理
const { connectionStatus, currentConfig, connectConfig, disconnectConfig, checkConnectionStatus } =
  useVpnConnection({ message, configs })

// 网络测试
const { testResults, testSingleConfig, handleBatchTest } = useNetworkTest({ message, configs })

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
}

// 表格操作
const tableActions: TableActions = {
  connectConfig,
  disconnectConfig,
  editConfig,
  testSingleConfig,
  exportConfig,
  deleteConfig
}

// 表格列配置
const columns = computed(() =>
  createTableColumns(testResults, connectionStatus, currentConfig, tableActions)
)

// 组件挂载时初始化
onMounted(async () => {
  await loadConfigs()
  await checkConnectionStatus()
})
</script>
