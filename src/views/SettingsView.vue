<template>
  <div class="settings-view">
    <h2>{{ t('settings.title') }}</h2>

    <SettingsSkeleton v-if="loading" />

    <SettingsForm
      v-else
      :config="config"
      :loading="loading"
      :saved="saved"
      :current-app-id="currentAppId"
      @save="handleSave"
    />
  </div>
</template>

<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { onMounted, nextTick, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useConfigStore } from '@/stores/config'

const { t } = useI18n()
const configStore = useConfigStore()

const { config, loading, saved, currentAppId } = storeToRefs(configStore)

let appIdInterval: number | null = null

onMounted(async () => {
  // 使用 nextTick 确保 DOM 先渲染完成
  await nextTick()
  await configStore.loadConfig()

  // 定期获取当前应用ID
  configStore.getCurrentAppId()
  appIdInterval = window.setInterval(() => {
    configStore.getCurrentAppId()
  }, 1000)
})

onUnmounted(() => {
  if (appIdInterval) {
    clearInterval(appIdInterval)
  }
})

function handleSave() {
  configStore.saveConfig()
}
</script>

<style scoped>
.settings-view {
  width: 100%;
}
</style>
