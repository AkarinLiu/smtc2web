<template>
    <div class="themes-view">
        <header class="page-header">
            <h2>{{ t("themes.title") }}</h2>
            <div class="header-actions">
                <button
                    class="btn btn-primary"
                    @click="handleUpload"
                    :disabled="uploadLoading"
                >
                    <span v-if="uploadLoading"
                        ><font-awesome-icon icon="hourglass" />
                        {{ t("themes.uploading") }}</span
                    >
                    <span v-else
                        ><font-awesome-icon icon="upload" />
                        {{ t("themes.upload") }}</span
                    >
                </button>
                <button class="btn btn-primary" @click="handleDownload">
                    <font-awesome-icon icon="download" />
                    {{ t("themes.download") }}
                </button>
            </div>
        </header>

        <ThemeSkeleton v-if="loading" :count="6" />

        <EmptyState
            v-else-if="!hasThemes"
            :icon="['fas', 'paint-brush']"
            :title="t('themes.empty.title')"
            :description="t('themes.empty.description')"
        />

        <ThemeGrid
            v-else
            :themes="themes"
            :current-theme="currentTheme"
            @select="handleSelect"
            @delete="handleDelete"
        />
    </div>
</template>

<script setup lang="ts">
import { storeToRefs } from "pinia";
import { onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useThemeStore } from "@/stores/theme";
import { tauriInvoke } from "@/utils";
import type { Theme } from "@/types/theme";
import ThemeSkeleton from "@/components/ThemeSkeleton.vue";
import EmptyState from "@/components/EmptyState.vue";
import ThemeGrid from "@/components/ThemeGrid.vue";

const { t } = useI18n();
const themeStore = useThemeStore();

const { themes, currentTheme, loading, uploadLoading, hasThemes } =
    storeToRefs(themeStore);

onMounted(async () => {
    await Promise.all([themeStore.loadThemes(), themeStore.loadCurrentTheme()]);
});

function handleSelect(folderName: string) {
    themeStore.selectTheme(folderName);
}

function handleDelete(theme: Theme) {
    themeStore.deleteTheme(theme);
}

function handleUpload() {
    themeStore.uploadTheme();
}

function handleDownload() {
    tauriInvoke("open_url", {
        url: "https://github.com/AkarinLiu/smtc2web/discussions/categories/5-theme-%E4%B8%BB%E9%A2%98",
    });
}
</script>

<style scoped>
.themes-view {
    width: 100%;
}

.header-actions {
    display: flex;
    align-items: center;
    gap: var(--fluent-space-sm);
}

.btn {
    padding: 10px 20px;
    border: none;
    border-radius: var(--fluent-radius-md);
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: all var(--fluent-transition-fast);
    display: inline-flex;
    align-items: center;
    gap: var(--fluent-space-xs);
}

.btn-primary {
    background-color: var(--fluent-accent);
    color: var(--fluent-text-on-accent);
}

.btn-primary:hover:not(:disabled) {
    background-color: var(--fluent-accent-hover);
}

.btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
}
</style>
