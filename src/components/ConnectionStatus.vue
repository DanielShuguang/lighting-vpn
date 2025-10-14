<template>
  <div class="flex items-center space-x-2">
    <div class="flex items-center space-x-1">
      <div class="w-2 h-2 rounded-full" :class="statusClass"></div>
      <span class="text-sm font-medium">{{ statusText }}</span>
    </div>

    <div v-if="currentConfig" class="text-xs text-gray-500">
      {{ currentConfig.name }}
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed } from 'vue'

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

interface Props {
  status: 'disconnected' | 'connecting' | 'connected' | 'error'
  currentConfig?: VpnConfig | null
}

const props = withDefaults(defineProps<Props>(), {
  status: 'disconnected',
  currentConfig: null
})

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
