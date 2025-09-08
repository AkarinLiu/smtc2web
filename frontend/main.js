import { createApp, reactive } from "./vue.esm-browser.js";

createApp({
  setup() {
    const info = reactive({
      title: null,
      artist: null,
      album: null,
      position: 0,
      duration: 0,
      pct: 0,
    });

    // 格式化秒 → mm:ss
    function fmtTime(sec) {
      if (sec == null || sec < 0) return "--";
      const m = Math.floor(sec / 60)
        .toString()
        .padStart(2, "0");
      const s = Math.floor(sec % 60)
        .toString()
        .padStart(2, "0");
      return `${m}:${s}`;
    }

    async function poll() {
      try {
        const r = await fetch("/api/now");
        const data = await r.json();
        Object.assign(info, {
          ...data,
          pct: data.pct ?? 0,
        });
      } catch (e) {
        console.error("poll error", e);
      }
      setTimeout(poll, 1000); // 1 秒轮询
    }

    poll(); // 启动
    return { info, fmtTime };
  },
}).mount("#app");
