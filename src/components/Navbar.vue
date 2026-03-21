<template>
  <nav class="navbar">
    <div class="navbar-brand">
      <span class="brand-icon">🎵</span>
      <span class="brand-text">smtc2web</span>
    </div>
    <div class="navbar-tabs">
      <RouterLink to="/themes" class="nav-tab" active-class="active">
        <span class="tab-icon">🎨</span>
        {{ t('nav.themes') }}
      </RouterLink>
      <RouterLink to="/settings" class="nav-tab" active-class="active">
        <span class="tab-icon">⚙️</span>
        {{ t('nav.settings') }}
      </RouterLink>
    </div>
  </nav>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>

<style scoped>
.navbar {
  background-color: var(--fluent-bg-primary);
  border-bottom: 1px solid var(--fluent-border);
  padding: 0 var(--fluent-space-lg);
  display: flex;
  align-items: center;
  height: 56px;
  box-shadow: var(--fluent-shadow-sm);
  flex-shrink: 0;
}

.navbar-brand {
  display: flex;
  align-items: center;
  gap: var(--fluent-space-sm);
  margin-right: var(--fluent-space-xl);
}

.brand-icon {
  font-size: 24px;
}

.brand-text {
  font-size: 20px;
  font-weight: 600;
  color: var(--fluent-text-primary);
}

.navbar-tabs {
  display: flex;
  gap: var(--fluent-space-xs);
}

.nav-tab {
  display: flex;
  align-items: center;
  gap: var(--fluent-space-xs);
  padding: var(--fluent-space-sm) var(--fluent-space-md);
  border: none;
  background: transparent;
  color: var(--fluent-text-secondary);
  font-size: 14px;
  font-weight: 500;
  text-decoration: none;
  border-radius: var(--fluent-radius-md);
  transition: all var(--fluent-transition-fast);
  cursor: pointer;
}

.nav-tab:hover {
  background-color: var(--fluent-bg-secondary);
  color: var(--fluent-text-primary);
}

.nav-tab.active {
  color: var(--fluent-accent);
  background-color: rgba(0, 120, 212, 0.1);
}

.tab-icon {
  font-size: 16px;
}

@media (max-width: 768px) {
  .navbar {
    padding: 0 var(--fluent-space-md);
  }
  
  .brand-text {
    font-size: 18px;
  }
  
  .tab-icon {
    display: none;
  }
}
</style>
