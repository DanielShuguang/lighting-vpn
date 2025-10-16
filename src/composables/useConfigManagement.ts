/**
 * 配置管理 Composable
 */
import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { MessageApi } from 'naive-ui'
import type { VpnConfig, ConfigsResponse } from '../types/vpn'

export interface UseConfigManagementOptions {
  message: MessageApi
}

export interface UseConfigManagementReturn {
  configs: Ref<VpnConfig[]>
  loading: Ref<boolean>
  loadConfigs: () => Promise<void>
  refreshConfigs: () => void
  deleteConfig: (id: string) => Promise<void>
  updateConfig: (updatedConfig: VpnConfig) => void
  addConfig: (newConfig: VpnConfig) => void
}

/**
 * 使用配置管理
 */
export function useConfigManagement(
  options: UseConfigManagementOptions
): UseConfigManagementReturn {
  const { message } = options

  const configs = ref<VpnConfig[]>([])
  const loading = ref(false)

  /**
   * 加载配置列表
   */
  const loadConfigs = async (): Promise<void> => {
    try {
      loading.value = true
      const result = await invoke<ConfigsResponse>('load_configs_command')
      configs.value = result.configs || []
    } catch (error) {
      message.error(`加载配置失败: ${error}`)
      throw error
    } finally {
      loading.value = false
    }
  }

  /**
   * 刷新配置列表
   */
  const refreshConfigs = (): void => {
    loadConfigs()
  }

  /**
   * 删除配置
   */
  const deleteConfig = async (id: string): Promise<void> => {
    try {
      const currentConfigs = configs.value.filter(c => c.id !== id)
      await invoke('save_configs_command', { configs: { configs: currentConfigs } })
      configs.value = currentConfigs
      message.success('配置删除成功')
    } catch (error) {
      message.error(`删除配置失败: ${error}`)
      throw error
    }
  }

  /**
   * 更新配置
   */
  const updateConfig = (updatedConfig: VpnConfig): void => {
    const index = configs.value.findIndex(c => c.id === updatedConfig.id)
    if (index !== -1) {
      configs.value[index] = updatedConfig
    }
  }

  /**
   * 添加配置
   */
  const addConfig = (newConfig: VpnConfig): void => {
    configs.value.push(newConfig)
  }

  return {
    configs,
    loading,
    loadConfigs,
    refreshConfigs,
    deleteConfig,
    updateConfig,
    addConfig
  }
}
