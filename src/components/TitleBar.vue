<template>
    <div data-tauri-drag-region class="titlebar">
        <div class="titlebar-left">
            <span class="titlebar-icon"
                ><font-awesome-icon icon="music"
            /></span>
            <span class="titlebar-title">smtc2web</span>
        </div>
        <nav class="titlebar-tabs">
            <RouterLink to="/themes" class="titlebar-tab" active-class="active">
                <span class="tab-icon"
                    ><font-awesome-icon icon="paint-brush"
                /></span>
                <span class="tab-label">{{ t("nav.themes") }}</span>
            </RouterLink>
            <RouterLink
                to="/settings"
                class="titlebar-tab"
                active-class="active"
            >
                <span class="tab-icon"><font-awesome-icon icon="gear" /></span>
                <span class="tab-label">{{ t("nav.settings") }}</span>
            </RouterLink>
        </nav>
        <div class="titlebar-controls">
            <button
                class="titlebar-btn"
                @mousedown.prevent="minimize"
                :title="t('titlebar.minimize')"
            >
                <svg width="12" height="12" viewBox="0 0 12 12">
                    <rect
                        x="0"
                        y="5.5"
                        width="12"
                        height="1"
                        fill="currentColor"
                    />
                </svg>
            </button>
            <button
                class="titlebar-btn"
                @mousedown.prevent="toggleMaximize"
                :title="
                    isMaximized ? t('titlebar.restore') : t('titlebar.maximize')
                "
            >
                <svg
                    v-if="!isMaximized"
                    width="12"
                    height="12"
                    viewBox="0 0 12 12"
                >
                    <rect
                        x="1"
                        y="1"
                        width="10"
                        height="10"
                        stroke="currentColor"
                        stroke-width="1"
                        fill="none"
                    />
                </svg>
                <svg v-else width="12" height="12" viewBox="0 0 12 12">
                    <rect
                        x="3"
                        y="0"
                        width="9"
                        height="9"
                        stroke="currentColor"
                        stroke-width="1"
                        fill="none"
                    />
                    <rect
                        x="0"
                        y="3"
                        width="9"
                        height="9"
                        stroke="currentColor"
                        stroke-width="1"
                        fill="var(--fluent-bg-primary)"
                    />
                </svg>
            </button>
            <button
                class="titlebar-btn titlebar-btn-close"
                @mousedown.prevent="closeWindow"
                :title="t('titlebar.close')"
            >
                <svg width="12" height="12" viewBox="0 0 12 12">
                    <line
                        x1="0"
                        y1="0"
                        x2="12"
                        y2="12"
                        stroke="currentColor"
                        stroke-width="1.2"
                    />
                    <line
                        x1="12"
                        y1="0"
                        x2="0"
                        y2="12"
                        stroke="currentColor"
                        stroke-width="1.2"
                    />
                </svg>
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { hasTauri, tauriInvoke } from "@/utils";

const { t } = useI18n();

const isMaximized = ref(false);

async function initWindow() {
    if (!hasTauri()) return;
    try {
        isMaximized.value = await tauriInvoke<boolean>("window_is_maximized");
    } catch {
        /* ignore polling failure */
    }
}

async function minimize() {
    try { await tauriInvoke("window_minimize"); } catch { /* ignore */ }
}

async function toggleMaximize() {
    try { await tauriInvoke("window_toggle_maximize"); } catch { /* ignore */ }
}

async function closeWindow() {
    try { await tauriInvoke("window_close"); } catch { /* ignore */ }
}

onMounted(() => { initWindow(); });
</script>

<style scoped>
.titlebar {
    display: flex;
    align-items: center;
    height: 44px;
    background-color: var(--fluent-bg-primary);
    border-bottom: 1px solid var(--fluent-border);
    flex-shrink: 0;
    user-select: none;
    -webkit-user-select: none;
    -webkit-app-region: drag;
}

.titlebar-left {
    display: flex;
    align-items: center;
    gap: var(--fluent-space-xs);
    padding-left: 16px;
    flex-shrink: 0;
}

.titlebar-icon {
    font-size: 16px;
    color: var(--fluent-accent);
}

.titlebar-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--fluent-text-primary);
}

/* Navigation tabs */
.titlebar-tabs {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-left: 24px;
    height: 100%;
    -webkit-app-region: no-drag;
}

.titlebar-tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px;
    height: 32px;
    border: none;
    background: transparent;
    color: var(--fluent-text-secondary);
    font-size: 13px;
    font-weight: 500;
    text-decoration: none;
    border-radius: var(--fluent-radius-md);
    transition: all var(--fluent-transition-fast);
    cursor: pointer;
}

.titlebar-tab:hover {
    background-color: var(--fluent-bg-secondary);
    color: var(--fluent-text-primary);
}

.titlebar-tab.active {
    color: var(--fluent-accent);
    background-color: rgba(0, 120, 212, 0.08);
}

.tab-icon {
    font-size: 14px;
}

/* Window controls */
.titlebar-controls {
    display: flex;
    height: 100%;
    margin-left: auto;
    -webkit-app-region: no-drag;
}

.titlebar-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 46px;
    height: 100%;
    border: none;
    background: transparent;
    color: var(--fluent-text-secondary);
    cursor: pointer;
    transition: background-color var(--fluent-transition-fast);
    -webkit-app-region: no-drag;
}

.titlebar-btn:hover {
    background-color: var(--fluent-bg-secondary);
}

.titlebar-btn-close:hover {
    background-color: #e81123;
    color: #ffffff;
}

@media (max-width: 768px) {
    .titlebar-title {
        display: none;
    }

    .titlebar-tabs {
        margin-left: 8px;
        gap: 0;
    }

    .tab-label {
        display: none;
    }
}
</style>
