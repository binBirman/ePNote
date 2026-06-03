import { call } from "./core";

export interface AppSettings {
  default_review_limit: number;
  daily_recommendation_limit: number;
  new_question_ratio: number;
  recommendation_randomness: number;
  show_debug_info: boolean;
}

export interface AppSettingsResponse {
  root: string;
  settings: AppSettings;
}

/**
 * 获取完整应用配置（数据目录 + 设置项）
 */
export function getAppSettings() {
  return call<AppSettingsResponse>("get_app_settings_comm", {});
}

/**
 * 保存设置项
 */
export function saveAppSettings(settings: AppSettings) {
  return call<void>("save_app_settings_comm", { settings });
}

/**
 * 打开数据目录（系统文件管理器）
 */
export function openDataDirectory() {
  return call<void>("open_data_directory_comm", {});
}
