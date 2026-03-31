<template>
  <div id="app">
    <Navbar />
    <main class="page">
      <RouterView />
    </main>
    <Toast />
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useConfigStore } from '@/stores/config'
import { useLocaleStore } from '@/stores/locale'
import Toast from '@/components/Toast.vue'

const configStore = useConfigStore()
const localeStore = useLocaleStore()

onMounted(async () => {
  // 首先加载配置（包含语言设置）
  await configStore.loadConfig()
  
  // 然后初始化语言（使用从后端加载的配置）
  localeStore.initLocale()
})
</script>

<style>
@import './styles/global.css';

#app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}
</style>
