<template>
    <nav class="navbar">
        <RouterLink to="/themes" class="navbar-tab" active-class="active">
            <span class="tab-icon"
                ><font-awesome-icon icon="paint-brush"
            /></span>
            <span class="tab-label">{{ t("nav.themes") }}</span>
        </RouterLink>
        <RouterLink
            to="/settings"
            class="navbar-tab"
            active-class="active"
        >
            <span class="tab-icon"><font-awesome-icon icon="gear" /></span>
            <span class="tab-label">{{ t("nav.settings") }}</span>
        </RouterLink>
    </nav>
</template>

<script setup lang="ts">
import { useI18n } from "vue-i18n";

const { t } = useI18n();
</script>

<style scoped>
/* 所有平台统一使用系统原生标题栏（tauri.conf.json decorations: true），
   此组件不再是自绘标题栏，仅保留应用内导航工具条。 */
.navbar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 16px;
    height: 44px;
    background-color: var(--fluent-bg-primary);
    border-bottom: 1px solid var(--fluent-border);
    flex-shrink: 0;
    user-select: none;
    -webkit-user-select: none;
}

.navbar-tab {
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

.navbar-tab:hover {
    background-color: var(--fluent-bg-secondary);
    color: var(--fluent-text-primary);
}

.navbar-tab.active {
    color: var(--fluent-accent);
    background-color: rgba(0, 120, 212, 0.08);
}

.tab-icon {
    font-size: 14px;
}

@media (max-width: 768px) {
    .navbar {
        gap: 0;
    }

    .tab-label {
        display: none;
    }
}
</style>