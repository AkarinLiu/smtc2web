import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface UpdateCheckResult {
  has_update: boolean;
  current_version: string;
  latest_version: string;
  notes: string | null;
  download_url: string | null;
  source: string;
  error: string | null;
}

export const useUpdateStore = defineStore("update", () => {
  const checking = ref(false);
  const downloading = ref(false);
  const lastResult = ref<UpdateCheckResult | null>(null);
  const showDialog = ref(false);

  const hasTauri = computed(() => {
    return typeof window !== "undefined" && window.__TAURI__ !== undefined;
  });

  /** 手动检查更新 */
  async function checkForUpdates(): Promise<UpdateCheckResult | null> {
    if (!hasTauri.value || !window.__TAURI__) return null;
    checking.value = true;

    try {
      const { invoke } = window.__TAURI__.core;
      const result = await invoke<UpdateCheckResult>("check_update");
      lastResult.value = result;
      if (result.has_update || result.error) {
        showDialog.value = true;
      }
      return result;
    } catch (e) {
      console.error("检查更新失败:", e);
      lastResult.value = {
        has_update: false,
        current_version: "",
        latest_version: "",
        notes: null,
        download_url: null,
        source: "",
        error: String(e),
      };
      return lastResult.value;
    } finally {
      checking.value = false;
    }
  }

  /** 通过 Tauri updater 插件下载并安装更新 */
  async function downloadAndInstall(): Promise<void> {
    if (!hasTauri.value || !window.__TAURI__) return;
    downloading.value = true;

    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const update = await check();
      if (update) {
        await update.downloadAndInstall();
        // 安装后 Tauri 会自动重启应用
      }
    } catch (e) {
      console.error("下载更新失败:", e);
      throw e;
    } finally {
      downloading.value = false;
    }
  }

  function closeDialog() {
    showDialog.value = false;
  }

  return {
    checking,
    downloading,
    lastResult,
    showDialog,
    checkForUpdates,
    downloadAndInstall,
    closeDialog,
  };
});
