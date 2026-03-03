<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { recommendQuestions, processReview, listSubjects } from '@/api/review'
import { getQuestionData, getImageBase64 } from '@/api/question'
import type { RecommendQuestion, ReviewResult, QuestionImage } from '@/types/question'

const router = useRouter()
const route = useRoute()

// 推荐题目（简化数据）
const recommendQuestionsData = ref<RecommendQuestion[]>([])
const questionDetails = ref<Map<number, any>>(new Map())
// 图片 base64 数据
const questionImages = ref<Map<number, QuestionImage[]>>(new Map())
const answerImages = ref<Map<number, QuestionImage[]>>(new Map())
const currentIndex = ref(0)
const showAnswer = ref(false)
const reviewResults = ref<{ questionId: number; result: ReviewResult }[]>([])
const isComplete = ref(false)
const selectedSubject = ref<string>('ALL')
const reviewLimit = ref<number>(10)
const loading = ref(false)

onMounted(async () => {
  selectedSubject.value = (route.query.subject as string) || 'ALL'
  // 获取复习题数限制，默认为10
  const limitParam = route.query.limit as string
  reviewLimit.value = limitParam ? parseInt(limitParam, 10) : 10
  if (isNaN(reviewLimit.value) || reviewLimit.value < 1) {
    reviewLimit.value = 10
  }

  loading.value = true

  try {
    // 获取推荐题目
    const subject = selectedSubject.value === 'ALL' ? undefined : selectedSubject.value
    const result = await recommendQuestions(reviewLimit.value, subject)
    recommendQuestionsData.value = result.questions

    // 加载题目详情和图片
    for (const q of recommendQuestionsData.value) {
      try {
        const detail = await getQuestionData(q.id)
        questionDetails.value.set(q.id, detail)

        // 加载题目图片 base64
        const qImages = await Promise.all(
          detail.question_images.map(async (img: QuestionImage) => ({
            path: await getImageBase64(img.path).catch(() => ''),
            asset_id: img.asset_id
          }))
        )
        questionImages.value.set(q.id, qImages.filter((img: QuestionImage) => img.path))

        // 加载答案图片 base64
        const aImages = await Promise.all(
          detail.answer_images.map(async (img: QuestionImage) => ({
            path: await getImageBase64(img.path).catch(() => ''),
            asset_id: img.asset_id
          }))
        )
        answerImages.value.set(q.id, aImages.filter((img: QuestionImage) => img.path))
      } catch (e) {
        console.error(`加载题目 ${q.id} 详情失败:`, e)
      }
    }

    // 如果没有题目，返回复习页
    if (recommendQuestionsData.value.length === 0) {
      router.push('/review')
    }
  } catch (e) {
    console.error('加载推荐题目失败:', e)
    router.push('/review')
  } finally {
    loading.value = false
  }
})

const currentQuestion = computed(() => {
  const q = recommendQuestionsData.value[currentIndex.value]
  if (!q) return null
  return {
    ...q,
    detail: questionDetails.value.get(q.id)
  }
})

const progress = computed(() => `${currentIndex.value + 1} / ${recommendQuestionsData.value.length}`)

const toggleAnswer = () => {
  showAnswer.value = !showAnswer.value
}

const handleReview = async (result: ReviewResult) => {
  const q = currentQuestion.value
  if (!q) return

  // 调用后端 API 提交复习结果
  try {
    await processReview(q.id, result)
  } catch (e) {
    console.error('提交复习结果失败:', e)
  }

  reviewResults.value.push({
    questionId: q.id,
    result
  })

  showAnswer.value = false
  currentIndex.value++

  if (currentIndex.value >= recommendQuestionsData.value.length) {
    isComplete.value = true
  }
}

const goBack = () => {
  router.push('/review')
}
</script>

<template>
  <div class="session-container">
    <!-- 加载状态 -->
    <div v-if="loading" class="loading">
      <div class="loading-text">加载中...</div>
    </div>

    <div v-else-if="!isComplete && currentQuestion" class="review-session">
      <!-- 顶部进度 -->
      <div class="progress-bar">
        <div class="progress-info">{{ progress }}</div>
        <div class="progress-line">
          <div
            class="progress-fill"
            :style="{ width: ((currentIndex + 1) / recommendQuestionsData.length * 100) + '%' }"
          ></div>
        </div>
      </div>

      <!-- 题目区域 -->
      <div class="question-card">
        <!-- 题目信息 -->
        <div class="question-header">
          <div class="question-name">{{ currentQuestion.name || '未命名题目' }}</div>
          <div class="question-meta">
            <span class="meta-tag">{{ currentQuestion.detail?.subject || '未分类' }}</span>
            <span v-for="kp in (currentQuestion.detail?.knowledge_points || []).slice(0, 2)" :key="kp" class="meta-tag">{{ kp }}</span>
          </div>
        </div>

        <!-- 题目图区域 -->
        <div v-if="questionImages.get(currentQuestion.id)?.length" class="question-images">
          <img
            v-for="img in questionImages.get(currentQuestion.id)"
            :key="img.asset_id"
            :src="img.path"
            :alt="`题目图片`"
            class="question-image"
          />
        </div>
        <div v-else class="question-image-placeholder">
          <div class="placeholder-text">暂无题目图片</div>
        </div>

        <!-- 显示答案按钮 -->
        <button class="show-answer-btn" @click="toggleAnswer">
          {{ showAnswer ? '隐藏答案' : '显示答案' }}
        </button>

        <!-- 答案区域（图片） -->
        <div v-if="showAnswer" class="answer-section">
          <div class="answer-label">答案：</div>
          <div v-if="answerImages.get(currentQuestion.id)?.length" class="answer-images">
            <img
              v-for="img in answerImages.get(currentQuestion.id)"
              :key="img.asset_id"
              :src="img.path"
              :alt="`答案图片`"
              class="answer-image"
            />
          </div>
          <div v-else class="answer-image-placeholder">
            <div class="placeholder-text">暂无答案图片</div>
          </div>
        </div>
      </div>

      <!-- 底部操作按钮 -->
      <div v-if="showAnswer" class="action-buttons">
        <button class="action-btn wrong" @click="handleReview('WRONG')">
          不记得
        </button>
        <button class="action-btn fuzzy" @click="handleReview('FUZZY')">
          模糊
        </button>
        <button class="action-btn correct" @click="handleReview('CORRECT')">
          记得
        </button>
      </div>
    </div>

    <!-- 完成页面 -->
    <div v-else class="complete-view">
      <div class="complete-card">
        <div class="complete-icon">✓</div>
        <h2 class="complete-title">今日复习完成</h2>
        <p class="complete-info">共复习 {{ recommendQuestionsData.length }} 题</p>

        <div class="result-summary">
          <div class="result-item">
            <span class="result-count">{{ reviewResults.filter(r => r.result === 'CORRECT').length }}</span>
            <span class="result-label">记得</span>
          </div>
          <div class="result-item">
            <span class="result-count">{{ reviewResults.filter(r => r.result === 'FUZZY').length }}</span>
            <span class="result-label">模糊</span>
          </div>
          <div class="result-item">
            <span class="result-count">{{ reviewResults.filter(r => r.result === 'WRONG').length }}</span>
            <span class="result-label">不记得</span>
          </div>
        </div>

        <button class="back-btn" @click="goBack">
          返回复习首页
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.session-container {
  max-width: 700px;
}

.loading {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: calc(100vh - 60px);
}

.loading-text {
  color: #666;
  font-size: 16px;
}

.review-session {
  display: flex;
  flex-direction: column;
  min-height: calc(100vh - 60px);
}

.progress-bar {
  margin-bottom: 24px;
}

.progress-info {
  text-align: center;
  color: #666;
  font-size: 14px;
  margin-bottom: 8px;
}

.progress-line {
  height: 6px;
  background-color: #e0e0e0;
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background-color: #4CAF50;
  transition: width 0.3s ease;
}

.question-card {
  background-color: #ffffff;
  border-radius: 12px;
  padding: 32px;
  flex: 1;
  display: flex;
  flex-direction: column;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
}

.question-header {
  margin-bottom: 24px;
  padding-bottom: 20px;
  border-bottom: 1px solid #e0e0e0;
}

.question-name {
  font-size: 24px;
  font-weight: 600;
  color: #333;
  margin-bottom: 12px;
}

.question-meta {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.meta-tag {
  padding: 6px 12px;
  background-color: #f0f0f0;
  border-radius: 4px;
  font-size: 13px;
  color: #666;
}

.question-images {
  margin-bottom: 24px;
}

.question-image {
  max-width: 100%;
  border-radius: 8px;
  margin-bottom: 12px;
}

.question-image-placeholder {
  background-color: #fafafa;
  border: 2px dashed #ddd;
  border-radius: 8px;
  height: 300px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 24px;
}

.placeholder-text {
  color: #999;
  font-size: 14px;
}

.show-answer-btn {
  align-self: center;
  padding: 12px 32px;
  background-color: #fff;
  border: 1px solid #4CAF50;
  border-radius: 8px;
  color: #4CAF50;
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 24px;
  transition: all 0.2s;
}

.show-answer-btn:hover {
  background-color: #4CAF50;
  color: #ffffff;
}

.answer-section {
  background-color: #fafafa;
  border-radius: 8px;
  padding: 20px;
  border-left: 4px solid #4CAF50;
}

.answer-label {
  color: #4CAF50;
  font-weight: 600;
  margin-bottom: 12px;
}

.answer-images {
  margin-bottom: 12px;
}

.answer-image {
  max-width: 100%;
  border-radius: 8px;
  margin-bottom: 12px;
}

.answer-image-placeholder {
  background-color: #fff;
  border: 2px dashed #ddd;
  border-radius: 8px;
  height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.action-buttons {
  display: flex;
  gap: 12px;
  margin-top: 24px;
  padding-bottom: 60px;
}

.action-btn {
  flex: 1;
  padding: 14px;
  border: none;
  border-radius: 8px;
  font-size: 15px;
  font-weight: 600;
  transition: all 0.2s;
  color: #ffffff;
}

.action-btn.wrong {
  background-color: #f44336;
}

.action-btn.wrong:hover {
  background-color: #d32f2f;
}

.action-btn.fuzzy {
  background-color: #ff9800;
}

.action-btn.fuzzy:hover {
  background-color: #f57c00;
}

.action-btn.correct {
  background-color: #4CAF50;
}

.action-btn.correct:hover {
  background-color: #45a049;
}

.complete-view {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: calc(100vh - 60px);
}

.complete-card {
  background-color: #ffffff;
  border-radius: 16px;
  padding: 48px;
  text-align: center;
  max-width: 400px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
}

.complete-icon {
  font-size: 72px;
  color: #4CAF50;
  margin-bottom: 16px;
}

.complete-title {
  font-size: 28px;
  color: #333;
  margin-bottom: 8px;
}

.complete-info {
  color: #666;
  font-size: 16px;
  margin-bottom: 32px;
}

.result-summary {
  display: flex;
  justify-content: space-around;
  margin-bottom: 32px;
  padding: 24px;
  background-color: #fafafa;
  border-radius: 12px;
}

.result-item {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.result-count {
  font-size: 32px;
  font-weight: bold;
  color: #333;
  margin-bottom: 8px;
}

.result-label {
  font-size: 14px;
  color: #666;
}

.back-btn {
  width: 100%;
  padding: 16px;
  background-color: #4CAF50;
  border: none;
  border-radius: 8px;
  color: #ffffff;
  font-size: 16px;
  font-weight: 600;
  transition: all 0.2s;
}

.back-btn:hover {
  background-color: #45a049;
}
</style>
