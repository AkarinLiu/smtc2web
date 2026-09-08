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
                <span class="current-app-label">{{
                    t("settings.processFilter.currentApp")
                }}</span>
                <code class="current-app-value">{{ currentAppId }}</code>
            </div>
        </div>

        <!-- 更新设置 -->
        <div class="form-section">
            <h3 class="section-title">{{ t("settings.update.title") }}</h3>

            <div class="form-group">
                <label>{{ t("settings.update.source.label") }}</label>
                <select
                    v-model="localConfig.update_source"
                    class="form-input select-trigger"
                >
                    <option value="github">GitHub</option>
                    <option value="official">
                        {{ t("settings.update.source.official") }}
                    </option>
                </select>
                <p class="hint">{{ t("settings.update.source.hint") }}</p>
            </div>

            <div class="form-group">
                <label class="checkbox-label">
                    <CheckboxRoot
                        v-model="localConfig.auto_check_update"
                        class="checkbox-box"
                    >
                        <CheckboxIndicator class="checkbox-indicator">
                            <font-awesome-icon icon="check" />
                        </CheckboxIndicator>
                    </CheckboxRoot>
                    <span>{{ t("settings.update.autoCheck") }}</span>
                </label>
                <p class="hint">{{ t("settings.update.autoCheckHint") }}</p>
            </div>

            <div class="form-group">
                <button
                    class="btn btn-secondary"
                    @click="handleCheckUpdate"
                    :disabled="checkingUpdate"
                >
                    <font-awesome-icon icon="rotate" :spin="checkingUpdate" />
                    {{
                        checkingUpdate
                            ? t("settings.update.checking")
                            : t("settings.update.checkNow")
                    }}
                </button>
                <span
                    v-if="updateStatus"
                    class="update-status"
                    :class="{ 'has-update': updateStatus.has_update }"
                >
                    {{ updateStatusText }}
                </span>
            </div>
        </div>

        <!-- 系统设置 -->
        <div class="form-section">
            <h3 class="section-title">{{ t("settings.system.title") }}</h3>

            <div class="form-group">
                <label class="checkbox-label">
                    <CheckboxRoot
                        v-model="localConfig.minimize_to_tray"
                        class="checkbox-box"
                    >
                        <CheckboxIndicator class="checkbox-indicator">
                            <font-awesome-icon icon="check" />
                        </CheckboxIndicator>
                    </CheckboxRoot>
                    <span>{{ t("settings.system.minimizeToTray") }}</span>
                </label>
                <p class="hint">{{ t("settings.system.minimizeToTrayHint") }}</p>
            </div>
        </div>

        <!-- 外观设置 -->
        <div class="form-section">
            <h3 class="section-title">{{ t("settings.appearance.title") }}</h3>

            <div class="form-group">
                <label>{{ t("settings.appearance.fontFamily.label") }}</label>
                <div class="font-selector">
                    <input
                        type="text"
                        v-model="localConfig.font_family"
                        :list="'font-list-' + uid"
                        :placeholder="scanningFonts ? t('settings.appearance.fontFamily.scanning') : t('settings.appearance.fontFamily.placeholder')"
                        class="form-input font-input"
                    />
                    <datalist :id="'font-list-' + uid">
                        <option v-for="f in systemFonts" :key="f" :value="f" />
                    </datalist>
                    <span v-if="scanningFonts" class="font-scan-hint">
                        {{ t("settings.appearance.fontFamily.scanningHint") }}
                    </span>
                </div>
                <p class="hint">{{ t("settings.appearance.fontFamily.hint") }}</p>
            </div>
        </div>

        <div class="form-actions">
            <button
                class="btn btn-primary"
                @click="handleSave"
                :disabled="loading"
            >
                <span v-if="loading"
                    ><font-awesome-icon icon="spinner" spin />
                    {{ t("settings.saving") }}</span
                >
                <span v-else-if="saved"
                    ><font-awesome-icon icon="check" />
                    {{ t("settings.saved") }}</span
                >
                <span v-else
                    ><font-awesome-icon icon="floppy-disk" />
                    {{ t("settings.save") }}</span
                >
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { reactive, ref, watch, computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import {
  CheckboxIndicator,
  CheckboxRoot,
} from "reka-ui";
import LanguageSelector from "./LanguageSelector.vue";
import type { AppConfig } from "@/types/config";
import { useUpdateStore, type UpdateCheckResult } from "@/stores/update";
import { hasTauri, tauriInvoke } from "@/utils";

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
const updateStore = useUpdateStore();

const localConfig = reactive<AppConfig>({ ...props.config });
const checkingUpdate = ref(false);
const updateStatus = ref<UpdateCheckResult | null>(null);
const systemFonts = ref<string[]>([]);
const scanningFonts = ref(false);
const uid = Array.from(globalThis.crypto.getRandomValues(new Uint8Array(16)))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");

const updateStatusText = computed(() => {
    if (!updateStatus.value) return "";
    if (updateStatus.value.error) {
        return `⚠ ${updateStatus.value.error}`;
    }
    if (updateStatus.value.has_update) {
        return t("settings.update.newVersionAvailable", {
            version: updateStatus.value.latest_version,
        });
    }
    return t("settings.update.alreadyLatest", {
        version: updateStatus.value.current_version,
    });
});

watch(
    () => props.config,
    (newConfig: AppConfig) => {
        Object.assign(localConfig, newConfig);
    },
    { deep: true },
);

async function handleCheckUpdate() {
    checkingUpdate.value = true;
    updateStatus.value = null;
    try {
        const result = await updateStore.checkForUpdates();
        updateStatus.value = result;
    } finally {
        checkingUpdate.value = false;
    }
}

function handleSave() {
    Object.assign(props.config, localConfig);
    emit("save");
}

async function scanFonts() {
    if (scanningFonts.value) return;
    scanningFonts.value = true;
    try {
        if (hasTauri()) {
            systemFonts.value = await tauriInvoke<string[]>("list_system_fonts");
        }
    } catch (e) {
        console.error("扫描系统字体失败:", e);
    } finally {
        scanningFonts.value = false;
    }
}

onMounted(() => {
    scanFonts();
});
</script>

<style scoped>
.settings-form {
    background-color: var(--ui-bg-card);
    padding: var(--ui-space-lg);
    border-radius: var(--ui-radius-lg);
    box-shadow: var(--ui-shadow-md);
    max-width: 720px;
}

.form-group {
    margin-bottom: var(--ui-space-lg);
}

.form-group label {
    display: block;
    font-size: 14px;
    font-weight: 600;
    margin-bottom: var(--ui-space-sm);
    color: var(--ui-text-primary);
}

.form-input {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid var(--ui-border);
    border-radius: var(--ui-radius-md);
    font-size: 14px;
    background-color: var(--ui-bg-primary);
    color: var(--ui-text-primary);
    transition: border-color var(--ui-transition-fast);
}

.form-input:focus {
    outline: none;
    border-color: var(--ui-accent);
}

.form-textarea {
    resize: vertical;
    min-height: 80px;
    font-family: monospace;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: var(--ui-space-sm);
  cursor: pointer;
  font-weight: 500 !important;
}

.checkbox-box {
  width: 18px;
  height: 18px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  border: 1px solid var(--ui-border);
  border-radius: var(--ui-radius-sm);
  background-color: var(--ui-bg-primary);
  color: var(--ui-text-on-accent);
  cursor: pointer;
  transition: background-color var(--ui-transition-fast),
    border-color var(--ui-transition-fast);
}

.checkbox-box[data-state='checked'] {
  background-color: var(--ui-accent);
  border-color: var(--ui-accent);
}

.checkbox-box:focus {
  outline: none;
  border-color: var(--ui-accent);
}

.checkbox-indicator {
  display: flex;
  font-size: 12px;
}

.select-trigger {
  appearance: none;
  -webkit-appearance: none;
  width: 100%;
  padding-right: 32px;
  cursor: pointer;
  text-align: left;
  background-image: var(--ui-select-chevron);
  background-repeat: no-repeat;
  background-position: right 12px center;
}

.hint {
    font-size: 12px;
    color: var(--ui-text-secondary);
    margin-top: var(--ui-space-xs);
}

.current-app {
    margin-top: var(--ui-space-sm);
    padding: var(--ui-space-sm) var(--ui-space-md);
    background-color: var(--ui-bg-secondary);
    border-radius: var(--ui-radius-md);
    display: flex;
    align-items: center;
    gap: var(--ui-space-sm);
}

.current-app-label {
    font-size: 12px;
    color: var(--ui-text-secondary);
}

.current-app-value {
    font-size: 13px;
    font-family: monospace;
    color: var(--ui-text-primary);
    background-color: var(--ui-bg-primary);
    padding: 2px 8px;
    border-radius: var(--ui-radius-sm);
}

.form-actions {
    margin-top: var(--ui-space-lg);
    padding-top: var(--ui-space-lg);
    border-top: 1px solid var(--ui-border);
}

.btn {
    padding: 10px 24px;
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

.btn-secondary {
    background-color: var(--ui-bg-secondary);
    color: var(--ui-text-primary);
    border: 1px solid var(--ui-border);
}

.btn-secondary:hover:not(:disabled) {
    background-color: var(--ui-bg-primary);
    border-color: var(--ui-accent);
}

.form-section {
    margin-top: var(--ui-space-lg);
    padding-top: var(--ui-space-lg);
    border-top: 1px solid var(--ui-border);
}

.section-title {
    font-size: 16px;
    font-weight: 700;
    margin-bottom: var(--ui-space-lg);
    color: var(--ui-text-primary);
}

.update-status {
    display: inline-block;
    margin-left: var(--ui-space-md);
    font-size: 13px;
    color: var(--ui-text-secondary);
}

.update-status.has-update {
    color: var(--ui-accent);
    font-weight: 600;
}

.font-selector {
    position: relative;
}

.font-input {
    font-family: inherit;
}

.font-scan-hint {
    font-size: 11px;
    color: var(--ui-text-secondary);
    margin-top: 4px;
}
</style>
