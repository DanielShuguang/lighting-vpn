<template>
  <n-modal
    v-model:show="showModal"
    preset="card"
    title="订阅管理"
    :style="{ width: '90vw', maxWidth: '800px' }"
    :mask-closable="false">
    <div class="space-y-4">
      <!-- 添加订阅表单（折叠面板） -->
      <n-collapse v-model:expanded-names="expandedKeys" accordion>
        <n-collapse-item name="add-subscription">
          <template #header>
            <div class="flex items-center gap-2">
              <n-icon :component="AddIcon" />
              <span class="font-semibold">添加新订阅</span>
            </div>
          </template>
          <template #header-extra>
            <n-icon :component="ChevronDownIcon" class="text-gray-400" />
          </template>

          <div class="pt-2">
            <n-form ref="formRef" :model="newSubscription" :rules="rules">
              <n-grid :cols="1" :sm-cols="2" :x-gap="12" :y-gap="8">
                <n-form-item-gi path="name" label="订阅名称">
                  <n-input v-model:value="newSubscription.name" placeholder="例如：机场A" />
                </n-form-item-gi>
                <n-form-item-gi path="update_interval" label="更新间隔(小时)">
                  <n-input-number
                    v-model:value="newSubscription.update_interval"
                    :min="1"
                    :max="720"
                    placeholder="24" />
                </n-form-item-gi>
                <n-form-item-gi :span="1" :sm-span="2" path="url" label="订阅地址">
                  <n-input
                    v-model:value="newSubscription.url"
                    type="textarea"
                    placeholder="https://example.com/subscription"
                    :autosize="{ minRows: 2, maxRows: 4 }" />
                </n-form-item-gi>
                <n-form-item-gi :span="1" :sm-span="2" label="选项">
                  <n-checkbox v-model:checked="newSubscription.use_proxy">使用代理更新</n-checkbox>
                </n-form-item-gi>
              </n-grid>
              <div class="flex justify-end mt-4">
                <n-button type="primary" @click="handleAddSubscription" :loading="adding">
                  <template #icon>
                    <n-icon :component="AddIcon" />
                  </template>
                  添加订阅
                </n-button>
              </div>
            </n-form>
          </div>
        </n-collapse-item>
      </n-collapse>

      <!-- 订阅列表 -->
      <div>
        <h3 class="text-sm font-semibold mb-3 text-gray-700">订阅列表</h3>
        <div class="overflow-x-auto">
          <n-data-table
            :columns="columns"
            :data="subscriptions"
            :loading="loading"
            :pagination="false"
            :bordered="false"
            :scroll-x="900"
            size="small" />
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex justify-end">
        <n-button @click="showModal = false">关闭</n-button>
      </div>
    </template>
  </n-modal>
</template>

<script lang="ts" setup>
import { ref, computed, h, onMounted } from 'vue'
import {
  NModal,
  NForm,
  NFormItemGi,
  NGrid,
  NInput,
  NInputNumber,
  NCheckbox,
  NButton,
  NDataTable,
  NTag,
  NIcon,
  NCollapse,
  NCollapseItem,
  NDropdown,
  useMessage,
  FormInst
} from 'naive-ui'
import type { DropdownOption } from 'naive-ui'
import {
  Refresh as RefreshIcon,
  Trash as DeleteIcon,
  Add as AddIcon,
  ChevronDown as ChevronDownIcon,
  EllipsisHorizontal as MoreIcon
} from '@vicons/ionicons5'
import { invoke } from '@tauri-apps/api/core'

interface Subscription {
  id: string
  name: string
  url: string
  enabled: boolean
  use_proxy: boolean
  update_interval: number
  last_update?: string
  config_count: number
  created_at: string
}

const props = defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
  subscriptionUpdated: []
}>()

const message = useMessage()
const formRef = ref<FormInst | null>(null)
const loading = ref(false)
const adding = ref(false)
const subscriptions = ref<Subscription[]>([])
const expandedKeys = ref<string[]>([])

const showModal = computed({
  get: () => props.show,
  set: value => emit('update:show', value)
})

const newSubscription = ref({
  name: '',
  url: '',
  use_proxy: false,
  update_interval: 24
})

const rules = {
  name: {
    required: true,
    message: '请输入订阅名称',
    trigger: 'blur'
  },
  url: {
    required: true,
    message: '请输入订阅地址',
    trigger: 'blur'
  }
}

const columns = [
  {
    title: '名称',
    key: 'name',
    minWidth: 100,
    ellipsis: true
  },
  {
    title: '状态',
    key: 'enabled',
    minWidth: 70,
    render: (row: Subscription) => {
      return h(
        NTag,
        { type: row.enabled ? 'success' : 'default', size: 'small' },
        { default: () => (row.enabled ? '启用' : '禁用') }
      )
    }
  },
  {
    title: '节点数',
    key: 'config_count',
    minWidth: 80
  },
  {
    title: '最后更新',
    key: 'last_update',
    minWidth: 140,
    render: (row: Subscription) => {
      if (!row.last_update) return '从未更新'
      return new Date(row.last_update).toLocaleString('zh-CN')
    }
  },
  {
    title: '更新间隔',
    key: 'update_interval',
    minWidth: 90,
    render: (row: Subscription) => `${row.update_interval}小时`
  },
  {
    title: '代理',
    key: 'use_proxy',
    minWidth: 60,
    render: (row: Subscription) => {
      return h(
        NTag,
        { type: row.use_proxy ? 'info' : 'default', size: 'small' },
        { default: () => (row.use_proxy ? '是' : '否') }
      )
    }
  },
  {
    title: '操作',
    key: 'actions',
    width: 140,
    fixed: 'right' as const,
    render: (row: Subscription) => {
      const handleSelect = (key: string) => {
        switch (key) {
          case 'refresh':
            handleRefreshSubscription(row.id)
            break
          case 'delete':
            if (confirm('确定要删除这个订阅吗？')) {
              handleDeleteSubscription(row.id)
            }
            break
        }
      }

      const dropdownOptions: DropdownOption[] = [
        {
          label: '更新订阅',
          key: 'refresh',
          icon: () => h(NIcon, null, { default: () => h(RefreshIcon) })
        },
        {
          type: 'divider',
          key: 'divider'
        },
        {
          label: '删除订阅',
          key: 'delete',
          icon: () => h(NIcon, { color: '#d03050' }, { default: () => h(DeleteIcon) })
        }
      ]

      return h('div', { class: 'flex items-center gap-2' }, [
        h(
          NButton,
          {
            size: 'small',
            type: 'primary',
            onClick: () => handleRefreshSubscription(row.id)
          },
          {
            icon: () => h(NIcon, null, { default: () => h(RefreshIcon) })
          }
        ),
        h(
          NDropdown,
          {
            trigger: 'click',
            options: dropdownOptions,
            placement: 'bottom-end',
            onSelect: handleSelect
          },
          {
            default: () =>
              h(
                NButton,
                {
                  size: 'small',
                  quaternary: true
                },
                {
                  icon: () => h(NIcon, null, { default: () => h(MoreIcon) })
                }
              )
          }
        )
      ])
    }
  }
]

// 加载订阅列表
const loadSubscriptions = async () => {
  try {
    loading.value = true
    const result = await invoke('load_subscriptions_command')
    subscriptions.value = (result as any).subscriptions || []
  } catch (error) {
    message.error(`加载订阅列表失败: ${error}`)
  } finally {
    loading.value = false
  }
}

// 添加订阅
const handleAddSubscription = async () => {
  try {
    await formRef.value?.validate()
    adding.value = true

    const result = await invoke('add_subscription_command', {
      name: newSubscription.value.name,
      url: newSubscription.value.url,
      useProxy: newSubscription.value.use_proxy,
      updateInterval: newSubscription.value.update_interval
    })

    const subscriptionId = (result as any).id

    message.success('订阅添加成功，正在获取节点...')

    // 立即更新订阅以获取配置
    try {
      const newConfigs = await invoke('refresh_subscription_command', {
        id: subscriptionId,
        proxyUrl: null
      })

      const result = await mergeConfigsToList(newConfigs as any[], subscriptionId)

      if (result.total > 0) {
        message.success(`订阅添加完成！获取到 ${result.total} 个节点`)
        emit('subscriptionUpdated')
      } else {
        message.warning('订阅添加成功，但未获取到任何节点')
      }
    } catch (updateError) {
      message.warning(`订阅已添加，但获取节点失败: ${updateError}`)
    }

    newSubscription.value = {
      name: '',
      url: '',
      use_proxy: false,
      update_interval: 24
    }
    // 添加成功后收起折叠面板
    expandedKeys.value = []
    await loadSubscriptions()
  } catch (error: any) {
    if (error.errorFields) {
      // 表单验证错误
      return
    }
    message.error(`添加订阅失败: ${error}`)
  } finally {
    adding.value = false
  }
}

// 刷新订阅
const handleRefreshSubscription = async (id: string) => {
  try {
    message.info('正在更新订阅...')
    const newConfigs = await invoke('refresh_subscription_command', {
      id,
      proxyUrl: null
    })

    const result = await mergeConfigsToList(newConfigs as any[], id)

    if (result.total === 0) {
      message.warning('订阅更新成功，但未获取到任何节点')
    } else {
      const msgs = [`获取 ${result.total} 个节点`]
      if (result.removed > 0) {
        msgs.push(`替换 ${result.removed} 个旧节点`)
      }
      message.success(`订阅更新成功！${msgs.join('，')}`)
      emit('subscriptionUpdated')
    }

    await loadSubscriptions()
  } catch (error) {
    message.error(`更新订阅失败: ${error}`)
  }
}

// 删除订阅
const handleDeleteSubscription = async (id: string) => {
  try {
    await invoke('delete_subscription_command', { id })
    message.success('订阅删除成功')
    await loadSubscriptions()
  } catch (error) {
    message.error(`删除订阅失败: ${error}`)
  }
}

// 合并配置到现有列表（同订阅覆盖，不同订阅去重）
const mergeConfigsToList = async (newConfigs: any[], subscriptionId?: string) => {
  if (newConfigs.length === 0) {
    return { total: 0, added: 0, removed: 0 }
  }

  // 加载现有配置
  const existingResult = await invoke('load_configs_command')
  let existingConfigs = (existingResult as any).configs || []

  let addedCount = 0
  let removedCount = 0

  if (subscriptionId) {
    // 如果是订阅更新，先删除该订阅的所有旧配置
    const oldSubscriptionConfigs = existingConfigs.filter(
      (config: any) => config.subscription_id === subscriptionId
    )
    removedCount = oldSubscriptionConfigs.length

    existingConfigs = existingConfigs.filter(
      (config: any) => config.subscription_id !== subscriptionId
    )

    // 直接添加新配置
    const mergedConfigs = [...existingConfigs, ...newConfigs]
    addedCount = newConfigs.length

    await invoke('save_configs_command', {
      configs: { configs: mergedConfigs }
    })

    return {
      total: newConfigs.length,
      added: addedCount,
      removed: removedCount
    }
  } else {
    // 如果不是订阅更新（手动导入），使用去重逻辑
    const configMap = new Map()
    existingConfigs.forEach((config: any) => {
      const key = `${config.name}-${config.server}-${config.port}`
      configMap.set(key, config)
    })

    newConfigs.forEach((config: any) => {
      const key = `${config.name}-${config.server}-${config.port}`
      if (!configMap.has(key)) {
        configMap.set(key, config)
        addedCount++
      }
    })

    const mergedConfigs = Array.from(configMap.values())
    await invoke('save_configs_command', {
      configs: { configs: mergedConfigs }
    })

    return {
      total: newConfigs.length,
      added: addedCount,
      removed: 0
    }
  }
}

onMounted(() => {
  loadSubscriptions()
})
</script>
