<template>
  <div class="language-selector">
    <select 
      :value="localeStore.currentLocale" 
      @change="handleChange"
      class="lang-select"
    >
      <option 
        v-for="lang in localeStore.availableLocales" 
        :key="lang.code" 
        :value="lang.code"
      >
        {{ lang.name }}
      </option>
    </select>
  </div>
</template>

<script setup lang="ts">
import { useLocaleStore } from '@/stores/locale'

const localeStore = useLocaleStore()

function handleChange(event: Event) {
  const target = event.target as HTMLSelectElement
  localeStore.setLocale(target.value)
}
</script>

<style scoped>
.language-selector {
  display: inline-block;
}

.lang-select {
  padding: 8px 12px;
  border: 1px solid var(--fluent-border);
  border-radius: var(--fluent-radius-md);
  background-color: var(--fluent-bg-primary);
  color: var(--fluent-text-primary);
  font-size: 14px;
  cursor: pointer;
  transition: border-color var(--fluent-transition-fast);
}

.lang-select:focus {
  outline: none;
  border-color: var(--fluent-accent);
}

.lang-select option {
  background-color: var(--fluent-bg-primary);
  color: var(--fluent-text-primary);
}
</style>
