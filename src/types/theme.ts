export interface Theme {
  name: string
  folder_name: string
  author: string
  version: string
  screenshot_path: string
  is_default?: boolean
}

export interface GitThemeInfo {
  repo_url: string
  branch: string
  folder_name: string
}
