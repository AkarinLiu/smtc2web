<template>
    <div id="app">
        <TitleBar />
        <main class="page">
            <RouterView />
        </main>
        <Toast />
        <UpdateDialog />
    </div>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { useConfigStore } from "@/stores/config";
import { useLocaleStore } from "@/stores/locale";
import { useUpdateStore } from "@/stores/update";
import Toast from "@/components/Toast.vue";
import UpdateDialog from "@/components/UpdateDialog.vue";

const configStore = useConfigStore();
const localeStore = useLocaleStore();
const updateStore = useUpdateStore();

onMounted(async () => {
    // 首先加载配置（包含语言设置）
    await configStore.loadConfig();

    // 然后初始化语言（使用从后端加载的配置）
    localeStore.initLocale();

    // 启动后自动检查更新
    if (configStore.config.auto_check_update) {
        setTimeout(() => {
            updateStore.checkForUpdates();
        }, 1000);
    }

    // 监听系统托盘发起的检查更新事件
    if (typeof window !== "undefined" && window.__TAURI__) {
        import("@tauri-apps/api/event").then(({ listen }) => {
            listen("check-update", () => {
                updateStore.checkForUpdates();
            });
        });
    }
});
</script>

<style>
@import "./styles/global.css";

#app {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
}
</style>
