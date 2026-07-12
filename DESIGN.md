# SMTC2Web UI 设计文档

## 技术栈

| 层 | 选型 |
|---|---|
| 框架 | Vue 3 (Composition API, `<script setup lang="ts">`) |
| 构建 | Vite |
| 路由 | Vue Router 4 (`createWebHistory`, 2 条路由) |
| 状态 | Pinia (Composition API 模式, 5 stores) |
| 国际化 | vue-i18n (`legacy: false`, zh-CN / en) |
| 图标 | FontAwesome (17 个图标, 全局注册) |
| 样式 | 纯 CSS + Fluent Design 变量 |
| 目标 | Tauri 桌面应用 |

## 路由

| 路径 | 视图 | 说明 |
|---|---|---|
| `/` | — | 重定向到 `/themes` |
| `/themes` | `ThemesView` | 主题管理主页 |
| `/settings` | `SettingsView` | 应用设置页 |

路由为懒加载。

## 组件树

```
App.vue
├── TitleBar.vue — 自定义标题栏 + 导航标签 + 窗口控件
├── <RouterView>
│   ├── ThemesView.vue
│   │   ├── ThemeSkeleton.vue  (加载态)
│   │   ├── EmptyState.vue     (空态)
│   │   └── ThemeGrid.vue
│   │       └── ThemeCard.vue  (单张主题卡片)
│   └── SettingsView.vue
│       ├── SettingsSkeleton.vue (加载态)
│       └── SettingsForm.vue
│           └── LanguageSelector.vue
├── Toast.vue          (全局通知, Teleport to body)
└── UpdateDialog.vue   (全局更新弹窗, Teleport to body)
```

## 状态管理 (Pinia Stores)

| Store | 职责 | 关键状态 |
|---|---|---|
| `config` | 应用配置读写 | `config`, `loading`, `currentAppId` |
| `theme` | 主题 CRUD | `themes[]`, `currentTheme`, `uploadLoading` |
| `locale` | 语言切换, 同步 i18n & 后端 | `currentLocale`, `availableLocales` |
| `toast` | Toast 通知 + confirm 弹窗 | `toasts[]`, `position` |
| `update` | 更新检查 & 下载 | `checking`, `downloading`, `showDialog` |

所有 store 通过 `hasTauri()` / `tauriInvoke()` 与 Tauri 后端通信。

## 关键设计模式

1. **Tauri 环境检测** — `hasTauri()` 守卫所有后端调用, 非 Tauri 环境使用 mock 数据
2. **模拟开发** — `theme store` 内置 `mockThemes`, 支持纯浏览器开发
3. **IPC 层封装** — `utils/index.ts` 只有 `hasTauri()` 和 `tauriInvoke()` 两个函数
4. **加载态三态** — 骨架屏 (loading) → 空状态视图 (empty) → 数据视图 (正常)
5. **i18n 优先** — 所有模板文本通过 `t()`, 无硬编码字符串
6. **TypeScript 严格** — `defineProps<Props>()` / `defineEmits<Emits>()`, 无隐式 `any`

## 样式系统

- **无 CSS 框架** — 纯 CSS, 无 Tailwind / Bootstrap
- **Fluent Design 变量** — 颜色/阴影/圆角/间距全部通过 `--fluent-*` CSS 变量控制
- **暗色模式** — `@media (prefers-color-scheme: dark)`, 覆盖 ~15 个变量
- **响应式断点** — 1400px / 768px / 480px
- **作用域样式** — 每个组件 `<style scoped>`, 全局样式仅 `global.css`

## 目录结构

```
src/
├── App.vue
├── main.ts
├── env.d.ts
├── assets/          — 静态资源 (SVG)
├── components/      — 10 个 UI 组件 (扁平结构)
├── config/          — FontAwesome 图标注册
├── i18n/            — vue-i18n 实例 + 语言文件
├── router/          — 路由定义
├── stores/          — 5 个 Pinia stores
├── styles/          — variables.css + global.css
├── types/           — TypeScript 接口定义
├── utils/           — 工具函数 (hasTauri, tauriInvoke)
└── views/           — 2 个页面级视图组件
```

## 数据流

```
用户操作 → Vue 组件 → Pinia Store → tauriInvoke() → Tauri Rust 后端
                                                    ↓
用户界面 ← Vue 响应式 ← Store state 更新 ←──── IPC 返回
```

## 注意

- 无测试文件 — 项目早期阶段
- 无 composables 目录 — 逻辑收敛于 store 和组件内
- 无 CSS 预处理器 — 纯 CSS, 通过 `@import` 导入变量
