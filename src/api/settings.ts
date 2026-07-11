import { call } from "./core";

export interface SubjectConfig {
  archived: boolean;
  recommendation_limit: number | null;
}

export interface AppSettings {
  default_review_limit: number;
  per_subject_daily_limit: number;
  subjects: Record<string, SubjectConfig>;
  new_question_ratio: number;
  new_question_guarantee_ratio: number;
  recommendation_randomness: number;
  show_debug_info: boolean;
  /** 本地时区相对 UTC 的偏移小时数（东时区为正，例如 +8）。 */
  timezone_offset_hours: number;
  /** 逻辑日切日小时（0..=23，例：03 = 凌晨 3 点切日）。 */
  day_cutoff_hour: number;
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
