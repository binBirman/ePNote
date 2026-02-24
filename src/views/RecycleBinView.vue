<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { recycleBin, getSubjects, filterRecycleBinBySubject, restoreFromRecycleBin, permanentDelete, type QuestionState, type Question } from '../mock/data'

const router = useRouter()

const subjects = ref<string[]>([])
const subjectFilter = ref<string>('ALL')
const displayedQuestions = ref<Question[]>([])
const selectedIds = ref<number[]>([])

onMounted(() => {
  subjects.value = ['全部', ...getSubjects()]
  loadRecycleBin()
})

const loadRecycleBin = () => {
  displayedQuestions.value = filterRecycleBinBySubject(subjectFilter.value)
}

const filteredQuestions = computed(() => {
  return displayedQuestions.value
})

const getStateColor = (state: QuestionState) => {
  switch (state) {
    case 'NEW':
      return '#2196F3'
    case 'LEARNING':
      return '#FF9800'
    case 'STABLE':
      return '#4CAF50'
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

const handleRestore = (id?: number) => {
  const idsToRestore = id ? [id] : selectedIds.value
  if (idsToRestore.length === 0) {
    alert('请选择要恢复的题目')
    return
  }

  if (!confirm(`确定要恢复 ${idsToRestore.length} 道题目吗？`)) {
    return
  }

  idsToRestore.forEach(id => restoreFromRecycleBin(id))
  selectedIds.value = []
  loadRecycleBin()
  alert('恢复成功！')
}

const handlePermanentDelete = (id?: number) => {
  const idsToDelete = id ? [id] : selectedIds.value
  if (idsToDelete.length === 0) {
    alert('请选择要删除的题目')
    return
  }

  if (!confirm(`确定要永久删除 ${idsToDelete.length} 道题目吗？此操作不可恢复！`)) {
    return
  }

  idsToDelete.forEach(id => permanentDelete(id))
  selectedIds.value = []
  loadRecycleBin()
  alert('删除成功！')
}

const isAllSelected = computed(() => {
  return filteredQuestions.value.length > 0 && selectedIds.value.length === filteredQuestions.value.length
})

const hasSelected = computed(() => {
  return selectedIds.value.length > 0
})

const goToDetail = (id: number) => {
  router.push(`/questions/${id}`)
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

    <!-- 工具栏 -->
    <div class="toolbar">
      <div class="toolbar-left">
        <select v-model="subjectFilter" class="subject-filter" @change="loadRecycleBin">
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
            <h3 class="question-title">{{ question.name }}</h3>
            <span class="state-badge" :style="{ backgroundColor: getStateColor(question.state) }">
              {{ getStateLabel(question.state) }}
            </span>
          </div>
          <div class="question-meta">
            <span class="meta-item">#{{ question.id }}</span>
            <span class="meta-item">{{ question.subject }}</span>
            <span class="meta-item">{{ question.knowledgePoint }}</span>
            <span class="meta-item">删除日期：{{ question.deletedDate }}</span>
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
  </div>
</template>

<style scoped>
.recycle-bin-container {
  max-width: 900px;
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
  background: none;
  border: none;
  color: #4CAF50;
  font-size: 14px;
  cursor: pointer;
  padding: 8px 16px;
  transition: color 0.2s;
}

.back-btn:hover {
  color: #45a049;
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
  color: #999;
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
  background-color: #fafafa;
  border-radius: 8px;
  margin-bottom: 12px;
}

.select-checkbox {
  width: 18px;
  height: 18px;
  cursor: pointer;
}

.select-label {
  color: #666;
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
  border-color: #ddd;
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
</style>
