<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import {
  show_list_deleted_questions_page,
  restoreQuestion,
  permanentlyDeleteQuestion,
} from '@/api/question'
import type { DeleteQuestion } from '@/types/question'
import type { QuestionState } from '@/types/question'

const router = useRouter()

const subjects = ref<string[]>([])
const subjectFilter = ref<string>('ALL')
const displayedQuestions = ref<DeleteQuestion[]>([])
const selectedIds = ref<number[]>([])
const isLoading = ref(true)
const error = ref<string | null>(null)

onMounted(async () => {
  await loadRecycleBin()
})

const loadRecycleBin = async () => {
  isLoading.value = true
  error.value = null
  try {
    console.log('Loading recycle bin...')
    const questions = await show_list_deleted_questions_page(0, 100)
    console.log('Deleted questions loaded:', questions)
    displayedQuestions.value = questions

    // Extract unique subjects
    const uniqueSubjects = Array.from(new Set(questions.map(q => q.subject)))
    subjects.value = ['全部', ...uniqueSubjects]
  } catch (e) {
    error.value = e instanceof Error ? e.message : '加载回收站失败'
    console.error('Failed to load recycle bin:', e)
  } finally {
    isLoading.value = false
  }
}

const filteredQuestions = computed(() => {
  if (subjectFilter.value === 'ALL') {
    return displayedQuestions.value
  }
  return displayedQuestions.value.filter(q => q.subject === subjectFilter.value)
})

const getStateColor = (state: QuestionState) => {
  switch (state) {
    case 'NEW':
      return '#2196F3'
    case 'LEARNING':
      return '#FF9800'
    case 'STABLE':
      return '#4CAF50'
    case 'DUE':
      return '#9C27B0'
    case 'SUSPENDED':
      return '#607D8B'
    default:
      return '#999'
  }
}

const getStateLabel = (state: QuestionState) => {
  switch (state) {
    case 'NEW':
      return '新题'
    case 'LEARNING':
      return '学习中'
    case 'STABLE':
      return '已掌握'
    case 'DUE':
      return '待复习'
    case 'SUSPENDED':
      return '暂停'
    default:
      return '未知'
  }
}

const toggleSelect = (id: number) => {
  const index = selectedIds.value.indexOf(id)
  if (index > -1) {
    selectedIds.value.splice(index, 1)
  } else {
    selectedIds.value.push(id)
  }
}

const toggleSelectAll = () => {
  if (selectedIds.value.length === filteredQuestions.value.length) {
    selectedIds.value = []
  } else {
    selectedIds.value = filteredQuestions.value.map(q => q.id)
  }
}

const handleRestore = async (id?: number) => {
  const idsToRestore = id ? [id] : selectedIds.value
  if (idsToRestore.length === 0) {
    alert('请选择要恢复的题目')
    return
  }

  if (!confirm(`确定要恢复 ${idsToRestore.length} 道题目吗？`)) {
    return
  }

  try {
    for (const id of idsToRestore) {
      await restoreQuestion(id)
    }
    selectedIds.value = []
    await loadRecycleBin()
    alert('恢复成功！')
  } catch (e) {
    error.value = e instanceof Error ? e.message : '恢复失败'
    console.error('Failed to restore question:', e)
  }
}

const handlePermanentDelete = async (id?: number) => {
  const idsToDelete = id ? [id] : selectedIds.value
  if (idsToDelete.length === 0) {
    alert('请选择要删除的题目')
    return
  }

  if (!confirm(`确定要永久删除 ${idsToDelete.length} 道题目吗？此操作不可恢复！`)) {
    return
  }

  try {
    for (const id of idsToDelete) {
      await permanentlyDeleteQuestion(id)
    }
    selectedIds.value = []
    await loadRecycleBin()
    alert('永久删除成功！')
  } catch (e) {
    error.value = e instanceof Error ? e.message : '删除失败'
    console.error('Failed to permanently delete question:', e)
  }
}

const isAllSelected = computed(() => {
  return filteredQuestions.value.length > 0 && selectedIds.value.length === filteredQuestions.value.length
})

const hasSelected = computed(() => {
  return selectedIds.value.length > 0
})

const goToDetail = (id: number) => {
  router.push(`/recycle-bin/${id}`)
}

const goBack = () => {
  router.push('/questions')
}
</script>

<template>
  <div class="recycle-bin-container">
    <div class="header-section">
      <h1 class="page-title">回收站</h1>
      <button class="back-btn" @click="goBack">
        ← 返回题目列表
      </button>
    </div>

    <!-- 加载状态 -->
    <div v-if="isLoading" class="loading">
      加载中...
    </div>

    <!-- 错误状态 -->
    <div v-else-if="error" class="error">
      {{ error }}
    </div>

    <!-- 正常状态 -->
    <template v-else>
      <!-- 工具栏 -->
      <div class="toolbar">
        <div class="toolbar-left">
          <select v-model="subjectFilter" class="subject-filter">
            <option value="ALL">全部科目</option>
            <option v-for="subject in subjects.slice(1)" :key="subject" :value="subject">
              {{ subject }}
            </option>
          </select>
        </div>
        <div class="toolbar-right">
          <button class="action-btn restore" @click="handleRestore()" :disabled="!hasSelected">
            恢复选中
          </button>
          <button class="action-btn delete" @click="handlePermanentDelete()" :disabled="!hasSelected">
            永久删除
          </button>
        </div>
      </div>

      <!-- 题目列表 -->
      <div v-if="filteredQuestions.length > 0" class="questions-list">
        <div class="select-all-row">
          <input
            type="checkbox"
            :checked="isAllSelected"
            @change="toggleSelectAll"
            class="select-checkbox"
          />
          <label class="select-label">全选</label>
        </div>

        <div
          v-for="question in filteredQuestions"
          :key="question.id"
          class="question-item"
        >
          <div class="question-checkbox">
            <input
              type="checkbox"
              :checked="selectedIds.includes(question.id)"
              @change="toggleSelect(question.id)"
              class="select-checkbox"
            />
          </div>

          <div class="question-content" @click="goToDetail(question.id)">
            <div class="question-header">
              <h3 class="question-title">{{ question.title }}</h3>
              <span class="state-badge" :style="{ backgroundColor: getStateColor(question.status as QuestionState) }">
                {{ getStateLabel(question.status as QuestionState) }}
              </span>
            </div>
            <div class="question-meta">
              <span class="meta-item">#{{ question.id }}</span>
              <span class="meta-item">{{ question.subject }}</span>
              <span class="meta-item">{{ question.knowledge_points.join(', ') }}</span>
              <span class="meta-item">删除日期：{{ question.deleted_at }}</span>
            </div>
          </div>

          <div class="question-actions">
            <button class="icon-btn restore" @click.stop="handleRestore(question.id)" title="恢复">
              ↺
            </button>
            <button class="icon-btn delete" @click.stop="handlePermanentDelete(question.id)" title="永久删除">
              ✕
            </button>
          </div>
        </div>
      </div>

      <!-- 空状态 -->
      <div v-else class="empty-state">
        <div class="empty-icon">🗑️</div>
        <p class="empty-text">回收站为空</p>
        <button class="empty-action" @click="goBack">返回题目列表</button>
      </div>
    </template>
  </div>
</template>

<style scoped>
.recycle-bin-container {
  width: 100%;
  max-width: 800px;
  margin: 0;
  background-color: #fff;
  min-height: 100vh;
  padding: 24px;
  border-radius: 12px;
}

.header-section {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.page-title {
  font-size: 28px;
  color: #333;
  margin: 0;
}

.back-btn {
  background-color: #ffffff;
  border: 2px solid #4CAF50;
  color: #4CAF50;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  padding: 12px 24px;
  border-radius: 8px;
  transition: all 0.2s;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.back-btn:hover {
  background-color: #4CAF50;
  color: #ffffff;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  gap: 16px;
}

.toolbar-left {
  display: flex;
  gap: 12px;
}

.subject-filter {
  padding: 10px 16px;
  border: 1px solid #ddd;
  border-radius: 8px;
  background-color: #fff;
  color: #333;
  font-size: 14px;
  cursor: pointer;
}

.subject-filter:focus {
  outline: none;
  border-color: #4CAF50;
}

.toolbar-right {
  display: flex;
  gap: 12px;
}

.action-btn {
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  color: #ffffff;
  font-size: 14px;
  font-weight: 600;
  transition: all 0.2s;
  cursor: pointer;
}

.action-btn:disabled {
  background-color: #e0e0e0;
  color: #fff;
  cursor: not-allowed;
}

.action-btn.restore {
  background-color: #4CAF50;
}

.action-btn.restore:hover:not(:disabled) {
  background-color: #45a049;
}

.action-btn.delete {
  background-color: #f44336;
}

.action-btn.delete:hover:not(:disabled) {
  background-color: #d32f2f;
}

.questions-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.select-all-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background-color: #f5f5f5;
  border-radius: 8px;
  margin-bottom: 12px;
}

.select-checkbox {
  width: 18px;
  height: 18px;
  cursor: pointer;
}

.select-label {
  color: #333;
  font-size: 14px;
  user-select: none;
}

.question-item {
  background-color: #ffffff;
  border-radius: 10px;
  padding: 16px;
  display: flex;
  align-items: center;
  gap: 16px;
  border: 1px solid #e0e0e0;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.05);
  transition: all 0.2s;
}

.question-item:hover {
  background-color: #fafafa;
  border-color: #4CAF50;
  transform: translateX(4px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.question-checkbox {
  flex-shrink: 0;
}

.question-content {
  flex: 1;
  cursor: pointer;
  min-width: 0;
}

.question-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 12px;
}

.question-title {
  flex: 1;
  font-size: 16px;
  color: #333;
  line-height: 1.5;
  margin: 0;
}

.state-badge {
  padding: 4px 10px;
  border-radius: 12px;
  color: #ffffff;
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
}

.question-meta {
  display: flex;
  gap: 20px;
  flex-wrap: wrap;
}

.meta-item {
  color: #666;
  font-size: 13px;
}

.question-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.icon-btn {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  border: none;
  font-size: 16px;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-btn.restore {
  background-color: #e8f5e9;
  color: #4CAF50;
}

.icon-btn.restore:hover {
  background-color: #4CAF50;
  color: #ffffff;
}

.icon-btn.delete {
  background-color: #ffebee;
  color: #f44336;
}

.icon-btn.delete:hover {
  background-color: #f44336;
  color: #ffffff;
}

.empty-state {
  text-align: center;
  padding: 80px 20px;
}

.empty-icon {
  font-size: 64px;
  margin-bottom: 16px;
}

.empty-text {
  color: #666;
  font-size: 16px;
  margin-bottom: 24px;
}

.empty-action {
  padding: 12px 24px;
  background-color: #4CAF50;
  border: none;
  border-radius: 8px;
  color: #ffffff;
  font-size: 14px;
  transition: all 0.2s;
}

.empty-action:hover {
  background-color: #45a049;
}

.loading {
  text-align: center;
  padding: 60px 20px;
  color: #666;
  font-size: 16px;
}

.error {
  text-align: center;
  padding: 60px 20px;
  color: #f44336;
  font-size: 16px;
}
</style>
