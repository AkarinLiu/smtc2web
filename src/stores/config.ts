import { defineStore } from "pinia";
import { reactive, ref } from "vue";
import type { AppConfig } from "@/types/config";
import { hasTauri, tauriInvoke } from "@/utils";

export const useConfigStore = defineStore("config", () => {
  const config = reactive<AppConfig>({
    server_port: 3030,
    address: "127.0.0.1",
    current_theme: "",
    locale: "zh-CN",
    process_filter: "*",
    update_source: "github",
    auto_check_update: true,
    autostart: false,
  });

  const loading = ref(false);
  const saved = ref(false);
  const currentAppId = ref("");

  async function loadConfig() {
    try {
      if (hasTauri()) {
        const data = await tauriInvoke<AppConfig>("get_config");
        Object.assign(config, data);
      }
    } catch (e) {
      console.error("加载配置失败:", e);
    }
  }

  async function saveConfig() {
    loading.value = true;
    saved.value = false;

    try {
      if (hasTauri()) {
        await tauriInvoke("save_config", { configDto: config });
        await tauriInvoke("set_autostart", { enable: config.autostart });
      }
      saved.value = true;
      setTimeout(() => (saved.value = false), 2000);
    } catch (e) {
      console.error("保存配置失败:", e);
      alert("保存配置失败: " + e);
    } finally {
      loading.value = false;
    }
  }

  async function getCurrentAppId() {
    try {
      if (hasTauri()) {
        const appId = await tauriInvoke<string>("get_current_app_id");
        currentAppId.value = appId;
        return appId;
      }
    } catch (e) {
      console.error("获取当前应用ID失败:", e);
    }
    return "";
  }

  return {
    config,
    loading,
    saved,
    currentAppId,
    loadConfig,
    saveConfig,
    getCurrentAppId,
  };
});
