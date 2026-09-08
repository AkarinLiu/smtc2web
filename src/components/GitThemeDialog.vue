<template>
  <DialogRoot :open="visible" @update:open="(v) => { if (!v) $emit('close') }">
    <DialogPortal>
      <DialogOverlay class="dialog-overlay" @click="$emit('close')" />
      <DialogContent class="dialog">
        <div class="dialog-header">
          <DialogTitle as="h3">{{ t("themes.git.dialog.title") }}</DialogTitle>
          <DialogClose as-child>
            <button class="close-btn" @click="$emit('close')">
              <font-awesome-icon icon="times" />
            </button>
          </DialogClose>
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
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import {
  DialogClose,
  DialogContent,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from "reka-ui";

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
  z-index: 9999;
}

.dialog-overlay[data-state='open'] {
  animation: dialog-fade-in 0.2s ease;
}

.dialog-overlay[data-state='closed'] {
  animation: dialog-fade-out 0.2s ease;
}

.dialog {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 10000;
  background-color: var(--ui-bg-card);
  border-radius: var(--ui-radius-lg);
  box-shadow: var(--ui-shadow-lg);
  width: 90%;
  max-width: 480px;
  overflow: hidden;
  outline: none;
}

.dialog[data-state='open'] {
  animation: dialog-fade-in 0.2s ease;
}

.dialog[data-state='closed'] {
  animation: dialog-fade-out 0.2s ease;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--ui-space-md) var(--ui-space-lg);
  border-bottom: 1px solid var(--ui-border);
}

.dialog-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--ui-text-primary);
}

.close-btn {
  background: none;
  border: none;
  color: var(--ui-text-secondary);
  cursor: pointer;
  font-size: 16px;
  padding: var(--ui-space-xs);
  border-radius: var(--ui-radius-sm);
  transition: background-color var(--ui-transition-fast);
}

.close-btn:hover {
  background-color: var(--ui-bg-secondary);
}

.dialog-body {
  padding: var(--ui-space-lg);
  display: flex;
  flex-direction: column;
  gap: var(--ui-space-md);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--ui-space-xs);
}

.form-group label {
  font-size: 13px;
  font-weight: 600;
  color: var(--ui-text-primary);
}

.form-group input {
  padding: 10px 12px;
  border: 1px solid var(--ui-border);
  border-radius: var(--ui-radius-md);
  font-size: 14px;
  background-color: var(--ui-bg-primary);
  color: var(--ui-text-primary);
  transition: border-color var(--ui-transition-fast);
}

.form-group input:focus {
  outline: none;
  border-color: var(--ui-accent);
}

.form-group input::placeholder {
  color: var(--ui-text-tertiary);
}

.dialog-footer {
  padding: var(--ui-space-md) var(--ui-space-lg);
  display: flex;
  justify-content: flex-end;
  border-top: 1px solid var(--ui-border);
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: var(--ui-radius-md);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--ui-transition-fast);
  display: inline-flex;
  align-items: center;
  gap: var(--ui-space-xs);
}

.btn-primary {
  background-color: var(--ui-accent);
  color: var(--ui-text-on-accent);
}

.btn-primary:hover:not(:disabled) {
  background-color: var(--ui-accent-hover);
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

@keyframes dialog-fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes dialog-fade-out {
  from {
    opacity: 1;
  }
  to {
    opacity: 0;
  }
}
</style>
