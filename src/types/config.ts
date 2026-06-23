export interface AppConfig {
  server_port: number
  address: string
  show_console: boolean
  current_theme: string
  locale: string
  process_filter: string
  /** 更新源: "github" | "official" */
  update_source: string
  /** 是否启用自动检查更新 */
  auto_check_update: boolean
}
