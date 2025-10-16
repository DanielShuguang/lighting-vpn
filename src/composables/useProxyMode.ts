/**
 * 代理模式管理 Composable
 */
import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'
import type { ProxyMode } from '../types/vpn'

export interface UseProxyModeReturn {
  currentMode: Ref<ProxyMode>
  loading: Ref<boolean>
  loadProxyMode: () => Promise<void>
  setProxyMode: (mode: ProxyMode) => Promise<void>
}

/**
 * 代理模式管理 Hook
 */
export function useProxyMode(): UseProxyModeReturn {
  const message = useMessage()
  const currentMode = ref<ProxyMode>('pac')
  const loading = ref(false)

  /**
   * 加载当前代理模式
   */
  const loadProxyMode = async () => {
    try {
      loading.value = true
      const mode = await invoke<string>('get_proxy_mode_command')
      currentMode.value = mode as ProxyMode
    } catch (error) {
      console.error('加载代理模式失败:', error)
      message.error(`加载代理模式失败: ${error}`)
      // 设置默认值
      currentMode.value = 'pac'
    } finally {
      loading.value = false
    }
  }

  /**
   * 设置代理模式
   */
  const setProxyMode = async (mode: ProxyMode) => {
    try {
      loading.value = true
      await invoke('set_proxy_mode_command', { mode })
      currentMode.value = mode

      // 根据模式给出不同的提示
      const modeMessages = {
        global: '全局代理模式：所有流量都将通过代理',
        pac: 'PAC模式：智能分流（国内直连，国外走代理）',
        direct: '直连模式：仅代理需要代理的流量'
      }

      message.success(`已切换到${modeMessages[mode]}`)
    } catch (error) {
      console.error('设置代理模式失败:', error)
      message.error(`设置代理模式失败: ${error}`)
      throw error
    } finally {
      loading.value = false
    }
  }

  return {
    currentMode,
    loading,
    loadProxyMode,
    setProxyMode
  }
}
