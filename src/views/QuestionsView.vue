<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import type { ActiveQuestion } from '@/types/question'
// import type { QuestionState } from '@/types/question'
import {
  show_list_available_questions_page,
  show_list_available_questions_by_state_page,
  show_list_available_questions_by_subject_page,
} from '@/api/question'

const router = useRouter()
const route = useRoute()

const searchKeyword = ref('')
const stateFilter = ref<'ALL' | 'NEW' | 'LEARNING' | 'STABLE'>('ALL')
const subjectFilter = ref<string>('ALL')
const pageSize = 10
const currentPage = ref(0)
const hasNext = ref(false)
interface LocalQuestion {
  id: number
  name: string
  subject: string
  knowledgePoint: string
  lastReviewed?: string
  createdDate?: string
  state: string
}
const displayedQuestions = ref<LocalQuestion[]>([])
const subjects = ref<string[]>([])

onMounted(() => {
  loadQuestions()
})

watch(() => route.query.r, (v) => {
  if (v) loadQuestions()
})

const mapActive = (a: ActiveQuestion): LocalQuestion => ({
  id: a.id,
  name: a.title,
  subject: a.subject || '未知',
  knowledgePoint: '',
  lastReviewed: a.last_review || '',
  createdDate: a.created_at || '',
  state: a.status || '',
})

const loadQuestions = async (page = 0) => {
  try {
    let res: ActiveQuestion[] = []
    if (subjectFilter.value !== 'ALL') {
      res = await show_list_available_questions_by_subject_page(subjectFilter.value, page, pageSize)
    } else if (stateFilter.value !== 'ALL') {
      res = await show_list_available_questions_by_state_page(stateFilter.value, page, pageSize)
    } else {
      res = await show_list_available_questions_page(page, pageSize)
    }

    displayedQuestions.value = res.map(mapActive)
    const set = new Set<string>()
    displayedQuestions.value.forEach(q => { if (q.subject) set.add(q.subject) })
    subjects.value = Array.from(set)

    currentPage.value = page
    hasNext.value = res.length >= pageSize
  } catch (e) {
    console.error('加载题目失败', e)
    displayedQuestions.value = []
    subjects.value = []
    hasNext.value = false
  }
}

const filteredQuestions = computed(() => {
  let filtered = displayedQuestions.value

  // 搜索过滤（支持题名和ID）
  if (searchKeyword.value) {
    const keyword = searchKeyword.value.trim()

    // 尝试作为ID搜索
    const idNumber = parseInt(keyword)
    if (!isNaN(idNumber)) {
      const byId = filtered.find(q => q.id === idNumber)
      if (byId) return [byId]
    }

    // 按名称搜索
    const lowerKeyword = keyword.toLowerCase()
    filtered = filtered.filter(q =>
      q.name.toLowerCase().includes(lowerKeyword) ||
      q.subject.toLowerCase().includes(lowerKeyword) ||
      q.knowledgePoint.toLowerCase().includes(lowerKeyword)
    )
  }

  // 科目过滤
  if (subjectFilter.value !== 'ALL') {
    filtered = filtered.filter(q => q.subject === subjectFilter.value)
  }

  // 状态过滤
  if (stateFilter.value !== 'ALL') {
    filtered = filtered.filter(q => q.state === stateFilter.value)
  }

  return filtered
})

const getStateColor = (state: string) => {
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

const getStateLabel = (state: string) => {
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

const goToDetail = (id: number) => {
  router.push(`/questions/${id}`)
}

const goToNew = () => {
  router.push('/questions/new')
}

const prevPage = () => {
  if (currentPage.value <= 0) return
  loadQuestions(currentPage.value - 1)
}

const nextPage = () => {
  if (!hasNext.value) return
  loadQuestions(currentPage.value + 1)
}

const goToRecycleBin = () => {
  router.push('/recycle-bin')
}
</script>

<template>
  <div class="questions-container">
    <h1 class="page-title">题目管理</h1>

    <!-- 工具栏 -->
    <div class="toolbar">
      <div class="toolbar-left">
        <input
          v-model="searchKeyword"
          type="text"
          class="search-input"
          placeholder="搜索题名或ID..."
        />
        <select v-model="subjectFilter" class="subject-filter">
          <option value="ALL">全部科目</option>
          <option v-for="subject in subjects" :key="subject" :value="subject">
            {{ subject }}
          </option>
        </select>
        <select v-model="stateFilter" class="state-filter">
          <option value="ALL">全部状态</option>
          <option value="NEW">新题</option>
          <option value="LEARNING">学习中</option>
          <option value="STABLE">已掌握</option>
        </select>
      </div>
      <div class="toolbar-right">
        <button class="recycle-bin-btn" @click="goToRecycleBin">
          🗑️ 回收站
        </button>
        <button class="new-btn" @click="goToNew">
          + 新建题目
        </button>
      </div>
    </div>

    <!-- 题目列表 -->
    <div class="questions-list">
      <div
        v-for="question in filteredQuestions"
        :key="question.id"
        class="question-item"
        @click="goToDetail(question.id)"
      >
        <div class="question-header">
          <h3 class="question-title">{{ question.name }}</h3>
          <span
            class="state-badge"
            :style="{ backgroundColor: getStateColor(question.state) }"
          >
            {{ getStateLabel(question.state) }}
          </span>
        </div>
        <div class="question-meta">
          <span class="meta-item">#{{ question.id }}</span>
          <span class="meta-item">{{ question.subject }}</span>
          <span class="meta-item">{{ question.knowledgePoint }}</span>
          <span class="meta-item">
            上次复习：{{ question.lastReviewed || '从未' }}
          </span>
          <span class="meta-item">
            创建日期：{{ question.createdDate }}
          </span>
        </div>
      </div>

      <div v-if="filteredQuestions.length === 0" class="empty-state">
        <div class="empty-icon">📝</div>
        <p class="empty-text">暂无题目</p>
        <button class="empty-action" @click="goToNew">立即创建</button>
      </div>

      <!-- 分页控件 -->
      <div class="pagination" style="display:flex;justify-content:center;gap:12px;margin-top:16px;">
        <button class="btn" @click="prevPage" :disabled="currentPage === 0">上一页</button>
        <div style="align-self:center">第 {{ currentPage + 1 }} 页</div>
        <button class="btn" @click="nextPage" :disabled="!hasNext">下一页</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.questions-container {
  max-width: 800px;
}

.page-title {
  font-size: 28px;
  color: #333;
  margin-bottom: 24px;
  text-align: center;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  gap: 16px;
}

.toolbar-left {
  display: flex;
  gap: 12px;
  flex: 1;
  flex-wrap: wrap;
}

.toolbar-right {
  display: flex;
  gap: 12px;
}

.search-input {
  flex: 1;
  min-width: 200px;
  max-width: 300px;
  padding: 10px 16px;
  border: 1px solid #ddd;
  border-radius: 8px;
  background-color: #fff;
  color: #333;
  font-size: 14px;
}

.search-input:focus {
  outline: none;
  border-color: #4CAF50;
}

.subject-filter,
.state-filter {
  padding: 10px 16px;
  border: 1px solid #ddd;
  border-radius: 8px;
  background-color: #fff;
  color: #333;
  font-size: 14px;
  cursor: pointer;
}

.subject-filter:focus,
.state-filter:focus {
  outline: none;
  border-color: #4CAF50;
}

.recycle-bin-btn {
  padding: 10px 20px;
  background-color: #fff;
  border: 1px solid #ddd;
  border-radius: 8px;
  color: #666;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
  white-space: nowrap;
  cursor: pointer;
}

.recycle-bin-btn:hover {
  background-color: #f5f5f5;
  border-color: #bbb;
  color: #333;
}

.new-btn {
  padding: 10px 20px;
  background-color: #4CAF50;
  border: none;
  border-radius: 8px;
  color: #ffffff;
  font-size: 14px;
  font-weight: 600;
  transition: all 0.2s;
  white-space: nowrap;
}

.new-btn:hover {
  background-color: #45a049;
}

.questions-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.question-item {
  background-color: #ffffff;
  border-radius: 10px;
  padding: 20px;
  cursor: pointer;
  transition: all 0.2s;
  border: 1px solid #e0e0e0;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.05);
}

.question-item:hover {
  background-color: #fafafa;
  border-color: #4CAF50;
  transform: translateX(4px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
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

.empty-state {
  text-align: center;
  padding: 60px 20px;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.empty-text {
  color: #666;
  font-size: 16px;
  margin-bottom: 20px;
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
