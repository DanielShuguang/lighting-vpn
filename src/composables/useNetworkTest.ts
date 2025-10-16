/**
 * 网络测试 Composable
 */
import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { MessageApi } from 'naive-ui'
import type { VpnConfig, LatencyTestResult, BatchTestResult } from '../types/vpn'

export interface UseNetworkTestOptions {
  message: MessageApi
  configs: Ref<VpnConfig[]>
}

export interface TestLatencyParams extends Record<string, unknown> {
  server: string
  port: number
  count: number
  timeoutSecs: number
}

export interface BatchTestParams extends Record<string, unknown> {
  configs: [string, string, string, number][]
  count: number
  timeoutSecs: number
}

export interface UseNetworkTestReturn {
  testResults: Ref<Map<string, LatencyTestResult>>
  testSingleConfig: (config: VpnConfig) => Promise<void>
  handleBatchTest: () => Promise<void>
  clearTestResults: () => void
}

/**
 * 使用网络测试
 */
export function useNetworkTest(options: UseNetworkTestOptions): UseNetworkTestReturn {
  const { message, configs } = options

  const testResults = ref<Map<string, LatencyTestResult>>(new Map())

  /**
   * 测试单个配置的延迟
   */
  const testSingleConfig = async (config: VpnConfig): Promise<void> => {
    try {
      message.info(`正在测试 ${config.name}...`)

      const params: TestLatencyParams = {
        server: config.server,
        port: config.port,
        count: 3,
        timeoutSecs: 5
      }

      const result = await invoke<LatencyTestResult>('test_latency_command', params)

      testResults.value.set(config.id, {
        latency: result.latency,
        success: result.success,
        error: result.error
      })

      if (result.success) {
        message.success(`${config.name} 延迟: ${result.latency}ms`)
      } else {
        message.error(`${config.name} 测试失败: ${result.error || '未知错误'}`)
      }
    } catch (error) {
      message.error(`测试失败: ${error}`)
      testResults.value.set(config.id, {
        latency: null,
        success: false,
        error: String(error)
      })
      throw error
    }
  }

  /**
   * 批量测试所有配置
   */
  const handleBatchTest = async (): Promise<void> => {
    if (configs.value.length === 0) {
      message.warning('没有可测试的配置')
      return
    }

    try {
      message.info(`正在测试 ${configs.value.length} 个配置...`)

      const configsData: [string, string, string, number][] = configs.value.map(c => [
        c.id,
        c.name,
        c.server,
        c.port
      ])

      const params: BatchTestParams = {
        configs: configsData,
        count: 3,
        timeoutSecs: 5
      }

      const results = await invoke<BatchTestResult[]>('batch_test_latencies_command', params)

      // 更新测试结果
      results.forEach((result: BatchTestResult) => {
        testResults.value.set(result.config_id, {
          latency: result.result.latency,
          success: result.result.success,
          error: result.result.error
        })
      })

      const successCount = results.filter(r => r.result.success).length
      message.success(`测试完成: ${successCount}/${configs.value.length} 个配置可用`)
    } catch (error) {
      message.error(`批量测试失败: ${error}`)
      throw error
    }
  }

  /**
   * 清除测试结果
   */
  const clearTestResults = (): void => {
    testResults.value.clear()
  }

  return {
    testResults,
    testSingleConfig,
    handleBatchTest,
    clearTestResults
  }
}
