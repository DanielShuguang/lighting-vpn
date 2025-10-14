<template>
  <n-modal
    :show="show"
    @update:show="$emit('update:show', $event)"
    preset="dialog"
    title="导入配置"
    :mask-closable="false">
    <div class="space-y-4">
      <!-- 导入方式选择 -->
      <n-radio-group v-model:value="importType" name="importType">
        <n-space>
          <n-radio value="url">URL 导入</n-radio>
          <n-radio value="qr">二维码扫描</n-radio>
          <n-radio value="clipboard">剪贴板导入</n-radio>
        </n-space>
      </n-radio-group>

      <!-- URL 导入 -->
      <div v-if="importType === 'url'" class="space-y-3">
        <n-input
          v-model:value="urlInput"
          type="textarea"
          placeholder="请输入 VPN 配置链接 (ss://, vmess://, trojan:// 等)"
          :rows="4" />
        <n-button type="primary" @click="importFromUrl" :loading="importing">解析并导入</n-button>
      </div>

      <!-- 二维码扫描 -->
      <div v-if="importType === 'qr'" class="space-y-3">
        <div class="flex space-x-2 justify-center">
          <n-button @click="startQrScan" :loading="qrScanning">
            {{ qrScanning ? '正在扫描...' : '摄像头扫描' }}
          </n-button>
          <n-button @click="selectImageFile" :loading="screenScanning">
            {{ screenScanning ? '正在识别...' : '选择图片' }}
          </n-button>
        </div>

        <!-- 摄像头扫描 -->
        <div v-if="qrScanning" class="relative">
          <video ref="videoRef" class="w-full h-64 bg-black rounded"></video>
          <div class="absolute inset-0 flex items-center justify-center">
            <div class="bg-black bg-opacity-50 text-white px-4 py-2 rounded">
              将二维码对准摄像头
            </div>
          </div>
        </div>

        <!-- 图片预览 -->
        <div v-if="screenImage" class="relative">
          <img
            :src="screenImage"
            alt="选择的图片"
            class="w-full h-64 object-contain bg-gray-100 rounded" />
          <div v-if="screenScanning" class="absolute inset-0 flex items-center justify-center">
            <div class="bg-black bg-opacity-50 text-white px-4 py-2 rounded">正在识别二维码...</div>
          </div>
          <div v-else class="absolute top-2 right-2">
            <n-button size="small" @click="screenImage = null">重新选择</n-button>
          </div>
        </div>

        <!-- 隐藏的文件输入 -->
        <input
          ref="fileInputRef"
          type="file"
          accept="image/*"
          style="display: none"
          @change="handleFileSelect" />

        <!-- 使用说明 -->
        <div
          v-if="importType === 'qr' && !qrScanning && !screenImage"
          class="text-sm text-gray-500 text-center">
          <p>选择扫描方式：</p>
          <p>• 摄像头扫描：将二维码对准摄像头</p>
          <p>• 选择图片：从本地选择包含二维码的图片</p>
        </div>
      </div>

      <!-- 剪贴板导入 -->
      <div v-if="importType === 'clipboard'" class="space-y-3">
        <n-button type="primary" @click="importFromClipboard" :loading="importing">
          从剪贴板导入
        </n-button>
        <div class="text-sm text-gray-500">请确保剪贴板中包含有效的 VPN 配置链接</div>
      </div>

      <!-- 预览配置 -->
      <div v-if="previewConfig" class="border rounded-lg p-4 bg-gray-50">
        <h4 class="font-semibold mb-2">配置预览</h4>
        <div class="space-y-2 text-sm">
          <div>
            <strong>名称:</strong>
            {{ previewConfig.name }}
          </div>
          <div>
            <strong>协议:</strong>
            {{ previewConfig.protocol }}
          </div>
          <div>
            <strong>服务器:</strong>
            {{ previewConfig.server }}
          </div>
          <div>
            <strong>端口:</strong>
            {{ previewConfig.port }}
          </div>
        </div>
        <div class="mt-4 flex space-x-2">
          <n-button type="primary" @click="confirmImport" :loading="importing">确认导入</n-button>
          <n-button @click="previewConfig = null">取消</n-button>
        </div>
      </div>
    </div>
  </n-modal>
</template>

<script lang="ts" setup>
import { ref, watch } from 'vue'
import { NModal, NRadioGroup, NRadio, NSpace, NInput, NButton, useMessage } from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'
import QrScanner from 'qr-scanner'

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
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
  imported: [config: VpnConfig]
}>()

const message = useMessage()
const importType = ref<'url' | 'qr' | 'clipboard'>('url')
const urlInput = ref('')
const importing = ref(false)
const qrScanning = ref(false)
const screenScanning = ref(false)
const previewConfig = ref<VpnConfig | null>(null)
const videoRef = ref<HTMLVideoElement>()
const fileInputRef = ref<HTMLInputElement>()
const screenImage = ref<string | null>(null)
let qrScanner: QrScanner | null = null

// 监听对话框显示状态
watch(
  () => props.show,
  newVal => {
    if (!newVal) {
      stopQrScan()
      resetForm()
    }
  }
)

// 重置表单
const resetForm = () => {
  urlInput.value = ''
  previewConfig.value = null
  importing.value = false
  screenImage.value = null
}

// URL 导入
const importFromUrl = async () => {
  if (!urlInput.value.trim()) {
    message.warning('请输入配置链接')
    return
  }

  try {
    importing.value = true
    const config = await invoke('parse_config_url_command', { url: urlInput.value.trim() })
    previewConfig.value = config as VpnConfig
  } catch (error) {
    message.error(`解析配置失败: ${error}`)
  } finally {
    importing.value = false
  }
}

// 开始二维码扫描
const startQrScan = async () => {
  try {
    qrScanning.value = true

    if (!videoRef.value) {
      message.error('无法访问摄像头')
      return
    }

    qrScanner = new QrScanner(
      videoRef.value,
      result => {
        handleQrResult(result.data)
      },
      {
        onDecodeError: () => {
          // 忽略解码错误，继续扫描
        }
      }
    )

    await qrScanner.start()
  } catch (error) {
    message.error(`启动摄像头失败: ${error}`)
    qrScanning.value = false
  }
}

// 停止二维码扫描
const stopQrScan = () => {
  if (qrScanner) {
    qrScanner.stop()
    qrScanner.destroy()
    qrScanner = null
  }
  qrScanning.value = false
}

// 处理二维码扫描结果
const handleQrResult = async (data: string) => {
  stopQrScan()

  try {
    const config = await invoke('parse_config_url_command', { url: data })
    previewConfig.value = config as VpnConfig
  } catch (error) {
    message.error(`解析二维码配置失败: ${error}`)
  }
}

// 选择图片文件
const selectImageFile = () => {
  fileInputRef.value?.click()
}

// 处理文件选择
const handleFileSelect = async (event: Event) => {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]

  if (!file) return

  try {
    screenScanning.value = true
    screenImage.value = null

    // 读取文件为Data URL
    const reader = new FileReader()
    reader.onload = async e => {
      const imageUrl = e.target?.result as string
      screenImage.value = imageUrl

      try {
        // 识别二维码
        const result = await QrScanner.scanImage(imageUrl)
        if (result) {
          await handleQrResult(result)
        } else {
          message.warning('未在图片中找到二维码')
        }
      } catch (error) {
        message.error(`识别二维码失败: ${error}`)
      } finally {
        screenScanning.value = false
      }
    }

    reader.readAsDataURL(file)
  } catch (error) {
    message.error(`读取图片失败: ${error}`)
    screenScanning.value = false
  }
}

// 从剪贴板导入
const importFromClipboard = async () => {
  try {
    importing.value = true
    const clipboardText = await navigator.clipboard.readText()

    if (!clipboardText.trim()) {
      message.warning('剪贴板为空')
      return
    }

    const config = await invoke('parse_config_url_command', { url: clipboardText.trim() })
    previewConfig.value = config as VpnConfig
  } catch (error) {
    message.error(`从剪贴板导入失败: ${error}`)
  } finally {
    importing.value = false
  }
}

// 确认导入
const confirmImport = async () => {
  if (!previewConfig.value) return

  try {
    importing.value = true

    // 加载现有配置
    const result = await invoke('load_configs_command')
    const configs = (result as any).configs || []

    // 添加新配置
    configs.push(previewConfig.value)

    // 保存配置
    await invoke('save_configs_command', { configs: { configs } })

    emit('imported', previewConfig.value)
    emit('update:show', false)
    message.success('配置导入成功')
  } catch (error) {
    message.error(`保存配置失败: ${error}`)
  } finally {
    importing.value = false
  }
}
</script>
