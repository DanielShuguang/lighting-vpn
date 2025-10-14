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
import { ref, onMounted, h } from 'vue'
import { NButton, NIcon, NDataTable, NTag, NPopconfirm, useMessage } from 'naive-ui'
import {
  Add as AddIcon,
  Refresh as RefreshIcon,
  Play as PlayIcon,
  Pause as PauseIcon,
  Create as EditIcon,
  Trash as DeleteIcon,
  Share as ShareIcon,
  CloudDownload as CloudDownloadIcon,
  Speedometer as SpeedometerIcon,
  Checkmark as CheckmarkIcon
} from '@vicons/ionicons5'
import { invoke } from '@tauri-apps/api/core'
import ImportDialog from './ImportDialog.vue'
import ExportDialog from './ExportDialog.vue'
import EditConfigDialog from './EditConfigDialog.vue'
import ConnectionStatus from './ConnectionStatus.vue'
import SubscriptionDialog from './SubscriptionDialog.vue'

interface VpnConfig {
  id: string
  name: string
  protocol: string
  server: string
  port: number
  password?: string
  method?: string
  remarks?: string
  created_at: string
  updated_at: string
}

const message = useMessage()
const configs = ref<VpnConfig[]>([])
const loading = ref(false)
const showImportDialog = ref(false)
const showExportDialog = ref(false)
const showEditDialog = ref(false)
const showSubscriptionDialog = ref(false)
const selectedConfig = ref<VpnConfig | null>(null)
const connectionStatus = ref<'disconnected' | 'connecting' | 'connected' | 'error'>('disconnected')
const currentConfig = ref<VpnConfig | null>(null)
const testResults = ref<Map<string, { latency: number | null; success: boolean }>>(new Map())

// 表格列定义
const columns = [
  {
    title: '名称',
    key: 'name',
    width: 200,
    ellipsis: true
  },
  {
    title: '协议',
    key: 'protocol',
    width: 100,
    render: (row: VpnConfig) => {
      const protocolMap: Record<string, { type: string; text: string }> = {
        Shadowsocks: { type: 'success', text: 'SS' },
        ShadowsocksR: { type: 'info', text: 'SSR' },
        V2Ray: { type: 'warning', text: 'V2Ray' },
        Vmess: { type: 'error', text: 'VMess' },
        Trojan: { type: 'default', text: 'Trojan' }
      }
      const protocol = protocolMap[row.protocol] || { type: 'default', text: row.protocol }
      return h(NTag, { type: protocol.type as any }, { default: () => protocol.text })
    }
  },
  {
    title: '服务器',
    key: 'server',
    width: 150,
    ellipsis: true
  },
  {
    title: '端口',
    key: 'port',
    width: 80
  },
  {
    title: '延迟',
    key: 'latency',
    width: 100,
    render: (row: VpnConfig) => {
      const result = testResults.value.get(row.id)
      if (!result) {
        return h(NTag, { type: 'default', size: 'small' }, { default: () => '未测试' })
      }
      if (!result.success) {
        return h(NTag, { type: 'error', size: 'small' }, { default: () => '失败' })
      }
      const latency = result.latency
      if (latency === null) {
        return h(NTag, { type: 'default', size: 'small' }, { default: () => '-' })
      }
      const type = latency < 100 ? 'success' : latency < 300 ? 'warning' : 'error'
      return h(NTag, { type, size: 'small' }, { default: () => `${latency}ms` })
    }
  },
  {
    title: '状态',
    key: 'status',
    width: 100,
    render: (row: VpnConfig) => {
      const isCurrent = currentConfig.value?.id === row.id
      if (isCurrent) {
        const statusMap = {
          connected: { type: 'success', text: '已连接' },
          connecting: { type: 'warning', text: '连接中' },
          error: { type: 'error', text: '连接失败' },
          disconnected: { type: 'default', text: '未连接' }
        }
        const status = statusMap[connectionStatus.value]
        return h(NTag, { type: status.type as any }, { default: () => status.text })
      }
      return h(NTag, { type: 'default' }, { default: () => '未连接' })
    }
  },
  {
    title: '操作',
    key: 'actions',
    width: 200,
    render: (row: VpnConfig) => {
      const isCurrent = currentConfig.value?.id === row.id
      const isConnected = isCurrent && connectionStatus.value === 'connected'
      const isConnecting = isCurrent && connectionStatus.value === 'connecting'

      const buttons = []

      // 连接/断开按钮
      if (isConnected) {
        buttons.push(
          h(
            NButton,
            {
              size: 'small',
              type: 'error',
              onClick: () => disconnectConfig()
            },
            {
              icon: () => h(NIcon, null, { default: () => h(PauseIcon) }),
              default: () => '断开'
            }
          )
        )
      } else if (isConnecting) {
        buttons.push(
          h(
            NButton,
            {
              size: 'small',
              type: 'warning',
              loading: true
            },
            {
              default: () => '连接中'
            }
          )
        )
      } else {
        buttons.push(
          h(
            NButton,
            {
              size: 'small',
              type: 'primary',
              onClick: () => connectConfig(row)
            },
            {
              icon: () => h(NIcon, null, { default: () => h(PlayIcon) }),
              default: () => '连接'
            }
          )
        )
      }

      // 编辑按钮
      buttons.push(
        h(
          NButton,
          {
            size: 'small',
            onClick: () => editConfig(row)
          },
          {
            icon: () => h(NIcon, null, { default: () => h(EditIcon) }),
            default: () => '编辑'
          }
        )
      )

      // 测试按钮
      buttons.push(
        h(
          NButton,
          {
            size: 'small',
            type: 'info',
            onClick: () => testSingleConfig(row)
          },
          {
            icon: () => h(NIcon, null, { default: () => h(CheckmarkIcon) }),
            default: () => '测试'
          }
        )
      )

      // 导出按钮
      buttons.push(
        h(
          NButton,
          {
            size: 'small',
            onClick: () => exportConfig(row)
          },
          {
            icon: () => h(NIcon, null, { default: () => h(ShareIcon) }),
            default: () => '导出'
          }
        )
      )

      // 删除按钮
      buttons.push(
        h(
          NPopconfirm,
          {
            onPositiveClick: () => deleteConfig(row.id)
          },
          {
            trigger: () =>
              h(
                NButton,
                {
                  size: 'small',
                  type: 'error',
                  quaternary: true
                },
                {
                  icon: () => h(NIcon, null, { default: () => h(DeleteIcon) }),
                  default: () => '删除'
                }
              ),
            default: () => '确定要删除这个配置吗？'
          }
        )
      )

      return h('div', { class: 'flex space-x-2' }, buttons)
    }
  }
]

// 加载配置
const loadConfigs = async () => {
  try {
    loading.value = true
    const result = await invoke('load_configs_command')
    configs.value = (result as any).configs || []
  } catch (error) {
    message.error(`加载配置失败: ${error}`)
  } finally {
    loading.value = false
  }
}

// 刷新配置
const refreshConfigs = () => {
  loadConfigs()
}

// 连接配置
const connectConfig = async (config: VpnConfig) => {
  try {
    connectionStatus.value = 'connecting'
    currentConfig.value = config
    message.info(`正在连接 ${config.name}...`)

    // 调用真实的连接命令
    await invoke('connect_vpn_command', { config })

    connectionStatus.value = 'connected'
    message.success(`已连接到 ${config.name}`)
  } catch (error) {
    connectionStatus.value = 'error'
    currentConfig.value = null
    message.error(`连接失败: ${error}`)
  }
}

// 断开连接
const disconnectConfig = async () => {
  try {
    connectionStatus.value = 'connecting'
    message.info('正在断开连接...')

    // 调用真实的断开命令
    await invoke('disconnect_vpn_command')

    connectionStatus.value = 'disconnected'
    currentConfig.value = null
    message.success('已断开连接')
  } catch (error) {
    connectionStatus.value = 'error'
    message.error(`断开连接失败: ${error}`)
  }
}

// 编辑配置
const editConfig = (config: VpnConfig) => {
  selectedConfig.value = config
  showEditDialog.value = true
}

// 导出配置
const exportConfig = (config: VpnConfig) => {
  selectedConfig.value = config
  showExportDialog.value = true
}

// 删除配置
const deleteConfig = async (id: string) => {
  try {
    const currentConfigs = configs.value.filter(c => c.id !== id)
    await invoke('save_configs_command', { configs: { configs: currentConfigs } })
    configs.value = currentConfigs
    message.success('配置删除成功')
  } catch (error) {
    message.error(`删除配置失败: ${error}`)
  }
}

// 处理配置导入
const handleConfigImported = (newConfig: VpnConfig) => {
  configs.value.push(newConfig)
  message.success('配置导入成功')
}

// 处理配置保存
const handleConfigSaved = (updatedConfig: VpnConfig) => {
  const index = configs.value.findIndex(c => c.id === updatedConfig.id)
  if (index !== -1) {
    configs.value[index] = updatedConfig
  }
  message.success('配置更新成功')
}

// 处理订阅更新
const handleSubscriptionUpdated = async () => {
  await loadConfigs()
}

// 测试单个配置
const testSingleConfig = async (config: VpnConfig) => {
  try {
    message.info(`正在测试 ${config.name}...`)
    const result = (await invoke('test_latency_command', {
      server: config.server,
      port: config.port,
      count: 3,
      timeoutSecs: 5
    })) as any

    testResults.value.set(config.id, {
      latency: result.latency,
      success: result.success
    })

    if (result.success) {
      message.success(`${config.name} 延迟: ${result.latency}ms`)
    } else {
      message.error(`${config.name} 测试失败: ${result.error}`)
    }
  } catch (error) {
    message.error(`测试失败: ${error}`)
  }
}

// 批量测试
const handleBatchTest = async () => {
  if (configs.value.length === 0) {
    message.warning('没有可测试的配置')
    return
  }

  try {
    message.info(`正在测试 ${configs.value.length} 个配置...`)
    const configsData = configs.value.map(c => [c.id, c.name, c.server, c.port])

    const results = (await invoke('batch_test_latencies_command', {
      configs: configsData,
      count: 3,
      timeoutSecs: 5
    })) as any[]

    // 更新测试结果
    results.forEach((result: any) => {
      testResults.value.set(result.config_id, {
        latency: result.result.latency,
        success: result.result.success
      })
    })

    const successCount = results.filter(r => r.result.success).length
    message.success(`测试完成: ${successCount}/${configs.value.length} 个配置可用`)
  } catch (error) {
    message.error(`批量测试失败: ${error}`)
  }
}

// 检查连接状态
const checkConnectionStatus = async () => {
  try {
    const isConnected = (await invoke('is_connected_command')) as boolean
    if (isConnected) {
      const info = (await invoke('get_connection_status_command')) as any
      if (info) {
        connectionStatus.value = 'connected'
        // 尝试从配置列表中找到对应的配置
        const config = configs.value.find(c => c.id === info.config_id)
        if (config) {
          currentConfig.value = config
        }
      }
    }
  } catch (error) {
    console.error('检查连接状态失败:', error)
  }
}

onMounted(async () => {
  await loadConfigs()
  await checkConnectionStatus()
})
</script>
