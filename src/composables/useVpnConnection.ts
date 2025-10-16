/**
 * VPN 连接管理 Composable
 */
import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { MessageApi } from 'naive-ui'
import type { VpnConfig, ConnectionStatus, ConnectionInfo } from '../types/vpn'

export interface UseVpnConnectionOptions {
  message: MessageApi
  configs: Ref<VpnConfig[]>
}

export interface UseVpnConnectionReturn {
  connectionStatus: Ref<ConnectionStatus>
  currentConfig: Ref<VpnConfig | null>
  connectConfig: (config: VpnConfig) => Promise<void>
  disconnectConfig: () => Promise<void>
  checkConnectionStatus: () => Promise<void>
}

/**
 * 使用 VPN 连接管理
 */
export function useVpnConnection(options: UseVpnConnectionOptions): UseVpnConnectionReturn {
  const { message, configs } = options

  const connectionStatus = ref<ConnectionStatus>('disconnected')
  const currentConfig = ref<VpnConfig | null>(null)

  /**
   * 连接到指定配置
   */
  const connectConfig = async (config: VpnConfig): Promise<void> => {
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
      throw error
    }
  }

  /**
   * 断开当前连接
   */
  const disconnectConfig = async (): Promise<void> => {
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
      throw error
    }
  }

  /**
   * 检查当前连接状态
   */
  const checkConnectionStatus = async (): Promise<void> => {
    try {
      const isConnected = await invoke<boolean>('is_connected_command')
      if (isConnected) {
        const info = await invoke<ConnectionInfo>('get_connection_status_command')
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

  return {
    connectionStatus,
    currentConfig,
    connectConfig,
    disconnectConfig,
    checkConnectionStatus
  }
}
