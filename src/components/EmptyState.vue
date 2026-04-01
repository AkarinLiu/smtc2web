<template>
  <div class="empty-state">
    <div class="empty-icon"><font-awesome-icon :icon="icon" /></div>
    <h3 class="empty-title">{{ title }}</h3>
    <p class="empty-description">{{ description }}</p>
  </div>
</template>

<script setup lang="ts">
interface Props {
  icon: string | string[] | [string, string]
  title: string
  description: string
}

defineProps<Props>()
</script>

<style scoped>
.empty-state {
  text-align: center;
  padding: 60px 20px;
  color: var(--fluent-text-secondary);
}

.empty-icon {
  font-size: 48px;
  margin-bottom: var(--fluent-space-md);
  opacity: 0.6;
  display: flex;
  justify-content: center;
}

.empty-title {
  font-size: 18px;
  font-weight: 600;
  margin-bottom: var(--fluent-space-sm);
  color: var(--fluent-text-primary);
}

.empty-description {
  font-size: 14px;
  color: var(--fluent-text-secondary);
}
</style>
