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
                <select v-model="localConfig.update_source" class="form-input">
                    <option value="github">GitHub</option>
                    <option value="official">
                        {{ t("settings.update.source.official") }}
                    </option>
                </select>
                <p class="hint">{{ t("settings.update.source.hint") }}</p>
            </div>

            <div class="form-group">
                <label class="checkbox-label">
                    <input
                        type="checkbox"
                        v-model="localConfig.auto_check_update"
                    />
                    {{ t("settings.update.autoCheck") }}
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
                    <input type="checkbox" v-model="localConfig.autostart" />
                    {{ t("settings.system.autostart") }}
                </label>
                <p class="hint">{{ t("settings.system.autostartHint") }}</p>
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
import { reactive, ref, watch, computed } from "vue";
import { useI18n } from "vue-i18n";
import LanguageSelector from "./LanguageSelector.vue";
import type { AppConfig } from "@/types/config";
import { useUpdateStore, type UpdateCheckResult } from "@/stores/update";

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
</script>

<style scoped>
.settings-form {
    background-color: var(--fluent-bg-card);
    padding: var(--fluent-space-lg);
    border-radius: var(--fluent-radius-lg);
    box-shadow: var(--fluent-shadow-md);
    max-width: 720px;
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

.btn-secondary {
    background-color: var(--fluent-bg-secondary);
    color: var(--fluent-text-primary);
    border: 1px solid var(--fluent-border);
}

.btn-secondary:hover:not(:disabled) {
    background-color: var(--fluent-bg-primary);
    border-color: var(--fluent-accent);
}

.form-section {
    margin-top: var(--fluent-space-lg);
    padding-top: var(--fluent-space-lg);
    border-top: 1px solid var(--fluent-border);
}

.section-title {
    font-size: 16px;
    font-weight: 700;
    margin-bottom: var(--fluent-space-lg);
    color: var(--fluent-text-primary);
}

.update-status {
    display: inline-block;
    margin-left: var(--fluent-space-md);
    font-size: 13px;
    color: var(--fluent-text-secondary);
}

.update-status.has-update {
    color: var(--fluent-accent);
    font-weight: 600;
}
</style>
