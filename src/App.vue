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
import { hasTauri } from "@/utils";

const configStore = useConfigStore();
const localeStore = useLocaleStore();
const updateStore = useUpdateStore();

onMounted(async () => {
    await configStore.loadConfig();
    localeStore.initLocale();

    if (configStore.config.auto_check_update) {
        setTimeout(() => updateStore.checkForUpdates(), 1000);
    }

    if (hasTauri()) {
        import("@tauri-apps/api/event").then(({ listen }) => {
            listen("check-update", () => updateStore.checkForUpdates());
        });
    }
});
</script>

<style>

#app {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
}
</style>
