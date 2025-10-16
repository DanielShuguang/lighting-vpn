<template>
  <n-modal
    v-model:show="showModal"
    preset="card"
    title="V2Ray 核心管理"
    :style="{ width: '90vw', maxWidth: '500px' }"
    :mask-closable="!downloading">
    <div class="space-y-4">
      <!-- 核心状态 -->
      <div v-if="coreInfo" class="space-y-3">
        <!-- 已安装 -->
        <div v-if="coreInfo.installed" class="space-y-3">
          <!-- 有更新提示 -->
          <n-alert v-if="coreInfo.has_update" type="warning" title="发现新版本">
            <div class="space-y-1">
              <div>当前版本：{{ coreInfo.version || '未知' }}</div>
              <div>最新版本：{{ coreInfo.latest_version }}</div>
              <div class="text-xs text-gray-500 mt-2">
                建议更新到最新版本以获得更好的性能和安全性
              </div>
            </div>
          </n-alert>

          <!-- 已是最新版本 -->
          <n-alert v-else type="success" title="核心已安装">
            <div class="space-y-1">
              <div>当前版本：{{ coreInfo.version || '未知' }}</div>
              <div v-if="coreInfo.latest_version" class="text-xs text-green-600">
                ✓ 已是最新版本
              </div>
              <div class="text-xs text-gray-500 break-all">路径：{{ coreInfo.path }}</div>
              <div class="text-xs text-gray-500">平台：{{ coreInfo.platform }}</div>
            </div>
          </n-alert>

          <div class="flex flex-wrap gap-2">
            <!-- 如果有更新，显示更新按钮 -->
            <n-button
              v-if="coreInfo.has_update"
              type="primary"
              @click="handleUpdate"
              :disabled="downloading"
              :loading="downloading">
              <template #icon>
                <n-icon><CloudDownloadIcon /></n-icon>
              </template>
              更新到 {{ coreInfo.latest_version }}
            </n-button>

            <!-- 重新下载按钮 -->
            <n-button type="warning" @click="handleRedownload" :disabled="downloading">
              <template #icon>
                <n-icon><RefreshIcon /></n-icon>
              </template>
              重新下载
            </n-button>

            <!-- 检查更新按钮 -->
            <n-button @click="handleCheckUpdate" :disabled="downloading">
              <template #icon>
                <n-icon><SyncIcon /></n-icon>
              </template>
              检查更新
            </n-button>

            <!-- 删除核心按钮 -->
            <n-button type="error" @click="handleRemove" :disabled="downloading">
              <template #icon>
                <n-icon><TrashIcon /></n-icon>
              </template>
              删除核心
            </n-button>
          </div>
        </div>

        <!-- 未安装 -->
        <div v-else class="space-y-3">
          <n-alert type="warning" title="未安装核心">
            <div>
              需要下载 V2Ray 核心才能使用 VPN 功能。
              <br />
              <span class="text-xs text-gray-500">平台：{{ coreInfo.platform }}</span>
            </div>
          </n-alert>

          <n-button
            type="primary"
            block
            @click="handleDownload"
            :disabled="downloading"
            :loading="downloading">
            <template #icon>
              <n-icon><DownloadIcon /></n-icon>
            </template>
            下载 V2Ray 核心
          </n-button>
        </div>
      </div>

      <!-- 加载中 -->
      <div v-else class="flex justify-center py-4">
        <n-spin size="medium" />
      </div>

      <!-- 下载进度 -->
      <div v-if="downloading" class="space-y-2">
        <n-progress
          type="line"
          :percentage="progress.percentage"
          :status="progress.percentage === 100 ? 'success' : 'default'"
          :show-indicator="false" />
        <div class="text-sm text-gray-600 text-center space-y-1">
          <div>{{ formatBytes(progress.downloaded) }} / {{ formatBytes(progress.total) }}</div>
          <div v-if="progress.percentage < 100" class="font-medium">
            {{ progress.percentage.toFixed(1) }}%
          </div>
          <div v-else class="text-blue-600">正在解压...</div>
        </div>
      </div>

      <!-- 说明信息 -->
      <n-alert type="info" title="温馨提示" :bordered="false">
        <ul class="text-sm space-y-1 list-disc list-inside">
          <li>首次使用需要下载 V2Ray 核心程序</li>
          <li>核心程序约 10-20 MB，下载时间取决于网络速度</li>
          <li>核心程序会自动保存到应用目录</li>
          <li>可以随时重新下载或删除核心</li>
        </ul>
      </n-alert>
    </div>

    <template #footer>
      <div class="flex justify-end">
        <n-button @click="showModal = false" :disabled="downloading">关闭</n-button>
      </div>
    </template>
  </n-modal>
</template>

<script lang="ts" setup>
import { computed, onMounted, watch } from 'vue'
import { NModal, NButton, NIcon, NAlert, NProgress, NSpin, useMessage, useDialog } from 'naive-ui'
import {
  Download as DownloadIcon,
  Refresh as RefreshIcon,
  Trash as TrashIcon,
  CloudDownload as CloudDownloadIcon,
  Sync as SyncIcon
} from '@vicons/ionicons5'
import { useCoreManager } from '../composables/useCoreManager'

const props = defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
}>()

const message = useMessage()
const dialog = useDialog()

const showModal = computed({
  get: () => props.show,
  set: value => emit('update:show', value)
})

const { coreInfo, downloading, progress, checkCore, downloadCore, removeCore, formatBytes } =
  useCoreManager()

/**
 * 处理下载核心 - 始终下载最新版本
 */
const handleDownload = async () => {
  try {
    // 先获取最新版本信息
    await checkCore()

    // 使用最新版本进行下载
    const version = coreInfo.value?.latest_version || undefined
    if (!version) {
      message.warning('无法获取最新版本信息，将下载默认版本')
    } else {
      message.info(`开始下载最新版本 ${version}`)
    }

    await downloadCore(version)
  } catch (error) {
    // 错误已在 composable 中处理
  }
}

/**
 * 处理更新
 */
const handleUpdate = async () => {
  if (!coreInfo.value?.latest_version) {
    message.error('无法获取最新版本信息')
    return
  }

  dialog.info({
    title: '更新核心',
    content: `确定要更新到 ${coreInfo.value.latest_version} 吗？`,
    positiveText: '确定',
    negativeText: '取消',
    onPositiveClick: () => {
      // 不使用 async，让弹窗立即关闭
      downloadCore(coreInfo.value?.latest_version || undefined).catch(() => {
        // 错误已在 composable 中处理
      })
    }
  })
}

/**
 * 处理检查更新
 */
const handleCheckUpdate = async () => {
  try {
    message.info('正在检查更新...')
    await checkCore()
    if (coreInfo.value?.has_update) {
      message.success(`发现新版本 ${coreInfo.value.latest_version}！`)
    } else {
      message.success('当前已是最新版本')
    }
  } catch (error) {
    message.error(`检查更新失败: ${error}`)
  }
}

/**
 * 处理重新下载
 */
const handleRedownload = () => {
  dialog.warning({
    title: '重新下载核心',
    content: '确定要重新下载 V2Ray 核心吗？这将覆盖现有版本。',
    positiveText: '确定',
    negativeText: '取消',
    onPositiveClick: () => {
      // 不使用 async，让弹窗立即关闭
      downloadCore().catch(() => {
        // 错误已在 composable 中处理
      })
    }
  })
}

/**
 * 处理删除
 */
const handleRemove = () => {
  dialog.error({
    title: '删除核心',
    content: '确定要删除 V2Ray 核心吗？删除后将无法使用 VPN 功能。',
    positiveText: '确定删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await removeCore()
      } catch (error) {
        // 错误已在 composable 中处理
      }
    }
  })
}

// 监听对话框打开，自动检查核心状态
watch(
  () => props.show,
  async newValue => {
    if (newValue) {
      await checkCore()
    }
  }
)

// 组件挂载时检查核心状态
onMounted(async () => {
  if (props.show) {
    await checkCore()
  }
})
</script>
