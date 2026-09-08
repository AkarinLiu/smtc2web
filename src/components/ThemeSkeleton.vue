<template>
  <div class="skeleton-grid">
    <div v-for="i in count" :key="i" class="skeleton-card">
      <div class="skeleton-preview"></div>
      <div class="skeleton-info">
        <div class="skeleton-title"></div>
        <div class="skeleton-meta"></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  count?: number
}

withDefaults(defineProps<Props>(), {
  count: 6
})
</script>

<style scoped>
.skeleton-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--ui-space-lg);
}

.skeleton-card {
  background-color: var(--ui-bg-card);
  border-radius: var(--ui-radius-lg);
  overflow: hidden;
  box-shadow: var(--ui-shadow-md);
}

.skeleton-preview {
  width: 100%;
  aspect-ratio: 16 / 9;
  background: linear-gradient(
    90deg,
    var(--ui-bg-secondary) 25%,
    var(--ui-bg-tertiary) 50%,
    var(--ui-bg-secondary) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

.skeleton-info {
  padding: var(--ui-space-md);
  display: flex;
  flex-direction: column;
  gap: var(--ui-space-sm);
}

.skeleton-title {
  height: 20px;
  width: 70%;
  background: linear-gradient(
    90deg,
    var(--ui-bg-secondary) 25%,
    var(--ui-bg-tertiary) 50%,
    var(--ui-bg-secondary) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
  border-radius: var(--ui-radius-sm);
}

.skeleton-meta {
  height: 16px;
  width: 50%;
  background: linear-gradient(
    90deg,
    var(--ui-bg-secondary) 25%,
    var(--ui-bg-tertiary) 50%,
    var(--ui-bg-secondary) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
  border-radius: var(--ui-radius-sm);
}

@keyframes shimmer {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}

@media (max-width: 768px) {
  .skeleton-grid {
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: var(--ui-space-md);
  }
}

@media (max-width: 480px) {
  .skeleton-grid {
    grid-template-columns: 1fr;
  }
}
</style>
