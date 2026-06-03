import { defineStore } from 'pinia'
import { ref } from 'vue'
import { getAppSettings, saveAppSettings, openDataDirectory } from '@/api/settings'
import type { AppSettings } from '@/api/settings'

export const useSettingsStore = defineStore('settings', () => {
  // ===== 设置状态 =====
  const defaultReviewLimit = ref<number>(10)
  const dailyRecommendationLimit = ref<number>(50)
  const newQuestionRatio = ref<number>(0.3)
  const recommendationRandomness = ref<number>(1.0)
  const showDebugInfo = ref<boolean>(false)

  // 数据目录（只读展示）
  const dataRoot = ref<string>('')
  const loaded = ref<boolean>(false)

  // ===== 方法 =====

  /** 从后端加载设置 */
  async function loadSettings() {
    try {
      const resp = await getAppSettings()
      dataRoot.value = resp.root || ''
      const s = resp.settings
      defaultReviewLimit.value = s.default_review_limit ?? 10
      dailyRecommendationLimit.value = s.daily_recommendation_limit ?? 50
      newQuestionRatio.value = s.new_question_ratio ?? 0.3
      recommendationRandomness.value = s.recommendation_randomness ?? 1.0
      showDebugInfo.value = s.show_debug_info ?? false
      loaded.value = true
    } catch (e) {
      console.error('加载设置失败，使用默认值:', e)
      loaded.value = true
    }
  }

  /** 保存设置到后端 */
  async function saveSettings() {
    const settings: AppSettings = {
      default_review_limit: defaultReviewLimit.value,
      daily_recommendation_limit: dailyRecommendationLimit.value,
      new_question_ratio: newQuestionRatio.value,
      recommendation_randomness: recommendationRandomness.value,
      show_debug_info: showDebugInfo.value,
    }
    await saveAppSettings(settings)
  }

  /** 打开数据目录 */
  async function openDataDir() {
    await openDataDirectory()
  }

  return {
    // 状态
    defaultReviewLimit,
    dailyRecommendationLimit,
    newQuestionRatio,
    recommendationRandomness,
    showDebugInfo,
    dataRoot,
    loaded,
    // 方法
    loadSettings,
    saveSettings,
    openDataDir,
  }
})
