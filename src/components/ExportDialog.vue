<template>
  <n-modal
    :show="show"
    @update:show="$emit('update:show', $event)"
    preset="dialog"
    title="导出配置"
    :mask-closable="false">
    <div v-if="config" class="space-y-4">
      <!-- 配置信息 -->
      <div class="border rounded-lg p-4 bg-gray-50">
        <h4 class="font-semibold mb-2">配置信息</h4>
        <div class="space-y-1 text-sm">
          <div>
            <strong>名称:</strong>
            {{ config.name }}
          </div>
          <div>
            <strong>协议:</strong>
            {{ config.protocol }}
          </div>
          <div>
            <strong>服务器:</strong>
            {{ config.server }}:{{ config.port }}
          </div>
        </div>
      </div>

      <!-- 导出方式选择 -->
      <div class="space-y-3">
        <h4 class="font-semibold">导出方式</h4>
        <n-radio-group v-model:value="exportType" name="exportType">
          <n-space vertical>
            <n-radio value="url">生成配置链接</n-radio>
            <n-radio value="qr">生成二维码</n-radio>
            <n-radio value="clipboard">复制到剪贴板</n-radio>
          </n-space>
        </n-radio-group>
      </div>

      <!-- 配置链接 -->
      <div v-if="exportType === 'url'" class="space-y-3">
        <n-input
          v-model:value="configUrl"
          type="textarea"
          placeholder="配置链接将在这里显示"
          :rows="3"
          readonly />
        <n-button type="primary" @click="copyToClipboard" :loading="copying">复制链接</n-button>
      </div>

      <!-- 二维码 -->
      <div v-if="exportType === 'qr'" class="space-y-3">
        <div class="text-center">
          <div v-if="qrCodeDataUrl" class="inline-block p-4 bg-white rounded-lg border">
            <img :src="qrCodeDataUrl" alt="配置二维码" class="w-48 h-48" />
          </div>
          <div
            v-else
            class="w-48 h-48 bg-gray-100 rounded-lg flex items-center justify-center mx-auto">
            <n-spin size="large" />
          </div>
        </div>
        <n-button type="primary" @click="downloadQrCode" :disabled="!qrCodeDataUrl">
          下载二维码
        </n-button>
      </div>

      <!-- 剪贴板 -->
      <div v-if="exportType === 'clipboard'" class="space-y-3">
        <n-button type="primary" @click="copyConfigToClipboard" :loading="copying">
          复制配置到剪贴板
        </n-button>
        <div class="text-sm text-gray-500">配置链接将直接复制到剪贴板</div>
      </div>
    </div>

    <template #action>
      <n-button @click="emit('update:show', false)">关闭</n-button>
    </template>
  </n-modal>
</template>

<script lang="ts" setup>
import { ref, watch } from 'vue'
import { NModal, NRadioGroup, NRadio, NSpace, NInput, NButton, NSpin, useMessage } from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'
import QRCode from 'qrcode'

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

const props = defineProps<{
  show: boolean
  config: VpnConfig | null
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
}>()

const message = useMessage()
const exportType = ref<'url' | 'qr' | 'clipboard'>('url')
const configUrl = ref('')
const qrCodeDataUrl = ref('')
const copying = ref(false)

// 监听配置变化，生成配置链接
watch(
  [() => props.config, () => props.show],
  async ([config, show]) => {
    if (config && show) {
      try {
        const url = await invoke('export_config_command', { config })
        configUrl.value = url as string

        // 生成二维码
        if (exportType.value === 'qr') {
          generateQrCode(url as string)
        }
      } catch (error) {
        message.error(`生成配置链接失败: ${error}`)
      }
    }
  },
  { immediate: true }
)

// 监听导出类型变化
watch(exportType, newType => {
  if (newType === 'qr' && configUrl.value) {
    generateQrCode(configUrl.value)
  }
})

// 生成二维码
const generateQrCode = async (text: string) => {
  try {
    const dataUrl = await QRCode.toDataURL(text, {
      width: 256,
      margin: 2,
      color: {
        dark: '#000000',
        light: '#FFFFFF'
      }
    })
    qrCodeDataUrl.value = dataUrl
  } catch (error) {
    message.error(`生成二维码失败: ${error}`)
  }
}

// 复制到剪贴板
const copyToClipboard = async () => {
  if (!configUrl.value) return

  try {
    copying.value = true
    await navigator.clipboard.writeText(configUrl.value)
    message.success('已复制到剪贴板')
  } catch (error) {
    message.error(`复制失败: ${error}`)
  } finally {
    copying.value = false
  }
}

// 复制配置到剪贴板
const copyConfigToClipboard = async () => {
  if (!configUrl.value) return

  try {
    copying.value = true
    await navigator.clipboard.writeText(configUrl.value)
    message.success('配置已复制到剪贴板')
  } catch (error) {
    message.error(`复制失败: ${error}`)
  } finally {
    copying.value = false
  }
}

// 下载二维码
const downloadQrCode = () => {
  if (!qrCodeDataUrl.value) return

  const link = document.createElement('a')
  link.download = `${props.config?.name || 'config'}-qrcode.png`
  link.href = qrCodeDataUrl.value
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
  message.success('二维码已下载')
}
</script>
