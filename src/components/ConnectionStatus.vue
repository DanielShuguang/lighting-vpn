<template>
  <div class="flex items-center space-x-2 bg-gray-50 rounded-lg px-3 py-2 border border-gray-200">
    <div class="flex items-center space-x-2">
      <div class="w-2 h-2 rounded-full" :class="statusClass"></div>
      <span class="text-sm font-medium">{{ statusText }}</span>
    </div>

    <div v-if="currentConfig" class="text-xs text-gray-500 max-w-[150px] truncate">
      {{ currentConfig.name }}
    </div>

    <!-- 测试连接按钮 -->
    <n-button
      v-if="props.status === 'connected'"
      type="info"
      size="tiny"
      @click="handleTestProxy"
      :loading="testing"
      class="ml-2">
      <template #icon>
        <n-icon><FlaskIcon /></n-icon>
      </template>
      测试
    </n-button>

    <!-- 断开连接按钮 -->
    <n-button
      v-if="props.status === 'connected'"
      type="error"
      size="tiny"
      @click="handleDisconnect"
      class="ml-2">
      <template #icon>
        <n-icon><StopIcon /></n-icon>
      </template>
      断开
    </n-button>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from 'vue'
import { NButton, NIcon, useMessage } from 'naive-ui'
import { Stop as StopIcon, Flask as FlaskIcon } from '@vicons/ionicons5'
import { invoke } from '@tauri-apps/api/core'
import type { VpnConfig, ConnectionStatus } from '../types/vpn'

interface Props {
  status: ConnectionStatus
  currentConfig?: VpnConfig | null
}

interface Emits {
  (e: 'disconnect'): void
}

const props = withDefaults(defineProps<Props>(), {
  status: 'disconnected',
  currentConfig: null
})

const emit = defineEmits<Emits>()
const message = useMessage()
const testing = ref(false)

const handleDisconnect = () => {
  emit('disconnect')
}

const handleTestProxy = async () => {
  testing.value = true
  try {
    const result = await invoke<string>('test_proxy_command')
    message.success(result)
  } catch (error) {
    message.error(String(error))
  } finally {
    testing.value = false
  }
}

const statusClass = computed(() => {
  switch (props.status) {
    case 'connected':
      return 'bg-green-500'
    case 'connecting':
      return 'bg-yellow-500 animate-pulse'
    case 'error':
      return 'bg-red-500'
    default:
      return 'bg-gray-400'
  }
})

const statusText = computed(() => {
  switch (props.status) {
    case 'connected':
      return '已连接'
    case 'connecting':
      return '连接中...'
    case 'error':
      return '连接失败'
    default:
      return '未连接'
  }
})
</script>
