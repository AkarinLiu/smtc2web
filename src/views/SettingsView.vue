<template>
  <div class="settings-view">
    <h2>{{ t('settings.title') }}</h2>

    <SettingsSkeleton v-if="loading" />

    <SettingsForm
      v-else
      :config="config"
      :loading="loading"
      :saved="saved"
      :current-app-id="configStore.currentAppId"
      @save="handleSave"
    />
  </div>
</template>

<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useConfigStore } from '@/stores/config'
import SettingsForm from '@/components/SettingsForm.vue'
import SettingsSkeleton from '@/components/SettingsSkeleton.vue'

const { t } = useI18n()
const configStore = useConfigStore()

const { config, loading, saved } = storeToRefs(configStore)

onMounted(async () => {
  await configStore.loadConfig()
  configStore.getCurrentAppId()
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
