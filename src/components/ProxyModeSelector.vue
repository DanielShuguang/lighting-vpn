<template>
  <div class="proxy-mode-selector flex items-center gap-2">
    <n-select
      v-model:value="selectedMode"
      :options="modeOptions"
      :loading="loading"
      :disabled="disabled"
      size="small"
      @update:value="handleModeChange"
      class="w-full sm:w-48"></n-select>

    <!-- PAC管理按钮 -->
    <n-dropdown
      v-if="selectedMode === 'pac'"
      trigger="click"
      :options="pacMenuOptions"
      @select="handlePacMenuSelect"
      placement="bottom-end">
      <n-button size="small" quaternary circle :loading="pacLoading">
        <template #icon>
          <n-icon><SettingsIcon /></n-icon>
        </template>
      </n-button>
    </n-dropdown>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref, h } from 'vue'
import { NSelect, NIcon, NButton, NDropdown, useMessage, useDialog } from 'naive-ui'
import {
  Settings as SettingsIcon,
  Download as DownloadIcon,
  Refresh as RefreshIcon
} from '@vicons/ionicons5'
import { invoke } from '@tauri-apps/api/core'
import { useProxyMode } from '../composables/useProxyMode'
import { PROXY_MODES } from '../types/vpn'
import type { ProxyMode } from '../types/vpn'
import type { DropdownOption } from 'naive-ui'

interface Props {
  disabled?: boolean
}

defineProps<Props>()

const { currentMode, loading, setProxyMode } = useProxyMode()
const message = useMessage()
const dialog = useDialog()
const pacLoading = ref(false)

// 将当前模式绑定到选择器
const selectedMode = computed({
  get: () => currentMode.value,
  set: (value: ProxyMode) => {
    currentMode.value = value
  }
})

// 生成选项
const modeOptions = computed(() => {
  return PROXY_MODES.map(mode => ({
    label: mode.label,
    value: mode.mode,
    description: mode.description
  }))
})

/**
 * 处理模式切换
 */
const handleModeChange = async (mode: ProxyMode) => {
  try {
    await setProxyMode(mode)
  } catch (error) {
    console.error('切换代理模式失败:', error)
  }
}

// PAC菜单选项
const pacMenuOptions = computed<DropdownOption[]>(() => [
  {
    label: '更新 PAC 文件',
    key: 'update-pac',
    icon: () => h(NIcon, null, { default: () => h(RefreshIcon) })
  },
  {
    label: '下载 GFWList',
    key: 'download-gfwlist',
    icon: () => h(NIcon, null, { default: () => h(DownloadIcon) })
  }
])

// 处理PAC菜单选择
const handlePacMenuSelect = (key: string) => {
  if (key === 'update-pac') {
    handleUpdatePac()
  } else if (key === 'download-gfwlist') {
    handleDownloadGfwlist()
  }
}

// 更新PAC文件
const handleUpdatePac = async () => {
  dialog.info({
    title: '更新 PAC 文件',
    content:
      '是否重新生成 PAC 文件？这将根据当前的规则文件（gfwlist.txt 或 pac_rules.txt）重新生成代理规则。',
    positiveText: '更新',
    negativeText: '取消',
    onPositiveClick: async () => {
      pacLoading.value = true
      try {
        const result = await invoke<string>('update_pac_command')
        message.success(result)
      } catch (error) {
        message.error(`更新失败: ${error}`)
      } finally {
        pacLoading.value = false
      }
    }
  })
}

// 下载GFWList
const handleDownloadGfwlist = async () => {
  dialog.info({
    title: '下载 GFWList',
    content: '将从 GitHub 下载最新的 GFWList 规则文件。下载完成后需要重新更新 PAC 文件才能生效。',
    positiveText: '下载',
    negativeText: '取消',
    onPositiveClick: async () => {
      pacLoading.value = true
      message.loading('正在下载 GFWList...', { duration: 0 })
      try {
        const result = await invoke<string>('download_gfwlist_command')
        message.destroyAll()
        message.success(result)

        // 自动更新PAC文件
        dialog.info({
          title: '下载成功',
          content: '是否立即更新 PAC 文件以应用新规则？',
          positiveText: '更新',
          negativeText: '稍后',
          onPositiveClick: async () => {
            try {
              await invoke<string>('update_pac_command')
              message.success('PAC 文件已更新')
            } catch (error) {
              message.error(`更新失败: ${error}`)
            }
          }
        })
      } catch (error) {
        message.destroyAll()
        message.error(`下载失败: ${error}`)
      } finally {
        pacLoading.value = false
      }
    }
  })
}
</script>

<style scoped>
.proxy-mode-selector {
  display: inline-flex;
}
</style>
