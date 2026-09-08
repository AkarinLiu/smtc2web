<template>
  <select
    class="lang-select"
    :value="localeStore.currentLocale"
    @change="onChange"
  >
    <option
      v-for="lang in localeStore.availableLocales"
      :key="lang.code"
      :value="lang.code"
    >
      {{ lang.name }}
    </option>
  </select>
</template>

<script setup lang="ts">
import { useLocaleStore } from '@/stores/locale'

const localeStore = useLocaleStore()

function onChange(e: Event) {
  localeStore.setLocale((e.target as HTMLSelectElement).value)
}
</script>

<style scoped>
.lang-select {
  appearance: none;
  -webkit-appearance: none;
  padding: 8px 32px 8px 12px;
  min-width: 160px;
  border: 1px solid var(--ui-border);
  border-radius: var(--ui-radius-md);
  background-color: var(--ui-bg-primary);
  color: var(--ui-text-primary);
  font-size: 14px;
  cursor: pointer;
  transition: border-color var(--ui-transition-fast);
  background-image: var(--ui-select-chevron);
  background-repeat: no-repeat;
  background-position: right 12px center;
}

.lang-select:focus {
  outline: none;
  border-color: var(--ui-accent);
}
</style>
