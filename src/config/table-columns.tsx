/**
 * 表格列配置
 */
import { h, type Ref } from 'vue'
import type { DataTableColumns } from 'naive-ui'
import { NButton, NIcon, NTag, NDropdown } from 'naive-ui'
import type { DropdownOption } from 'naive-ui'
import {
  Play as PlayIcon,
  Pause as PauseIcon,
  Create as EditIcon,
  Trash as DeleteIcon,
  Share as ShareIcon,
  Checkmark as CheckmarkIcon,
  EllipsisHorizontal as MoreIcon
} from '@vicons/ionicons5'
import type {
  VpnConfig,
  ConnectionStatus,
  LatencyTestResult,
  ProtocolTagMap,
  StatusTagMap
} from '../types/vpn'

/**
 * 协议标签映射
 */
export const protocolTagMap: ProtocolTagMap = {
  Shadowsocks: { type: 'success', text: 'SS' },
  ShadowsocksR: { type: 'info', text: 'SSR' },
  V2Ray: { type: 'warning', text: 'V2Ray' },
  Vmess: { type: 'error', text: 'VMess' },
  Trojan: { type: 'default', text: 'Trojan' }
}

/**
 * 状态标签映射
 */
export const statusTagMap: StatusTagMap = {
  connected: { type: 'success', text: '已连接' },
  connecting: { type: 'warning', text: '连接中' },
  error: { type: 'error', text: '连接失败' },
  disconnected: { type: 'default', text: '未连接' }
}

/**
 * 表格操作按钮配置
 */
export interface TableActions {
  connectConfig: (config: VpnConfig) => void
  disconnectConfig: () => void
  editConfig: (config: VpnConfig) => void
  testSingleConfig: (config: VpnConfig) => void
  exportConfig: (config: VpnConfig) => void
  deleteConfig: (id: string) => void
  isConfigTesting: (configId: string) => boolean
}

/**
 * 创建表格列配置
 */
export function createTableColumns(
  testResults: Ref<Map<string, LatencyTestResult>>,
  connectionStatus: Ref<ConnectionStatus>,
  currentConfig: Ref<VpnConfig | null>,
  actions: TableActions,
  enableSelection: boolean = false
): DataTableColumns<VpnConfig> {
  const columns: DataTableColumns<VpnConfig> = []

  // 添加复选框列
  if (enableSelection) {
    columns.push({
      type: 'selection'
    })
  }

  columns.push(
    {
      title: '名称',
      key: 'name',
      minWidth: 150,
      maxWidth: 300,
      ellipsis: {
        tooltip: true
      }
    },
    {
      title: '协议',
      key: 'protocol',
      minWidth: 80,
      render: (row: VpnConfig) => {
        const protocol = protocolTagMap[row.protocol] || { type: 'default', text: row.protocol }
        return h(NTag, { type: protocol.type }, { default: () => protocol.text })
      }
    },
    {
      title: '服务器',
      key: 'server',
      minWidth: 120,
      ellipsis: true
    },
    {
      title: '端口',
      key: 'port',
      minWidth: 60
    },
    {
      title: '延迟',
      key: 'latency',
      minWidth: 80,
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
      minWidth: 80,
      render: (row: VpnConfig) => {
        const isCurrent = currentConfig.value?.id === row.id
        if (isCurrent) {
          const status = statusTagMap[connectionStatus.value]
          return h(NTag, { type: status.type }, { default: () => status.text })
        }
        return h(NTag, { type: 'default' }, { default: () => '未连接' })
      }
    },
    {
      title: '操作',
      key: 'actions',
      width: 180,
      fixed: 'right' as const,
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
                onClick: () => actions.disconnectConfig()
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
                onClick: () => actions.connectConfig(row)
              },
              {
                icon: () => h(NIcon, null, { default: () => h(PlayIcon) }),
                default: () => '连接'
              }
            )
          )
        }

        // 下拉菜单：其他操作
        const isTesting = actions.isConfigTesting(row.id)

        const handleSelect = (key: string) => {
          switch (key) {
            case 'edit':
              actions.editConfig(row)
              break
            case 'test':
              if (!isTesting) {
                actions.testSingleConfig(row)
              }
              break
            case 'export':
              actions.exportConfig(row)
              break
            case 'delete':
              if (confirm('确定要删除这个配置吗？')) {
                actions.deleteConfig(row.id)
              }
              break
          }
        }

        const dropdownOptions: DropdownOption[] = [
          {
            label: '编辑配置',
            key: 'edit',
            icon: () => h(NIcon, null, { default: () => h(EditIcon) })
          },
          {
            label: isTesting ? '测速中...' : '测速',
            key: 'test',
            disabled: isTesting,
            icon: () => h(NIcon, null, { default: () => h(CheckmarkIcon) })
          },
          {
            label: '导出配置',
            key: 'export',
            icon: () => h(NIcon, null, { default: () => h(ShareIcon) })
          },
          {
            type: 'divider',
            key: 'divider'
          },
          {
            label: '删除配置',
            key: 'delete',
            icon: () => h(NIcon, { color: '#d03050' }, { default: () => h(DeleteIcon) })
          }
        ]

        buttons.push(
          h(
            NDropdown,
            {
              trigger: 'click',
              options: dropdownOptions,
              placement: 'bottom-end',
              onSelect: handleSelect
            },
            {
              default: () =>
                h(
                  NButton,
                  {
                    size: 'small',
                    quaternary: true
                  },
                  {
                    icon: () => h(NIcon, null, { default: () => h(MoreIcon) })
                  }
                )
            }
          )
        )

        return h('div', { class: 'flex items-center gap-2' }, buttons)
      }
    }
  )

  return columns
}
