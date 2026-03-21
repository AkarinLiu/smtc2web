<template>
  <div class="settings-skeleton">
    <div class="skeleton-section" v-for="i in sections" :key="i">
      <div class="skeleton-header"></div>
      <div class="skeleton-input" v-for="j in inputs" :key="j"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  sections?: number
  inputs?: number
}

withDefaults(defineProps<Props>(), {
  sections: 2,
  inputs: 3
})
</script>

<style scoped>
.settings-skeleton {
  display: flex;
  flex-direction: column;
  gap: var(--fluent-space-lg);
  max-width: 600px;
}

.skeleton-section {
  background-color: var(--fluent-bg-card);
  border-radius: var(--fluent-radius-lg);
  padding: var(--fluent-space-lg);
  box-shadow: var(--fluent-shadow-md);
  display: flex;
  flex-direction: column;
  gap: var(--fluent-space-md);
}

.skeleton-header {
  height: 24px;
  width: 40%;
  background: linear-gradient(
    90deg,
    var(--fluent-bg-secondary) 25%,
    var(--fluent-bg-tertiary) 50%,
    var(--fluent-bg-secondary) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
  border-radius: var(--fluent-radius-sm);
}

.skeleton-input {
  height: 40px;
  width: 100%;
  background: linear-gradient(
    90deg,
    var(--fluent-bg-secondary) 25%,
    var(--fluent-bg-tertiary) 50%,
    var(--fluent-bg-secondary) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
  border-radius: var(--fluent-radius-md);
}

@keyframes shimmer {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}
</style>
