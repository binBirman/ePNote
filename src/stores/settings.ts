import { defineStore } from 'pinia'
import { ref } from 'vue'
import { getAppSettings, saveAppSettings, openDataDirectory } from '@/api/settings'
import { listSubjects } from '@/api/review'
import type { AppSettings, SubjectConfig } from '@/api/settings'

export const useSettingsStore = defineStore('settings', () => {
  // ===== 设置状态 =====
  const defaultReviewLimit = ref<number>(10)
  const perSubjectDailyLimit = ref<number>(10)
  const newQuestionRatio = ref<number>(0.3)
  const recommendationRandomness = ref<number>(1.0)
  const showDebugInfo = ref<boolean>(false)

  // 科目池管理
  const subjectConfigs = ref<Record<string, SubjectConfig>>({})
  const allSubjects = ref<string[]>([])

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
      perSubjectDailyLimit.value = s.per_subject_daily_limit ?? 10
      newQuestionRatio.value = s.new_question_ratio ?? 0.3
      recommendationRandomness.value = s.recommendation_randomness ?? 1.0
      showDebugInfo.value = s.show_debug_info ?? false
      subjectConfigs.value = s.subjects || {}
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
      per_subject_daily_limit: perSubjectDailyLimit.value,
      new_question_ratio: newQuestionRatio.value,
      recommendation_randomness: recommendationRandomness.value,
      show_debug_info: showDebugInfo.value,
      subjects: subjectConfigs.value,
    }
    await saveAppSettings(settings)
  }

  /** 从数据库加载全部科目 */
  async function loadSubjectPool() {
    try {
      const subjectList = await listSubjects()
      allSubjects.value = subjectList.filter(s => s !== '')
    } catch (e) {
      console.error('加载科目列表失败:', e)
    }
  }

  /** 切换科目归档状态 */
  function toggleSubjectArchive(subject: string) {
    const config = subjectConfigs.value[subject] ?? { archived: false, recommendation_limit: null }
    config.archived = !config.archived
    subjectConfigs.value = { ...subjectConfigs.value, [subject]: config }
  }

  /** 打开数据目录 */
  async function openDataDir() {
    await openDataDirectory()
  }

  return {
    // 状态
    defaultReviewLimit,
    perSubjectDailyLimit,
    newQuestionRatio,
    recommendationRandomness,
    showDebugInfo,
    subjectConfigs,
    allSubjects,
    dataRoot,
    loaded,
    // 方法
    loadSettings,
    saveSettings,
    loadSubjectPool,
    toggleSubjectArchive,
    openDataDir,
  }
})
