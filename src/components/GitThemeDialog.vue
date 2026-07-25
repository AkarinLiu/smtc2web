<template>
  <Teleport to="body">
    <Transition name="dialog-fade">
      <div v-if="visible" class="dialog-overlay" @click.self="$emit('close')">
        <div class="dialog">
          <div class="dialog-header">
            <h3>{{ t("themes.git.dialog.title") }}</h3>
            <button class="close-btn" @click="$emit('close')">
              <font-awesome-icon icon="times" />
            </button>
          </div>
          <div class="dialog-body">
            <div class="form-group">
              <label>{{ t("themes.git.dialog.url") }}</label>
              <input
                v-model="repoUrl"
                type="url"
                :placeholder="t('themes.git.dialog.urlPlaceholder')"
                :disabled="installing"
                @keydown.enter="handleInstall"
              />
            </div>
            <div class="form-group">
              <label>{{ t("themes.git.dialog.branch") }}</label>
              <input
                v-model="branch"
                type="text"
                :placeholder="t('themes.git.dialog.branchPlaceholder')"
                :disabled="installing"
                @keydown.enter="handleInstall"
              />
            </div>
          </div>
          <div class="dialog-footer">
            <button
              class="btn btn-primary"
              :disabled="!repoUrl.trim() || installing"
              @click="handleInstall"
            >
              <font-awesome-icon v-if="installing" icon="spinner" spin />
              <font-awesome-icon v-else icon="code-branch" />
              {{
                installing
                  ? t("themes.git.dialog.installing")
                  : t("themes.git.dialog.installBtn")
              }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";

const { t } = useI18n();

defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  close: [];
  install: [repoUrl: string, branch: string];
}>();

const repoUrl = ref("");
const branch = ref("");
const installing = ref(false);

async function handleInstall() {
  if (!repoUrl.value.trim() || installing.value) return;
  installing.value = true;
  try {
    emit("install", repoUrl.value.trim(), branch.value.trim());
  } finally {
    installing.value = false;
  }
}

defineExpose({ installing });
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.dialog {
  background-color: var(--fluent-bg-card);
  border-radius: var(--fluent-radius-lg);
  box-shadow: var(--fluent-shadow-lg);
  width: 90%;
  max-width: 480px;
  overflow: hidden;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--fluent-space-md) var(--fluent-space-lg);
  border-bottom: 1px solid var(--fluent-border);
}

.dialog-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--fluent-text-primary);
}

.close-btn {
  background: none;
  border: none;
  color: var(--fluent-text-secondary);
  cursor: pointer;
  font-size: 16px;
  padding: var(--fluent-space-xs);
  border-radius: var(--fluent-radius-sm);
  transition: background-color var(--fluent-transition-fast);
}

.close-btn:hover {
  background-color: var(--fluent-bg-secondary);
}

.dialog-body {
  padding: var(--fluent-space-lg);
  display: flex;
  flex-direction: column;
  gap: var(--fluent-space-md);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--fluent-space-xs);
}

.form-group label {
  font-size: 13px;
  font-weight: 600;
  color: var(--fluent-text-primary);
}

.form-group input {
  padding: 10px 12px;
  border: 1px solid var(--fluent-border);
  border-radius: var(--fluent-radius-md);
  font-size: 14px;
  background-color: var(--fluent-bg-primary);
  color: var(--fluent-text-primary);
  transition: border-color var(--fluent-transition-fast);
}

.form-group input:focus {
  outline: none;
  border-color: var(--fluent-accent);
}

.form-group input::placeholder {
  color: var(--fluent-text-tertiary);
}

.dialog-footer {
  padding: var(--fluent-space-md) var(--fluent-space-lg);
  display: flex;
  justify-content: flex-end;
  border-top: 1px solid var(--fluent-border);
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

.dialog-fade-enter-active,
.dialog-fade-leave-active {
  transition: opacity var(--fluent-transition-fast);
}

.dialog-fade-enter-from,
.dialog-fade-leave-to {
  opacity: 0;
}
</style>
