/**
 * V2Ray 核心管理 Composable
 */
import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useMessage } from 'naive-ui'

/**
 * 核心信息
 */
export interface CoreInfo {
  installed: boolean
  version: string | null
  path: string | null
  platform: string
  latest_version: string | null
  has_update: boolean
}

/**
 * 下载进度
 */
export interface DownloadProgress {
  downloaded: number
  total: number
  percentage: number
}

/**
 * 核心管理返回类型
 */
export interface UseCoreManagerReturn {
  coreInfo: Ref<CoreInfo | null>
  downloading: Ref<boolean>
  progress: Ref<DownloadProgress>
  checkCore: () => Promise<void>
  downloadCore: (version?: string) => Promise<void>
  removeCore: () => Promise<void>
  formatBytes: (bytes: number) => string
}

/**
 * 核心管理 Hook
 */
export function useCoreManager(): UseCoreManagerReturn {
  const message = useMessage()
  const coreInfo = ref<CoreInfo | null>(null)
  const downloading = ref(false)
  const progress = ref<DownloadProgress>({
    downloaded: 0,
    total: 0,
    percentage: 0
  })

  /**
   * 检查核心状态
   */
  const checkCore = async (): Promise<void> => {
    try {
      const result = await invoke<CoreInfo>('check_core_command')
      coreInfo.value = result
    } catch (error) {
      message.error(`检查核心失败: ${error}`)
      throw error
    }
  }

  /**
   * 下载核心
   */
  const downloadCore = async (version?: string): Promise<void> => {
    try {
      downloading.value = true
      progress.value = { downloaded: 0, total: 0, percentage: 0 }

      // 监听下载进度
      const unlisten = await listen<DownloadProgress>('core-download-progress', event => {
        progress.value = event.payload
      })

      message.info('开始下载 V2Ray 核心...')

      // 开始下载
      await invoke<string>('download_core_command', { version })

      message.success('V2Ray 核心下载成功！')

      // 取消监听
      unlisten()

      // 重新检查核心状态
      await checkCore()
    } catch (error) {
      message.error(`下载核心失败: ${error}`)
      throw error
    } finally {
      downloading.value = false
    }
  }

  /**
   * 删除核心
   */
  const removeCore = async (): Promise<void> => {
    try {
      await invoke('remove_core_command')
      message.success('V2Ray 核心已删除')
      await checkCore()
    } catch (error) {
      message.error(`删除核心失败: ${error}`)
      throw error
    }
  }

  /**
   * 格式化字节数
   */
  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`
  }

  return {
    coreInfo,
    downloading,
    progress,
    checkCore,
    downloadCore,
    removeCore,
    formatBytes
  }
}
