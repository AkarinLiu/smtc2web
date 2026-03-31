import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import { useToastStore } from "./toast";
import type { Theme } from "@/types/theme";

export const useThemeStore = defineStore("theme", () => {
  const { t } = useI18n();
  const toast = useToastStore();

  const themes = ref<Theme[]>([]);
  const currentTheme = ref("");
  const loading = ref(false);
  const uploadLoading = ref(false);

  const hasThemes = computed(() => themes.value.length > 0);

  const hasTauri = computed(() => {
    return typeof window !== "undefined" && window.__TAURI__ !== undefined;
  });

  const mockThemes: Theme[] = [
    {
      name: "默认主题",
      folder_name: "default",
      author: "smtc2web",
      version: "1.0.0",
      screenshot_path: "",
      is_default: true,
    },
  ];

  async function loadThemes() {
    loading.value = true;
    try {
      if (hasTauri.value && window.__TAURI__) {
        const { invoke } = window.__TAURI__.core;
        themes.value = await invoke<Theme[]>("get_themes");
      } else {
        themes.value = mockThemes;
      }
    } catch (e) {
      console.error("加载主题失败:", e);
      themes.value = mockThemes;
      toast.error(t("messages.theme.loadError"));
    } finally {
      loading.value = false;
    }
  }

  async function loadCurrentTheme() {
    try {
      if (hasTauri.value && window.__TAURI__) {
        const { invoke } = window.__TAURI__.core;
        currentTheme.value = await invoke<string>("get_current_theme");
      } else {
        currentTheme.value = "default";
      }
    } catch (e) {
      console.error("加载当前主题失败:", e);
    }
  }

  async function selectTheme(folderName: string) {
    if (folderName === currentTheme.value) return;

    try {
      if (hasTauri.value && window.__TAURI__) {
        const { invoke } = window.__TAURI__.core;
        await invoke("set_theme", { themeName: folderName });
        await loadCurrentTheme();
        toast.success(t("messages.theme.switchSuccess"));
      } else {
        currentTheme.value = folderName;
        toast.success(t("messages.theme.switchSuccess") + " (模拟模式)");
      }
    } catch (e: any) {
      console.error("切换主题失败:", e);
      const message = e.message || e.toString() || String(e);
      toast.error(t("messages.theme.switchError", { message }));
    }
  }

  async function deleteTheme(theme: Theme) {
    // 使用 Toast 确认弹窗
    toast.confirm(
      t("messages.theme.deleteConfirm", { name: theme.name }),
      {
        confirmText: t("common.delete"),
        cancelText: t("common.cancel"),
        onConfirm: async () => {
          try {
            if (hasTauri.value && window.__TAURI__) {
              const { invoke } = window.__TAURI__.core;
              await invoke("delete_theme", { themeFolder: theme.folder_name });
              await loadThemes();
            } else {
              themes.value = themes.value.filter(
                (t: Theme) => t.folder_name !== theme.folder_name,
              );
            }
          } catch (e: any) {
            console.error("删除主题失败:", e);
            const message = e.message || e.toString() || String(e);
            toast.error(t("messages.theme.deleteError", { message }));
          }
        },
      },
      t("common.confirm"),
    );
  }

  async function uploadTheme() {
    if (!hasTauri.value) {
      toast.error("上传功能需要 Tauri 环境");
      return;
    }

    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".zip";

    input.onchange = async (e: Event) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      if (!file.name.endsWith(".zip")) {
        toast.error(t("messages.theme.invalidFormat"));
        return;
      }

      uploadLoading.value = true;
      try {
        const arrayBuffer = await file.arrayBuffer();
        const uint8Array = new Uint8Array(arrayBuffer);

        if (window.__TAURI__) {
          const { invoke } = window.__TAURI__.core;
          const themeName = await invoke<string>("upload_theme_from_bytes", {
            fileName: file.name,
            fileData: Array.from(uint8Array),
          });

          await new Promise((resolve) => setTimeout(resolve, 500));
          await loadThemes();
          toast.success(t("messages.theme.uploadSuccess", { name: themeName }));
        }
      } catch (e: any) {
        console.error("上传主题失败:", e);
        // 处理 Tauri 返回的错误信息
        const errorMessage = e.message || e.toString() || String(e);

        // 检查是否是验证错误，提取其中的详细信息
        if (errorMessage.includes("此主题无效")) {
          // 直接使用后端返回的详细错误信息
          toast.error(errorMessage, t("common.error"));
        } else {
          toast.error(
            t("messages.theme.uploadError", { message: errorMessage }),
          );
        }
      } finally {
        uploadLoading.value = false;
      }
    };

    input.click();
  }

  function getScreenshotUrl(path: string): string | null {
    return path && path.startsWith("data:") ? path : null;
  }

  return {
    themes,
    currentTheme,
    loading,
    uploadLoading,
    hasThemes,
    loadThemes,
    loadCurrentTheme,
    selectTheme,
    deleteTheme,
    uploadTheme,
    getScreenshotUrl,
  };
});
