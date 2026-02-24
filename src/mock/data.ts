export type QuestionState = 'NEW' | 'LEARNING' | 'STABLE'

export interface Question {
  id: number
  name: string // 题目名
  subject: string // 科目
  knowledgePoint: string // 知识点
  questionImage: string // 题目图
  answerImage: string // 答案图
  state: QuestionState
  lastReviewed?: string
  createdDate: string
  deletedDate?: string // 删除日期（用于回收站）
}

export interface ReviewResult {
  questionId: number
  result: 'WRONG' | 'FUZZY' | 'CORRECT'
  timestamp: string
}

export interface Stats {
  totalQuestions: number
  todayPending: number
  yesterdayReviewed: number
  stateDistribution: {
    NEW: number
    LEARNING: number
    STABLE: number
  }
}

// Mock 数据
export const mockQuestions: Question[] = [
  {
    id: 1,
    name: 'Vue响应式原理',
    subject: '前端开发',
    knowledgePoint: 'Vue3响应式',
    questionImage: 'question-1.png',
    answerImage: 'answer-1.png',
    state: 'NEW',
    createdDate: '2026-02-20'
  },
  {
    id: 2,
    name: 'TypeScript类型系统',
    subject: '前端开发',
    knowledgePoint: 'TypeScript类型',
    questionImage: 'question-2.png',
    answerImage: 'answer-2.png',
    state: 'LEARNING',
    lastReviewed: '2026-02-23',
    createdDate: '2026-02-18'
  },
  {
    id: 3,
    name: 'HTTP状态码',
    subject: '网络协议',
    knowledgePoint: 'HTTP状态码',
    questionImage: 'question-3.png',
    answerImage: 'answer-3.png',
    state: 'STABLE',
    lastReviewed: '2026-02-23',
    createdDate: '2026-02-15'
  },
  {
    id: 4,
    name: '闭包概念',
    subject: 'JavaScript',
    knowledgePoint: '闭包',
    questionImage: 'question-4.png',
    answerImage: 'answer-4.png',
    state: 'NEW',
    createdDate: '2026-02-21'
  },
  {
    id: 5,
    name: 'Promise状态',
    subject: 'JavaScript',
    knowledgePoint: 'Promise',
    questionImage: 'question-5.png',
    answerImage: 'answer-5.png',
    state: 'LEARNING',
    lastReviewed: '2026-02-22',
    createdDate: '2026-02-19'
  }
]

export const mockStats: Stats = {
  totalQuestions: 5,
  todayPending: 3,
  yesterdayReviewed: 2,
  stateDistribution: {
    NEW: 2,
    LEARNING: 2,
    STABLE: 1
  }
}

// 初始化状态
export let initialized = false
export let selectedDirectory = ''

// 模拟初始化
export function mockInit(directory: string) {
  selectedDirectory = directory
  initialized = true
}

// 获取题目
export function getQuestions(): Question[] {
  return [...mockQuestions]
}

// 获取题目
export function getQuestionById(id: number): Question | undefined {
  return mockQuestions.find(q => q.id === id)
}

// 添加题目
export function addQuestion(question: Omit<Question, 'id' | 'createdDate'>): Question {
  const newQuestion: Question = {
    ...question,
    id: Math.max(...mockQuestions.map(q => q.id), 0) + 1,
    createdDate: new Date().toISOString().split('T')[0]
  }
  mockQuestions.push(newQuestion)
  return newQuestion
}

// 删除题目（移动到回收站）
export function deleteQuestion(id: number): void {
  moveToRecycleBin(id)
}

// 搜索题目（支持按题名和ID搜索）
export function searchQuestions(keyword: string): Question[] {
  if (!keyword) return [...mockQuestions]

  // 尝试作为ID搜索
  const idNumber = parseInt(keyword)
  if (!isNaN(idNumber)) {
    const byId = mockQuestions.find(q => q.id === idNumber)
    if (byId) return [byId]
  }

  // 按名称搜索
  const lowerKeyword = keyword.toLowerCase()
  return mockQuestions.filter(q =>
    q.name.toLowerCase().includes(lowerKeyword) ||
    q.subject.toLowerCase().includes(lowerKeyword) ||
    q.knowledgePoint.toLowerCase().includes(lowerKeyword)
  )
}

// 按状态筛选题目
export function filterQuestionsByState(state: QuestionState | 'ALL'): Question[] {
  if (state === 'ALL') return [...mockQuestions]
  return mockQuestions.filter(q => q.state === state)
}

// 重置初始化状态（用于测试）
export function resetInit() {
  initialized = false
  selectedDirectory = ''
}

// ========== 回收站功能 ==========

// 回收站数据
export const recycleBin: Question[] = []

// 获取所有科目
export function getSubjects(): string[] {
  const subjects = new Set(mockQuestions.map(q => q.subject))
  return Array.from(subjects).sort()
}

// 按科目筛选题目
export function filterQuestionsBySubject(subject: string | 'ALL'): Question[] {
  if (subject === 'ALL') return [...mockQuestions]
  return mockQuestions.filter(q => q.subject === subject)
}

// 按科目筛选回收站题目
export function filterRecycleBinBySubject(subject: string | 'ALL'): Question[] {
  if (subject === 'ALL') return [...recycleBin]
  return recycleBin.filter(q => q.subject === subject)
}

// 移动到回收站
export function moveToRecycleBin(id: number): void {
  const index = mockQuestions.findIndex(q => q.id === id)
  if (index !== -1) {
    const question = mockQuestions.splice(index, 1)[0]
    question.deletedDate = new Date().toISOString().split('T')[0]
    recycleBin.push(question)
  }
}

// 恢复题目
export function restoreFromRecycleBin(id: number): void {
  const index = recycleBin.findIndex(q => q.id === id)
  if (index !== -1) {
    const question = recycleBin.splice(index, 1)[0]
    delete question.deletedDate
    mockQuestions.push(question)
  }
}

// 永久删除
export function permanentDelete(id: number): void {
  const index = recycleBin.findIndex(q => q.id === id)
  if (index !== -1) {
    recycleBin.splice(index, 1)
  }
}
