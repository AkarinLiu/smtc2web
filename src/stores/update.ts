import { defineStore } from 'pinia'
import { ref } from 'vue'
import { hasTauri, tauriInvoke } from '@/utils'

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
  const downloadError = ref<string | null>(null);

  async function checkForUpdates(): Promise<UpdateCheckResult | null> {
    if (!hasTauri()) return null;
    checking.value = true;
    downloadError.value = null;

    try {
      const result = await tauriInvoke<UpdateCheckResult>("check_update");
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

  async function downloadAndInstall(): Promise<void> {
    if (!hasTauri()) return;
    if (!lastResult.value?.download_url) return;
    downloading.value = true;
    downloadError.value = null;

    try {
      await tauriInvoke("start_update", {
        downloadUrl: lastResult.value.download_url,
      });
    } catch (e) {
      console.error("下载更新失败:", e);
      downloadError.value = String(e);
      throw e;
    } finally {
      downloading.value = false;
    }
  }

  function closeDialog() {
    showDialog.value = false;
    downloadError.value = null;
  }

  return {
    checking,
    downloading,
    lastResult,
    showDialog,
    downloadError,
    checkForUpdates,
    downloadAndInstall,
    closeDialog,
  };
});
