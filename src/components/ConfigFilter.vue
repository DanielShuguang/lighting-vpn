<template>
  <div class="config-filter space-y-2">
    <div class="flex flex-col sm:flex-row gap-2">
      <!-- 名称搜索 -->
      <n-input
        v-model:value="internalSearchText"
        placeholder="搜索配置名称..."
        clearable
        size="small"
        class="flex-1">
        <template #prefix>
          <n-icon><SearchIcon /></n-icon>
        </template>
      </n-input>

      <!-- 订阅筛选 -->
      <n-select
        v-model:value="selectedSubscription"
        :options="subscriptionOptions"
        placeholder="全部订阅"
        clearable
        size="small"
        class="w-full sm:w-48">
        <template #prefix>
          <n-icon><CloudIcon /></n-icon>
        </template>
      </n-select>

      <!-- 协议筛选 -->
      <n-select
        v-model:value="selectedProtocol"
        :options="protocolOptions"
        placeholder="全部协议"
        clearable
        size="small"
        class="w-full sm:w-48">
        <template #prefix>
          <n-icon><FilterIcon /></n-icon>
        </template>
      </n-select>
    </div>

    <!-- 统计信息 -->
    <div v-if="showStats" class="flex items-center gap-3 text-xs text-gray-600">
      <span>共 {{ totalCount }} 个配置</span>
      <span v-if="filteredCount !== totalCount" class="text-blue-600">
        筛选后 {{ filteredCount }} 个
      </span>
      <span v-if="subscriptionStats.length > 0" class="text-gray-400">
        | {{ subscriptionStats.length }} 个订阅源
      </span>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, watch, ref } from 'vue'
import { useDebounceFn } from '@vueuse/core'
import { NInput, NSelect, NIcon } from 'naive-ui'
import { Search as SearchIcon, Cloud as CloudIcon, Filter as FilterIcon } from '@vicons/ionicons5'
import type { VpnConfig } from '../types/vpn'

interface Subscription {
  id: string
  name: string
}

interface Props {
  configs: VpnConfig[]
  subscriptions: Subscription[]
  showStats?: boolean
}

interface Emits {
  (e: 'update:searchText', value: string): void
  (e: 'update:selectedSubscription', value: string | null): void
  (e: 'update:selectedProtocol', value: string | null): void
}

const props = withDefaults(defineProps<Props>(), {
  showStats: true
})

const emit = defineEmits<Emits>()

// 内部搜索文本（用于立即响应输入）
const internalSearchText = ref('')

// 搜索和筛选状态
const searchText = defineModel<string>('searchText', { default: '' })
const selectedSubscription = defineModel<string | null>('selectedSubscription', { default: null })
const selectedProtocol = defineModel<string | null>('selectedProtocol', { default: null })

// 防抖更新搜索文本（300ms）
const debouncedUpdateSearch = useDebounceFn((value: string) => {
  searchText.value = value
  emit('update:searchText', value)
}, 300)

// 监听内部搜索文本变化
watch(internalSearchText, newValue => {
  debouncedUpdateSearch(newValue)
})

// 统计信息
const totalCount = computed(() => props.configs.length)

const filteredCount = computed(() => {
  return props.configs.filter(config => {
    const matchName =
      !searchText.value || config.name.toLowerCase().includes(searchText.value.toLowerCase())
    const matchSubscription =
      !selectedSubscription.value || config.subscription_id === selectedSubscription.value
    const matchProtocol = !selectedProtocol.value || config.protocol === selectedProtocol.value
    return matchName && matchSubscription && matchProtocol
  }).length
})

// 统计订阅
const subscriptionStats = computed(() => {
  const stats = new Map<string, { name: string; count: number }>()

  // 添加手动导入的配置统计
  const manualCount = props.configs.filter(c => !c.subscription_id).length
  if (manualCount > 0) {
    stats.set('manual', { name: '手动导入', count: manualCount })
  }

  // 统计各订阅的配置数量
  props.configs.forEach(config => {
    if (config.subscription_id) {
      const sub = props.subscriptions.find(s => s.id === config.subscription_id)
      if (sub) {
        const existing = stats.get(config.subscription_id)
        if (existing) {
          existing.count++
        } else {
          stats.set(config.subscription_id, { name: sub.name, count: 1 })
        }
      }
    }
  })

  return Array.from(stats.entries())
    .map(([id, data]) => ({ id, ...data }))
    .sort((a, b) => b.count - a.count)
})

// 订阅选项
const subscriptionOptions = computed(() => {
  return subscriptionStats.value.map(stat => ({
    label: `${stat.name} (${stat.count})`,
    value: stat.id === 'manual' ? null : stat.id
  }))
})

// 协议选项
const protocolOptions = computed(() => {
  const protocols = new Set(props.configs.map(c => c.protocol))
  return Array.from(protocols).map(protocol => ({
    label: protocol,
    value: protocol
  }))
})

// 监听变化，触发事件（订阅和协议不需要防抖，立即触发）
watch(selectedSubscription, value => emit('update:selectedSubscription', value))
watch(selectedProtocol, value => emit('update:selectedProtocol', value))

// 初始化内部搜索文本
watch(
  () => searchText.value,
  newValue => {
    if (newValue !== internalSearchText.value) {
      internalSearchText.value = newValue
    }
  },
  { immediate: true }
)
</script>

<style scoped>
.config-filter {
  background: white;
  padding: 12px;
  border-radius: 8px;
  border: 1px solid #e5e7eb;
}
</style>
