/**
 * 表格列配置
 */
import { h, type Ref } from 'vue'
import type { DataTableColumns } from 'naive-ui'
import { NButton, NIcon, NTag, NPopconfirm } from 'naive-ui'
import {
  Play as PlayIcon,
  Pause as PauseIcon,
  Create as EditIcon,
  Trash as DeleteIcon,
  Share as ShareIcon,
  Checkmark as CheckmarkIcon
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
}

/**
 * 创建表格列配置
 */
export function createTableColumns(
  testResults: Ref<Map<string, LatencyTestResult>>,
  connectionStatus: Ref<ConnectionStatus>,
  currentConfig: Ref<VpnConfig | null>,
  actions: TableActions
): DataTableColumns<VpnConfig> {
  return [
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
        const protocol = protocolTagMap[row.protocol] || { type: 'default', text: row.protocol }
        return h(NTag, { type: protocol.type }, { default: () => protocol.text })
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
          const status = statusTagMap[connectionStatus.value]
          return h(NTag, { type: status.type }, { default: () => status.text })
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

        // 编辑按钮
        buttons.push(
          h(
            NButton,
            {
              size: 'small',
              onClick: () => actions.editConfig(row)
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
              onClick: () => actions.testSingleConfig(row)
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
              onClick: () => actions.exportConfig(row)
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
              onPositiveClick: () => actions.deleteConfig(row.id)
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
}
