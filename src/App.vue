<script lang="ts" setup>
import { ref } from 'vue'
import { NConfigProvider, NMessageProvider, NDialogProvider, NGlobalStyle } from 'naive-ui'
import { zhCN } from 'naive-ui'
import MainInterface from './components/MainInterface.vue'

const theme = ref(null)
</script>

<template>
  <n-config-provider :locale="zhCN" :theme="theme">
    <n-global-style />
    <n-message-provider>
      <n-dialog-provider>
        <MainInterface />
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<style>
html,
body {
  margin: 0;
  padding: 0;
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

#app {
  height: 100vh;
  width: 100vw;
}
</style>
