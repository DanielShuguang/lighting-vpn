<template>
  <n-modal
    v-model:show="showModal"
    preset="card"
    title="订阅管理"
    style="width: 800px"
    :mask-closable="false">
    <div class="space-y-4">
      <!-- 添加订阅表单 -->
      <div class="bg-gray-50 p-4 rounded-lg">
        <h3 class="text-sm font-semibold mb-3 text-gray-700">添加新订阅</h3>
        <n-form ref="formRef" :model="newSubscription" :rules="rules">
          <n-grid :cols="2" :x-gap="12" :y-gap="8">
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
            <n-form-item-gi :span="2" path="url" label="订阅地址">
              <n-input
                v-model:value="newSubscription.url"
                type="textarea"
                placeholder="https://example.com/subscription"
                :autosize="{ minRows: 2, maxRows: 4 }" />
            </n-form-item-gi>
            <n-form-item-gi :span="2" label="选项">
              <n-checkbox v-model:checked="newSubscription.use_proxy">使用代理更新</n-checkbox>
            </n-form-item-gi>
          </n-grid>
          <div class="flex justify-end">
            <n-button type="primary" @click="handleAddSubscription" :loading="adding">
              添加订阅
            </n-button>
          </div>
        </n-form>
      </div>

      <!-- 订阅列表 -->
      <div>
        <h3 class="text-sm font-semibold mb-3 text-gray-700">订阅列表</h3>
        <n-data-table
          :columns="columns"
          :data="subscriptions"
          :loading="loading"
          :pagination="false"
          :bordered="false"
          size="small" />
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
  NPopconfirm,
  NIcon,
  useMessage,
  FormInst
} from 'naive-ui'
import { Refresh as RefreshIcon, Trash as DeleteIcon } from '@vicons/ionicons5'
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
    width: 120,
    ellipsis: true
  },
  {
    title: '状态',
    key: 'enabled',
    width: 70,
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
    width: 80
  },
  {
    title: '最后更新',
    key: 'last_update',
    width: 150,
    render: (row: Subscription) => {
      if (!row.last_update) return '从未更新'
      return new Date(row.last_update).toLocaleString('zh-CN')
    }
  },
  {
    title: '更新间隔',
    key: 'update_interval',
    width: 100,
    render: (row: Subscription) => `${row.update_interval}小时`
  },
  {
    title: '代理',
    key: 'use_proxy',
    width: 70,
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
    width: 120,
    render: (row: Subscription) => {
      return h('div', { class: 'flex space-x-2' }, [
        h(
          NButton,
          {
            size: 'small',
            type: 'primary',
            onClick: () => handleRefreshSubscription(row.id)
          },
          {
            icon: () => h(NIcon, null, { default: () => h(RefreshIcon) }),
            default: () => '更新'
          }
        ),
        h(
          NPopconfirm,
          {
            onPositiveClick: () => handleDeleteSubscription(row.id)
          },
          {
            trigger: () =>
              h(
                NButton,
                {
                  size: 'small',
                  type: 'error',
                  quaternary: true
                },
                {
                  icon: () => h(NIcon, null, { default: () => h(DeleteIcon) }),
                  default: () => '删除'
                }
              ),
            default: () => '确定要删除这个订阅吗？'
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

    await invoke('add_subscription_command', {
      name: newSubscription.value.name,
      url: newSubscription.value.url,
      useProxy: newSubscription.value.use_proxy,
      updateInterval: newSubscription.value.update_interval
    })

    message.success('订阅添加成功')
    newSubscription.value = {
      name: '',
      url: '',
      use_proxy: false,
      update_interval: 24
    }
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
    const configs = await invoke('refresh_subscription_command', {
      id,
      proxyUrl: null
    })

    message.success(`订阅更新成功，获取到 ${(configs as any).length} 个节点`)
    emit('subscriptionUpdated')
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

onMounted(() => {
  loadSubscriptions()
})
</script>
