<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { getQuestionData, restoreQuestion, permanentlyDeleteQuestion, getImageBase64 } from '@/api/question'
import type { QuestionInfo, QuestionImage } from '@/types/question'

const router = useRouter()
const route = useRoute()

const question = ref<QuestionInfo | null>(null)
const isLoading = ref(true)
const isProcessing = ref(false)
const error = ref<string | null>(null)

// 图片 base64 数据
const questionImages = ref<QuestionImage[]>([])
const answerImages = ref<QuestionImage[]>([])

onMounted(async () => {
  const id = Number(route.params.id)
  try {
    question.value = await getQuestionData(id)
    if (!question.value) {
      error.value = '题目不存在或已被永久删除'
      return
    }
    // 加载图片 base64 数据
    const qImages = await Promise.all(
      question.value.question_images.map(async (img: QuestionImage) => ({
        path: await getImageBase64(img.path).catch(() => ''),
        asset_id: img.asset_id
      }))
    )
    questionImages.value = qImages.filter((img: QuestionImage) => img.path)
    const aImages = await Promise.all(
      question.value.answer_images.map(async (img: QuestionImage) => ({
        path: await getImageBase64(img.path).catch(() => ''),
        asset_id: img.asset_id
      }))
    )
    answerImages.value = aImages.filter((img: QuestionImage) => img.path)
  } catch (e) {
    error.value = e instanceof Error ? e.message : '获取题目详情失败'
    console.error('Failed to load question:', e)
  } finally {
    isLoading.value = false
  }
})

const handleRestore = async () => {
  if (!question.value) return

  if (!confirm('确定要恢复这道题目吗？')) {
    return
  }

  isProcessing.value = true
  try {
    await restoreQuestion(question.value.id)
    alert('恢复成功！')
    router.push('/recycle-bin')
  } catch (e) {
    error.value = e instanceof Error ? e.message : '恢复失败'
    console.error('Failed to restore question:', e)
  } finally {
    isProcessing.value = false
  }
}

const handlePermanentDelete = async () => {
  if (!question.value) return

  if (!confirm('这道题目吗？确定要永久删除此操作不可恢复！')) {
    return
  }

  isProcessing.value = true
  try {
    await permanentlyDeleteQuestion(question.value.id)
    alert('永久删除成功！')
    router.push('/recycle-bin')
  } catch (e) {
    error.value = e instanceof Error ? e.message : '删除失败'
    console.error('Failed to permanently delete question:', e)
  } finally {
    isProcessing.value = false
  }
}

const goBack = () => {
  router.push('/recycle-bin')
}
</script>

<template>
  <div class="detail-container">
    <button class="back-link" @click="goBack">
      ← 返回回收站
    </button>

    <div v-if="isLoading" class="loading">
      加载中...
    </div>

    <div v-else-if="error" class="error">
      {{ error }}
    </div>

    <div v-else-if="question" class="detail-card">
      <!-- 题目标题和状态 -->
      <div class="detail-header">
        <h1 class="question-title">{{ question.name || '未命名题目' }}</h1>
        <span class="state-badge large">已删除</span>
      </div>

      <!-- 题目信息 -->
      <div class="info-row">
        <span class="info-label">题目 ID：</span>
        <span class="info-value">{{ question.id }}</span>
      </div>
      <div class="info-row">
        <span class="info-label">科目：</span>
        <span class="info-value">{{ question.subject || '未设置' }}</span>
      </div>
      <div class="info-row">
        <span class="info-label">知识点：</span>
        <span class="info-value">{{ question.knowledge_points.join(', ') || '无' }}</span>
      </div>
      <div class="info-row">
        <span class="info-label">创建日期：</span>
        <span class="info-value">{{ question.created_at }}</span>
      </div>
      <div class="info-row">
        <span class="info-label">删除日期：</span>
        <span class="info-value">{{ question.last_reviewed_at || '未知' }}</span>
      </div>

      <!-- 题目图区域 -->
      <div v-if="questionImages.length > 0" class="images-section">
        <h3 class="section-title">题目图片</h3>
        <div class="images-grid">
          <div v-for="(img, index) in questionImages" :key="index" class="image-item">
            <img :src="img.path" :alt="'题目图片 ' + (index + 1)" class="question-img" />
          </div>
        </div>
      </div>
      <div v-else class="images-section">
        <h3 class="section-title">题目图片</h3>
        <div class="no-image-hint">暂无题目图片</div>
      </div>

      <!-- 答案区域 -->
      <div v-if="question.answer_images.length > 0" class="answer-section">
        <h3 class="section-title">答案图片</h3>
        <div class="images-grid">
          <div v-for="(img, index) in answerImages" :key="index" class="image-item answer">
            <img :src="img.path" :alt="'答案图片 ' + (index + 1)" class="answer-img" />
          </div>
        </div>
      </div>
      <div v-else class="answer-section">
        <h3 class="section-title">答案图片</h3>
        <div class="no-image-hint">暂无答案图片</div>
      </div>

      <!-- 操作按钮 -->
      <div class="action-buttons">
        <button class="action-btn restore" @click="handleRestore" :disabled="isProcessing">
          恢复题目
        </button>
        <button class="action-btn delete" @click="handlePermanentDelete" :disabled="isProcessing">
          永久删除
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.detail-container {
  width: 100%;
  max-width: 900px;
  margin: 0;
}

.back-link {
  background: none;
  border: none;
  color: #4CAF50;
  font-size: 14px;
  cursor: pointer;
  padding: 8px 0;
  margin-bottom: 20px;
  transition: color 0.2s;
}

.back-link:hover {
  color: #45a049;
}

.detail-card {
  background-color: #ffffff;
  border-radius: 12px;
  padding: 32px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
}

.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 24px;
  padding-bottom: 24px;
  border-bottom: 1px solid #e0e0e0;
}

.question-title {
  flex: 1;
  font-size: 24px;
  color: #333;
  line-height: 1.4;
  margin: 0;
}

.state-badge {
  padding: 6px 14px;
  border-radius: 14px;
  color: #ffffff;
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
  background-color: #f44336;
}

.state-badge.large {
  font-size: 14px;
  padding: 8px 16px;
}

.info-row {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
  color: #666;
  font-size: 14px;
}

.info-label {
  font-weight: 500;
}

.info-value {
  color: #333;
}

.images-section {
  margin: 24px 0;
}

.answer-section {
  margin-bottom: 24px;
}

.section-title {
  font-size: 16px;
  color: #333;
  margin-bottom: 16px;
  font-weight: 600;
}

.images-grid {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.image-item {
  background-color: #fafafa;
  border: none;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.image-item.answer {
  border-color: #ff9800;
}

.question-img,
.answer-img {
  width: 100%;
  height: auto;
  max-height: 600px;
  object-fit: contain;
  border-radius: 4px;
}

.no-image-hint {
  color: #999;
  font-size: 14px;
  padding: 20px;
  text-align: center;
  background-color: #fafafa;
  border-radius: 8px;
}

.action-buttons {
  display: flex;
  gap: 12px;
  margin-top: 24px;
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

.action-btn:disabled {
  opacity: 0.6;
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
