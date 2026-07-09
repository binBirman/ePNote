<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, nextTick, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import type { ActiveQuestion } from '@/types/question'
import {
  classifyQuestions,
  searchQuestions,
  show_subjects,
} from '@/api/question'

const router = useRouter()
const route = useRoute()

const STATE_OPTIONS: { value: string; label: string }[] = [
  { value: 'NEW', label: '新题' },
  { value: 'LEARNING', label: '学习中' },
  { value: 'STABLE', label: '已掌握' },
  { value: 'SUSPENDED', label: '已暂停' },
]

const readQueryPage = (): number => {
  const raw = route.query.page
  const n = Array.isArray(raw) ? Number(raw[0]) : Number(raw)
  return Number.isFinite(n) && n >= 0 ? n : 0
}
const readQueryString = (key: string, def: string): string => {
  const raw = route.query[key]
  const v = Array.isArray(raw) ? raw[0] : raw
  return typeof v === 'string' && v.length > 0 ? v : def
}

const initialSearchKeyword = readQueryString('q', '')
const initialStateFilter = readQueryString('state', 'ALL')
const initialSubjectFilter = readQueryString('subject', 'ALL')
const initialPage = readQueryPage()

const searchKeyword = ref(initialSearchKeyword)
const stateFilter = ref<string>(initialStateFilter)
const subjectFilter = ref<string>(initialSubjectFilter)
const pageSize = 10
const dynamicPageSize = ref(pageSize)
const currentPage = ref(initialPage)
const hasNext = ref(false)
const errorMsg = ref('')

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

const isSearching = computed(() => searchKeyword.value.trim().length > 0)

const containerRef = ref<HTMLElement | null>(null)
const titleRef = ref<HTMLElement | null>(null)
const toolbarRef = ref<HTMLElement | null>(null)
const paginationRef = ref<HTMLElement | null>(null)
const firstItemRef = ref<HTMLElement | null>(null)

const MIN_PAGE_SIZE = 3
const DEFAULT_ITEM_HEIGHT = 120
const LIST_GAP = 12
const VIEWPORT_BUFFER = 16
const OUTER_PADDING_ESTIMATE = 90
const FILL_THRESHOLD = 2 / 3

const setFirstItemRef = (el: unknown) => {
  if (el && !firstItemRef.value && el instanceof Element) {
    firstItemRef.value = el as HTMLElement
  }
}

const getEffectiveSize = (): number => dynamicPageSize.value || pageSize

let resizeTimer: ReturnType<typeof setTimeout> | null = null
let resizeObserver: ResizeObserver | null = null

const recomputePageSize = () => {
  if (typeof window === 'undefined') return
  if (!titleRef.value || !toolbarRef.value || !paginationRef.value) return
  const vh = window.innerHeight
  const chromeH =
    titleRef.value.offsetHeight +
    toolbarRef.value.offsetHeight +
    paginationRef.value.offsetHeight +
    OUTER_PADDING_ESTIMATE
  const available = Math.max(0, vh - chromeH - VIEWPORT_BUFFER)
  if (available <= 0) {
    if (dynamicPageSize.value !== MIN_PAGE_SIZE) dynamicPageSize.value = MIN_PAGE_SIZE
    return
  }
  const itemH = firstItemRef.value
    ? firstItemRef.value.offsetHeight
    : DEFAULT_ITEM_HEIGHT
  const usableH = itemH > 0 ? itemH : DEFAULT_ITEM_HEIGHT
  const slot = usableH + LIST_GAP
  const raw = available / slot
  const base = Math.floor(raw)
  const frac = raw - base
  // 余下空间 > 2/3 个格子时多塞一个；否则按 floor 保留底部留白
  const filled = frac > FILL_THRESHOLD && frac < 1 ? base + 1 : base
  const newSize = Math.max(MIN_PAGE_SIZE, filled)
  if (newSize !== dynamicPageSize.value) {
    dynamicPageSize.value = newSize
  }
}

const scheduleRecompute = () => {
  if (resizeTimer) clearTimeout(resizeTimer)
  resizeTimer = setTimeout(() => {
    resizeTimer = null
    recomputePageSize()
  }, 150)
}

const setupResize = () => {
  if (typeof ResizeObserver !== 'undefined') {
    resizeObserver = new ResizeObserver(scheduleRecompute)
    resizeObserver.observe(document.body)
  } else {
    window.addEventListener('resize', scheduleRecompute)
  }
}

const teardownResize = () => {
  if (resizeTimer) {
    clearTimeout(resizeTimer)
    resizeTimer = null
  }
  if (loadDebounce) {
    clearTimeout(loadDebounce)
    loadDebounce = null
  }
  if (resizeObserver) {
    resizeObserver.disconnect()
    resizeObserver = null
  } else if (typeof window !== 'undefined') {
    window.removeEventListener('resize', scheduleRecompute)
  }
}

let initialized = false

onMounted(async () => {
  initialized = true
  setupResize()
  await nextTick()
  recomputePageSize()
  try {
    subjects.value = await show_subjects()
  } catch (e) {
    console.error('加载科目失败', e)
    subjects.value = []
  }
  await loadQuestions(currentPage.value)
  await nextTick()
  recomputePageSize()
})

onBeforeUnmount(teardownResize)

const persistToUrl = () => {
  if (!initialized) return
  const query: Record<string, string> = {}
  if (searchKeyword.value) query.q = searchKeyword.value
  if (stateFilter.value !== 'ALL') query.state = stateFilter.value
  if (subjectFilter.value !== 'ALL') query.subject = subjectFilter.value
  if (currentPage.value > 0) query.page = String(currentPage.value)
  router.replace({ path: '/questions', query })
}

watch(() => route.fullPath, () => {
  if (initialized) loadQuestions(currentPage.value)
})

watch([searchKeyword, stateFilter, subjectFilter, currentPage], persistToUrl)

// 防抖：搜索框与分类控件共用
let loadDebounce: ReturnType<typeof setTimeout> | null = null
const scheduleLoad = (page = 0) => {
  if (loadDebounce) clearTimeout(loadDebounce)
  loadDebounce = setTimeout(() => {
    loadQuestions(page)
  }, 250)
}

// 搜索或分类变化时重置到第一页
watch(subjectFilter, () => initialized && scheduleLoad(0))
watch(stateFilter, () => initialized && scheduleLoad(0))
watch(searchKeyword, () => initialized && scheduleLoad(0))

const clearSearch = () => {
  searchKeyword.value = ''
}

const mapActive = (a: ActiveQuestion): LocalQuestion => ({
  id: a.id,
  name: a.title,
  subject: a.subject || '未知',
  knowledgePoint: a.knowledge_points.join('、'),
  lastReviewed: a.last_review || '',
  createdDate: a.created_at || '',
  state: a.status || '',
})

const loadQuestions = async (page = 0) => {
  errorMsg.value = ''
  const subject = subjectFilter.value === 'ALL' ? null : subjectFilter.value
  const questionState = stateFilter.value === 'ALL' ? null : stateFilter.value

  try {
    const res = await fetchQuestionsPage(subject, questionState, page)

    displayedQuestions.value = res.map(mapActive)
    currentPage.value = page
    hasNext.value = res.length >= getEffectiveSize()
    await nextTick()
    recomputePageSize()
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    errorMsg.value = msg
    console.error('加载题目失败', msg)
    displayedQuestions.value = []
    hasNext.value = false
  }
}

const fetchQuestionsPage = async (
  subject: string | null,
  questionState: string | null,
  page: number,
): Promise<ActiveQuestion[]> => {
  const size = getEffectiveSize()
  return isSearching.value
    ? await searchQuestions({
        query: searchKeyword.value.trim(),
        subject,
        questionState,
        page,
        pageSize: size,
      })
    : await classifyQuestions({ subject, questionState, page, pageSize: size })
}

const probeHasNext = async (page: number): Promise<boolean> => {
  const subject = subjectFilter.value === 'ALL' ? null : subjectFilter.value
  const questionState = stateFilter.value === 'ALL' ? null : stateFilter.value
  try {
    const res = await fetchQuestionsPage(subject, questionState, page)
    return res.length >= getEffectiveSize()
  } catch {
    return false
  }
}

watch(dynamicPageSize, async (newSize, oldSize) => {
  if (!initialized) return
  if (newSize === oldSize) return
  await loadQuestions(currentPage.value)
})

const filteredQuestions = computed(() => displayedQuestions.value)

const getStateColor = (state: string) => {
  switch (state) {
    case 'NEW':
      return '#2196F3'
    case 'LEARNING':
      return '#FF9800'
    case 'STABLE':
      return '#4CAF50'
    case 'SUSPENDED':
      return '#9E9E9E'
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
    case 'SUSPENDED':
      return '已暂停'
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

const firstPage = () => {
  if (currentPage.value === 0) return
  loadQuestions(0)
}

const pageJumpInput = ref('')
const isJumpingToLast = ref(false)

watch(currentPage, (n) => {
  pageJumpInput.value = String(Math.max(1, n + 1))
}, { immediate: true })

const jumpToPage = () => {
  const raw = pageJumpInput.value.trim()
  if (!raw) {
    pageJumpInput.value = String(currentPage.value + 1)
    return
  }
  const n = Number(raw)
  if (!Number.isFinite(n) || n < 1) {
    pageJumpInput.value = String(currentPage.value + 1)
    return
  }
  loadQuestions(Math.floor(n) - 1)
}

const lastPage = async () => {
  if (!hasNext.value || isJumpingToLast.value) return
  isJumpingToLast.value = true
  try {
    let lo = currentPage.value
    let hi = lo + 1
    while (await probeHasNext(hi)) {
      lo = hi
      hi = hi * 2
    }
    while (lo + 1 < hi) {
      const mid = (lo + hi) >> 1
      if (await probeHasNext(mid)) lo = mid
      else hi = mid
    }
    loadQuestions(lo)
  } finally {
    isJumpingToLast.value = false
  }
}

const goToRecycleBin = () => {
  router.push('/recycle-bin')
}
</script>

<template>
  <div ref="containerRef" class="questions-container">
    <h1 ref="titleRef" class="page-title">题目管理</h1>

    <!-- 工具栏 -->
    <div ref="toolbarRef" class="toolbar">
      <div class="toolbar-left">
        <div class="search-wrap">
          <input
            v-model="searchKeyword"
            type="text"
            class="search-input"
            placeholder="搜索题号 / 题名 / 知识点（纯数字按 ID 精确）"
          />
          <button v-if="isSearching" class="search-clear" @click="clearSearch" title="清除">×</button>
        </div>
        <select v-model="subjectFilter" class="subject-filter">
          <option value="ALL">全部科目</option>
          <option v-for="subject in subjects" :key="subject" :value="subject">
            {{ subject }}
          </option>
        </select>
        <select v-model="stateFilter" class="state-filter">
          <option value="ALL">全部状态</option>
          <option v-for="s in STATE_OPTIONS" :key="s.value" :value="s.value">{{ s.label }}</option>
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

    <div v-if="errorMsg" class="error-bar">
      {{ errorMsg }}
    </div>

    <!-- 题目列表 -->
    <div class="questions-list">
      <div
        v-for="question in filteredQuestions"
        :key="question.id"
        class="question-item"
        :ref="setFirstItemRef"
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
      <div ref="paginationRef" class="pagination">
        <button class="btn" @click="firstPage" :disabled="currentPage === 0">« 首页</button>
        <button class="btn" @click="prevPage" :disabled="currentPage === 0">上一页</button>
        <div class="page-indicator">
          第
          <input
            v-model="pageJumpInput"
            type="number"
            min="1"
            class="page-jump-input"
            @keydown.enter="jumpToPage"
            @blur="jumpToPage"
          />
          页
        </div>
        <button class="btn" @click="nextPage" :disabled="!hasNext">下一页</button>
        <button class="btn" @click="lastPage" :disabled="!hasNext || isJumpingToLast">
          {{ isJumpingToLast ? '定位中…' : '末页 »' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.questions-container {
  width: 100%;
  max-width: 800px;
  margin: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
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
  align-items: center;
}

.toolbar-right {
  display: flex;
  gap: 12px;
}

.search-wrap {
  position: relative;
  flex: 1;
  min-width: 240px;
  max-width: 320px;
}

.search-input {
  width: 100%;
  padding: 10px 32px 10px 16px;
  border: 1px solid #ddd;
  border-radius: 8px;
  background-color: #fff;
  color: #333;
  font-size: 14px;
  box-sizing: border-box;
}

.search-input:focus {
  outline: none;
  border-color: #4CAF50;
}

.search-clear {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  border: none;
  background: transparent;
  font-size: 18px;
  color: #999;
  cursor: pointer;
  padding: 0 4px;
  line-height: 1;
}

.search-clear:hover {
  color: #333;
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

.error-bar {
  margin-bottom: 16px;
  padding: 10px 16px;
  background-color: #fdecea;
  color: #b71c1c;
  border: 1px solid #f5c2c0;
  border-radius: 6px;
  font-size: 13px;
}

.questions-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  flex: 0 1 auto;
  min-height: 0;
  max-height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
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
  word-break: break-word;
  overflow-wrap: anywhere;
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
  word-break: break-word;
  overflow-wrap: anywhere;
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

.pagination {
  flex-shrink: 0;
  background-color: #f5f5f5;
  padding: 14px 0;
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  box-shadow: 0 -4px 12px rgba(0, 0, 0, 0.08);
}

.page-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
}

.page-jump-input {
  width: 60px;
  padding: 6px 8px;
  border: 1px solid #ddd;
  border-radius: 6px;
  background-color: #fff;
  color: #333;
  font-size: 14px;
  text-align: center;
  -moz-appearance: textfield;
}

.page-jump-input::-webkit-outer-spin-button,
.page-jump-input::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.page-jump-input:focus {
  outline: none;
  border-color: #4CAF50;
}
</style>