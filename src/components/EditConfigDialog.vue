<template>
  <n-modal
    :show="show"
    @update:show="$emit('update:show', $event)"
    preset="dialog"
    title="编辑配置"
    :mask-closable="false">
    <div v-if="config" class="space-y-4">
      <n-form
        ref="formRef"
        :model="formData"
        :rules="rules"
        label-placement="left"
        label-width="80">
        <n-form-item label="名称" path="name">
          <n-input v-model:value="formData.name" placeholder="请输入配置名称" />
        </n-form-item>

        <n-form-item label="协议" path="protocol">
          <n-select
            v-model:value="formData.protocol"
            :options="protocolOptions"
            placeholder="请选择协议" />
        </n-form-item>

        <n-form-item label="服务器" path="server">
          <n-input v-model:value="formData.server" placeholder="请输入服务器地址" />
        </n-form-item>

        <n-form-item label="端口" path="port">
          <n-input-number v-model:value="formData.port" :min="1" :max="65535" class="w-full" />
        </n-form-item>

        <n-form-item v-if="needsPassword" label="密码" path="password">
          <n-input
            v-model:value="formData.password"
            type="password"
            placeholder="请输入密码"
            show-password-on="click" />
        </n-form-item>

        <n-form-item v-if="needsMethod" label="加密方法" path="method">
          <n-select
            v-model:value="formData.method"
            :options="methodOptions"
            placeholder="请选择加密方法" />
        </n-form-item>

        <n-form-item label="备注" path="remarks">
          <n-input
            v-model:value="formData.remarks"
            type="textarea"
            placeholder="请输入备注信息"
            :rows="2" />
        </n-form-item>
      </n-form>
    </div>

    <template #action>
      <n-space>
        <n-button @click="emit('update:show', false)">取消</n-button>
        <n-button type="primary" @click="saveConfig" :loading="saving">保存</n-button>
      </n-space>
    </template>
  </n-modal>
</template>

<script lang="ts" setup>
import { ref, watch, computed } from 'vue'
import {
  NModal,
  NForm,
  NFormItem,
  NInput,
  NSelect,
  NInputNumber,
  NButton,
  NSpace,
  useMessage,
  type FormInst,
  type FormRules
} from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'
import type { VpnConfig } from '../types/vpn'

const props = defineProps<{
  show: boolean
  config: VpnConfig | null
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
  saved: [config: VpnConfig]
}>()

const message = useMessage()
const formRef = ref<FormInst>()
const saving = ref(false)

const formData = ref({
  name: '',
  protocol: '',
  server: '',
  port: 0,
  password: '',
  method: '',
  remarks: ''
})

// 协议选项
const protocolOptions = [
  { label: 'Shadowsocks', value: 'Shadowsocks' },
  { label: 'ShadowsocksR', value: 'ShadowsocksR' },
  { label: 'V2Ray', value: 'V2Ray' },
  { label: 'VMess', value: 'Vmess' },
  { label: 'Trojan', value: 'Trojan' }
]

// 加密方法选项
const methodOptions = [
  { label: 'aes-256-gcm', value: 'aes-256-gcm' },
  { label: 'aes-128-gcm', value: 'aes-128-gcm' },
  { label: 'chacha20-ietf-poly1305', value: 'chacha20-ietf-poly1305' },
  { label: 'xchacha20-ietf-poly1305', value: 'xchacha20-ietf-poly1305' },
  { label: 'aes-256-cfb', value: 'aes-256-cfb' },
  { label: 'aes-128-cfb', value: 'aes-128-cfb' },
  { label: 'chacha20-ietf', value: 'chacha20-ietf' },
  { label: 'rc4-md5', value: 'rc4-md5' }
]

// 是否需要密码
const needsPassword = computed(() => {
  return ['Shadowsocks', 'ShadowsocksR', 'Vmess', 'Trojan'].includes(formData.value.protocol)
})

// 是否需要加密方法
const needsMethod = computed(() => {
  return ['Shadowsocks', 'ShadowsocksR'].includes(formData.value.protocol)
})

// 表单验证规则
const rules: FormRules = {
  name: [{ required: true, message: '请输入配置名称', trigger: 'blur' }],
  protocol: [{ required: true, message: '请选择协议', trigger: 'change' }],
  server: [{ required: true, message: '请输入服务器地址', trigger: 'blur' }],
  port: [{ required: true, type: 'number', message: '请输入端口号', trigger: 'blur' }],
  password: [
    {
      required: true,
      message: '请输入密码',
      trigger: 'blur',
      validator: (_rule, value) => {
        if (needsPassword.value && !value) {
          return new Error('请输入密码')
        }
        return true
      }
    }
  ],
  method: [
    {
      required: true,
      message: '请选择加密方法',
      trigger: 'change',
      validator: (_rule, value) => {
        if (needsMethod.value && !value) {
          return new Error('请选择加密方法')
        }
        return true
      }
    }
  ]
}

// 监听配置变化，填充表单
watch(
  [() => props.config, () => props.show],
  ([config, show]) => {
    if (config && show) {
      formData.value = {
        name: config.name,
        protocol: config.protocol,
        server: config.server,
        port: config.port,
        password: config.password || '',
        method: config.method || '',
        remarks: config.remarks || ''
      }
    }
  },
  { immediate: true }
)

// 保存配置
const saveConfig = async () => {
  if (!props.config) return

  try {
    await formRef.value?.validate()
    saving.value = true

    // 加载现有配置
    const result = await invoke('load_configs_command')
    const configs = (result as any).configs || []

    // 更新配置
    const updatedConfig: VpnConfig = {
      ...props.config,
      name: formData.value.name,
      protocol: formData.value.protocol as any,
      server: formData.value.server,
      port: formData.value.port,
      password: formData.value.password || undefined,
      method: formData.value.method || undefined,
      remarks: formData.value.remarks || undefined,
      updated_at: new Date().toISOString()
    }

    // 找到并更新配置
    const index = configs.findIndex((c: VpnConfig) => c.id === props.config!.id)
    if (index !== -1) {
      configs[index] = updatedConfig
    }

    // 保存配置
    await invoke('save_configs_command', { configs: { configs } })

    emit('saved', updatedConfig)
    emit('update:show', false)
    message.success('配置保存成功')
  } catch (error) {
    if (error instanceof Error) {
      message.error(`保存配置失败: ${error.message}`)
    } else {
      message.error(`保存配置失败: ${error}`)
    }
  } finally {
    saving.value = false
  }
}
</script>
