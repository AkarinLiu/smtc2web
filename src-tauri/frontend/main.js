import { createApp, reactive } from "./lib/js/vue.esm-browser.js";

createApp({
  setup() {
    const info = reactive({
      title: null,
      artist: null,
      album: null,
      position: "00:00",
      duration: "00:00",
      pct: 0,
      is_playing: false,
    });

    let lastData = {};

    async function poll() {
      try {
        const r = await fetch("/api/now");
        const data = await r.json();

        // 简单比较数据是否变化
        if (JSON.stringify(data) !== JSON.stringify(lastData)) {
          Object.assign(info, {
            ...data,
            pct: parseFloat((data.pct ?? 0).toFixed(1)),
          });
          lastData = data;
        }

        // 根据播放状态动态调整轮询间隔
        const pollInterval = info.is_playing ? 100 : 200;
        setTimeout(poll, pollInterval);
      } catch (e) {
        console.error("poll error", e);
        setTimeout(poll, 250); // 错误时使用默认间隔
      }
    }

    poll(); // 启动
    return { info };
  },
}).mount("#app");
