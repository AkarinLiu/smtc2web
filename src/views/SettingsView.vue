<template>
  <div class="settings-view">
    <h2>设置</h2>
    
    <SettingsForm
      :config="config"
      :loading="loading"
      :saved="saved"
      @save="handleSave"
    />
  </div>
</template>

<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { onMounted } from 'vue'
import { useConfigStore } from '@/stores/config'

const configStore = useConfigStore()

const { config, loading, saved } = storeToRefs(configStore)

onMounted(() => {
  configStore.loadConfig()
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
