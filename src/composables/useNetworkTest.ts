/**
 * 网络测试 Composable
 */
import { ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { MessageApi } from 'naive-ui'
import type { VpnConfig, LatencyTestResult } from '../types/vpn'

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
  testingConfigs: Ref<Set<string>>
  isBatchTesting: Ref<boolean>
  isAutoConnecting: Ref<boolean>
  testSingleConfig: (config: VpnConfig) => Promise<void>
  handleBatchTest: () => Promise<void>
  cancelBatchTest: () => void
  handleAutoConnect: () => Promise<void>
  cancelAutoConnect: () => void
  clearTestResults: () => void
  isConfigTesting: (configId: string) => boolean
}

/**
 * 使用网络测试
 */
export function useNetworkTest(options: UseNetworkTestOptions): UseNetworkTestReturn {
  const { message, configs } = options

  const testResults = ref<Map<string, LatencyTestResult>>(new Map())
  const testingConfigs = ref<Set<string>>(new Set())
  const isBatchTesting = ref(false)
  const isAutoConnecting = ref(false)
  let cancelBatchTestFlag = false
  let cancelAutoConnectFlag = false

  /**
   * 检查配置是否正在测试中
   */
  const isConfigTesting = (configId: string): boolean => {
    return testingConfigs.value.has(configId)
  }

  /**
   * 取消批量测试
   */
  const cancelBatchTest = (): void => {
    if (isBatchTesting.value) {
      cancelBatchTestFlag = true
      message.warning('正在取消批量测试...')
    }
  }

  /**
   * 测试单个配置的延迟
   */
  const testSingleConfig = async (config: VpnConfig): Promise<void> => {
    // 防止重复测试
    if (testingConfigs.value.has(config.id)) {
      message.warning(`${config.name} 正在测试中，请稍候...`)
      return
    }

    try {
      testingConfigs.value.add(config.id)
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
    } finally {
      testingConfigs.value.delete(config.id)
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

    if (isBatchTesting.value) {
      message.warning('批量测试正在进行中，请稍候...')
      return
    }

    try {
      isBatchTesting.value = true
      cancelBatchTestFlag = false
      message.info(`正在测试 ${configs.value.length} 个配置...`)

      // 逐个测试，以便支持中断
      let testedCount = 0
      let successCount = 0

      for (const config of configs.value) {
        // 检查是否需要取消
        if (cancelBatchTestFlag) {
          message.info(`已取消批量测试。已测试 ${testedCount}/${configs.value.length} 个配置`)
          break
        }

        try {
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
            successCount++
          }
        } catch (error) {
          testResults.value.set(config.id, {
            latency: null,
            success: false,
            error: String(error)
          })
        }

        testedCount++
      }

      if (!cancelBatchTestFlag) {
        message.success(`测试完成: ${successCount}/${configs.value.length} 个配置可用`)
      }
    } catch (error) {
      message.error(`批量测试失败: ${error}`)
      throw error
    } finally {
      isBatchTesting.value = false
      cancelBatchTestFlag = false
    }
  }

  /**
   * 自动连接 - 测试并连接到第一个可用的服务
   */
  const handleAutoConnect = async (): Promise<void> => {
    if (configs.value.length === 0) {
      message.warning('没有可用的配置')
      return
    }

    isAutoConnecting.value = true
    cancelAutoConnectFlag = false

    try {
      message.info('🔍 开始自动连接，正在依次测试服务...')

      // 按优先级排序：优先测试之前有成功记录的
      const sortedConfigs = [...configs.value].sort((a, b) => {
        const aResult = testResults.value.get(a.id)
        const bResult = testResults.value.get(b.id)

        // 优先测试之前成功且延迟低的
        if (aResult?.success && bResult?.success) {
          return (aResult.latency || 9999) - (bResult.latency || 9999)
        }
        if (aResult?.success) return -1
        if (bResult?.success) return 1
        return 0
      })

      let connected = false
      let testedCount = 0

      for (const config of sortedConfigs) {
        if (cancelAutoConnectFlag) {
          message.info('⏹️ 自动连接已取消')
          break
        }

        testedCount++
        // 标记正在测试
        testingConfigs.value.add(config.id)

        try {
          message.info(`[${testedCount}/${sortedConfigs.length}] 正在测试：${config.name}`)

          // 测试延迟
          const params: TestLatencyParams = {
            server: config.server,
            port: config.port,
            count: 2, // 自动连接时减少测试次数以加快速度
            timeoutSecs: 3
          }

          const result = await invoke<LatencyTestResult>('test_latency_command', params)
          testResults.value.set(config.id, result)

          // 判断是否可用（延迟小于 3000ms 且成功）
          if (result.success && result.latency !== null && result.latency < 3000) {
            message.success(`✅ 找到可用服务：${config.name} (${result.latency}ms)`)

            // 连接到此服务
            try {
              await invoke('connect_vpn_command', { config })
              connected = true
              message.success(`🎉 已自动连接到：${config.name}`)
              break
            } catch (error) {
              message.error(`❌ 连接失败：${error}，继续尝试下一个...`)
              // 继续尝试下一个
            }
          } else {
            const reason = !result.success
              ? '无法连接'
              : result.latency === null
              ? '超时'
              : '延迟过高'
            message.warning(`⚠️ ${config.name} ${reason}，继续测试下一个...`)
          }
        } catch (error) {
          testResults.value.set(config.id, {
            latency: null,
            success: false,
            error: String(error)
          })
          message.warning(`⚠️ ${config.name} 测试失败，继续测试下一个...`)
        } finally {
          testingConfigs.value.delete(config.id)
        }
      }

      if (!connected && !cancelAutoConnectFlag) {
        message.error(`😔 未找到可用的服务（已测试 ${testedCount} 个配置）`)
      }
    } catch (error) {
      message.error(`自动连接失败: ${error}`)
    } finally {
      isAutoConnecting.value = false
      testingConfigs.value.clear()
      cancelAutoConnectFlag = false
    }
  }

  /**
   * 取消自动连接
   */
  const cancelAutoConnect = (): void => {
    if (isAutoConnecting.value) {
      cancelAutoConnectFlag = true
      message.warning('正在取消自动连接...')
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
    testingConfigs,
    isBatchTesting,
    isAutoConnecting,
    testSingleConfig,
    handleBatchTest,
    cancelBatchTest,
    handleAutoConnect,
    cancelAutoConnect,
    clearTestResults,
    isConfigTesting
  }
}
