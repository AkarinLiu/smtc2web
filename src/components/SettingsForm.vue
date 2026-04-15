<template>
  <div class="settings-form">
    <!-- 语言选择 -->
    <div class="form-group">
      <label>{{ t("settings.language.label") }}</label>
      <LanguageSelector />
    </div>

    <div class="form-group">
      <label>{{ t("settings.serverPort.label") }}</label>
      <input
        type="number"
        v-model.number="localConfig.server_port"
        min="1"
        max="65535"
        class="form-input"
      />
    </div>

    <div class="form-group">
      <label>{{ t("settings.serverAddress.label") }}</label>
      <input
        type="text"
        v-model="localConfig.address"
        placeholder="127.0.0.1"
        class="form-input"
      />
    </div>

    <!-- 进程过滤器 -->
    <div class="form-group">
      <label>{{ t("settings.processFilter.label") }}</label>
      <textarea
        v-model="localConfig.process_filter"
        class="form-input form-textarea"
        rows="4"
        placeholder="*"
      />
      <p class="hint">{{ t("settings.processFilter.hint") }}</p>

      <!-- 当前应用名称 -->
      <div v-if="currentAppId" class="current-app">
        <span class="current-app-label">{{ t("settings.processFilter.currentApp") }}</span>
        <code class="current-app-value">{{ currentAppId }}</code>
      </div>
    </div>

    <div class="form-actions">
      <button class="btn btn-primary" @click="handleSave" :disabled="loading">
        <span v-if="loading"><font-awesome-icon icon="spinner" spin /> {{ t("settings.saving") }}</span>
        <span v-else-if="saved"><font-awesome-icon icon="check" /> {{ t("settings.saved") }}</span>
        <span v-else><font-awesome-icon icon="floppy-disk" /> {{ t("settings.save") }}</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, watch } from "vue";
import { useI18n } from "vue-i18n";
import LanguageSelector from "./LanguageSelector.vue";
import type { AppConfig } from "@/types/config";

interface Props {
  config: AppConfig;
  loading: boolean;
  saved: boolean;
  currentAppId?: string;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  save: [];
}>();

const { t } = useI18n();

const localConfig = reactive<AppConfig>({ ...props.config });

watch(
  () => props.config,
  (newConfig: AppConfig) => {
    Object.assign(localConfig, newConfig);
  },
  { deep: true },
);

function handleSave() {
  Object.assign(props.config, localConfig);
  emit("save");
}
</script>

<style scoped>
.settings-form {
  background-color: var(--fluent-bg-card);
  padding: var(--fluent-space-lg);
  border-radius: var(--fluent-radius-lg);
  box-shadow: var(--fluent-shadow-md);
  max-width: 600px;
}

.form-group {
  margin-bottom: var(--fluent-space-lg);
}

.form-group label {
  display: block;
  font-size: 14px;
  font-weight: 600;
  margin-bottom: var(--fluent-space-sm);
  color: var(--fluent-text-primary);
}

.form-input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--fluent-border);
  border-radius: var(--fluent-radius-md);
  font-size: 14px;
  background-color: var(--fluent-bg-primary);
  color: var(--fluent-text-primary);
  transition: border-color var(--fluent-transition-fast);
}

.form-input:focus {
  outline: none;
  border-color: var(--fluent-accent);
}

.form-textarea {
  resize: vertical;
  min-height: 80px;
  font-family: monospace;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: var(--fluent-space-sm);
  cursor: pointer;
  font-weight: 500 !important;
}

.checkbox-label input[type="checkbox"] {
  width: 18px;
  height: 18px;
  cursor: pointer;
  accent-color: var(--fluent-accent);
}

.hint {
  font-size: 12px;
  color: var(--fluent-text-secondary);
  margin-top: var(--fluent-space-xs);
}

.regex-error {
  margin-top: var(--fluent-space-sm);
  padding: var(--fluent-space-sm);
  background-color: var(--fluent-error-bg, #fef2f2);
  border: 1px solid var(--fluent-error-border, #fecaca);
  border-radius: var(--fluent-radius-md);
}

.regex-error p {
  font-size: 13px;
  color: var(--fluent-error-text, #dc2626);
  margin: 0 0 var(--fluent-space-xs) 0;
  font-weight: 500;
}

.regex-error ul {
  margin: 0;
  padding-left: var(--fluent-space-lg);
  font-size: 12px;
  color: var(--fluent-error-text, #dc2626);
}

.regex-error li {
  margin-bottom: 2px;
}

.current-app {
  margin-top: var(--fluent-space-sm);
  padding: var(--fluent-space-sm) var(--fluent-space-md);
  background-color: var(--fluent-bg-secondary);
  border-radius: var(--fluent-radius-md);
  display: flex;
  align-items: center;
  gap: var(--fluent-space-sm);
}

.current-app-label {
  font-size: 12px;
  color: var(--fluent-text-secondary);
}

.current-app-value {
  font-size: 13px;
  font-family: monospace;
  color: var(--fluent-text-primary);
  background-color: var(--fluent-bg-primary);
  padding: 2px 8px;
  border-radius: var(--fluent-radius-sm);
}

.form-actions {
  margin-top: var(--fluent-space-lg);
  padding-top: var(--fluent-space-lg);
  border-top: 1px solid var(--fluent-border);
}

.btn {
  padding: 10px 24px;
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
