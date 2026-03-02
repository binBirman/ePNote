<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { getQuestionData, deleteQuestion, updateQuestion, getImageBase64 } from '@/api/question'
import type { QuestionInfo, QuestionImage } from '@/types/question'
import type { QuestionState } from '@/types/question'

const router = useRouter()
const route = useRoute()

const question = ref<QuestionInfo | null>(null)
const isLoading = ref(true)
const isDeleting = ref(false)
const isEditing = ref(false)
const error = ref<string | null>(null)
const answerVisible = ref(false)

// 图片 base64 数据
const questionImages = ref<QuestionImage[]>([])
const answerImages = ref<QuestionImage[]>([])

// 编辑状态
const editForm = ref({
  name: '',
  subject: '',
  knowledge_points: [] as string[],
})

onMounted(async () => {
  const id = Number(route.params.id)
  try {
    question.value = await getQuestionData(id)
    if (!question.value) {
      error.value = '题目不存在'
      router.push('/questions')
    }
    // 初始化编辑表单
    if (question.value) {
      editForm.value = {
        name: question.value.name || '',
        subject: question.value.subject || '',
        knowledge_points: question.value.knowledge_points || [],
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
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : '获取题目详情失败'
    console.error('Failed to load question:', e)
  } finally {
    isLoading.value = false
  }
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

const handleEdit = () => {
  if (!question.value) return
  // 进入编辑模式
  editForm.value = {
    name: question.value.name || '',
    subject: question.value.subject || '',
    knowledge_points: question.value.knowledge_points || [],
  }
  isEditing.value = true
}

const handleSaveEdit = async () => {
  if (!question.value) return

  if (!editForm.value.name.trim()) {
    alert('请输入题目名')
    return
  }

  try {
    isEditing.value = true
    await updateQuestion(question.value.id, {
      name: editForm.value.name,
      subject: editForm.value.subject,
      knowledge_points: editForm.value.knowledge_points,
    })
    // 重新加载数据
    question.value = await getQuestionData(question.value.id)
    isEditing.value = false
    alert('保存成功！')
  } catch (e) {
    error.value = e instanceof Error ? e.message : '保存失败'
    console.error('Failed to update question:', e)
  } finally {
    isEditing.value = false
  }
}

const handleCancelEdit = () => {
  isEditing.value = false
  // 恢复原始数据
  if (question.value) {
    editForm.value = {
      name: question.value.name || '',
      subject: question.value.subject || '',
      knowledge_points: question.value.knowledge_points || [],
    }
  }
}

const handleDelete = async () => {
  if (!question.value || !confirm('确定要删除这道题目吗？题目将移至回收站。')) {
    return
  }

  isDeleting.value = true
  try {
    await deleteQuestion(question.value.id)
    router.push('/questions')
  } catch (e) {
    error.value = e instanceof Error ? e.message : '删除失败'
    console.error('Failed to delete question:', e)
  } finally {
    isDeleting.value = false
  }
}

const goBack = () => {
  router.push('/questions')
}
</script>

<template>
  <div class="detail-container">
    <button class="back-link" @click="goBack">
      ← 返回列表
    </button>

    <div v-if="isLoading" class="loading">
      加载中...
    </div>

    <div v-else-if="error" class="error">
      {{ error }}
    </div>

    <div v-else-if="question && !isDeleting" class="detail-card">
      <!-- 编辑模式 -->
      <template v-if="isEditing">
        <h2 class="section-title">编辑题目</h2>
        <div class="edit-form">
          <div class="form-group">
            <label class="form-label">题目名</label>
            <input v-model="editForm.name" type="text" class="form-input" placeholder="请输入题目名" />
          </div>
          <div class="form-group">
            <label class="form-label">科目</label>
            <input v-model="editForm.subject" type="text" class="form-input" placeholder="请输入科目" />
          </div>
          <div class="form-group">
            <label class="form-label">知识点（用逗号分隔）</label>
            <input
              :value="editForm.knowledge_points.join(', ')"
              @input="editForm.knowledge_points = ($event.target as HTMLInputElement).value.split(',').map(s => s.trim()).filter(Boolean)"
              type="text"
              class="form-input"
              placeholder="请输入知识点，用逗号分隔"
            />
          </div>
        </div>
        <div class="action-buttons">
          <button class="action-btn cancel" @click="handleCancelEdit">取消</button>
          <button class="action-btn save" @click="handleSaveEdit">保存</button>
        </div>
      </template>

      <!-- 查看模式 -->
      <template v-else>
        <!-- 题目标题和状态 -->
        <div class="detail-header">
          <h1 class="question-title">{{ question.name || '未命名题目' }}</h1>
          <span
            class="state-badge large"
            :style="{ backgroundColor: getStateColor(question.state as QuestionState) }"
          >
            {{ getStateLabel(question.state as QuestionState) }}
          </span>
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
          <span class="info-label">上次复习：</span>
          <span class="info-value">{{ question.last_reviewed_at || '从未' }}</span>
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

        <!-- 答案区域（图片）- 默认隐藏，点击显示 -->
        <div v-if="question.answer_images.length > 0" class="answer-section">
          <div class="answer-header">
            <h3 class="section-title">答案</h3>
            <button class="toggle-answer-btn" @click="answerVisible = !answerVisible">
              {{ answerVisible ? '点击隐藏答案' : '点击显示答案' }}
            </button>
          </div>
          <div v-if="answerVisible && answerImages.length > 0" class="images-grid">
            <div v-for="(img, index) in answerImages" :key="index" class="image-item answer">
              <img :src="img.path" :alt="'答案图片 ' + (index + 1)" class="answer-img" />
            </div>
          </div>
          <div v-else class="answer-hidden-placeholder">
            <span>答案已隐藏，点击上方按钮显示</span>
          </div>
        </div>

        <!-- 操作按钮 -->
        <div class="action-buttons">
          <button class="action-btn edit" @click="handleEdit">
            编辑
          </button>
          <button class="action-btn delete" @click="handleDelete">
            删除
          </button>
        </div>
      </template>
    </div>

    <div v-else-if="isDeleting" class="loading">
      删除中...
    </div>
  </div>
</template>

<style scoped>
.detail-container {
  width: 100%;
  max-width: 1000px;
  margin: 0 auto;
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

.question-img,
.answer-img {
  width: 100%;
  height: auto;
  max-height: 600px;
  object-fit: contain;
  border-radius: 4px;
}

.placeholder-text {
  color: #999;
  font-size: 14px;
  text-align: center;
  padding: 8px;
  word-break: break-all;
}

.answer-section {
  margin-bottom: 24px;
}

.answer-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.toggle-answer-btn {
  padding: 8px 16px;
  background-color: #ff9800;
  border: none;
  border-radius: 6px;
  color: #ffffff;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.toggle-answer-btn:hover {
  background-color: #f57c00;
}

.answer-hidden-placeholder {
  background-color: #f5f5f5;
  border: 2px dashed #ddd;
  border-radius: 8px;
  height: 100px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #999;
  font-size: 14px;
}

.image-item.answer {
  border-color: #ff9800;
}

/* 编辑模式样式 */
.edit-form {
  margin-bottom: 24px;
}

.form-group {
  margin-bottom: 16px;
}

.form-label {
  display: block;
  color: #333;
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 8px;
}

.form-input {
  width: 100%;
  padding: 12px 16px;
  border: 1px solid #ddd;
  border-radius: 8px;
  font-size: 15px;
  color: #333;
  box-sizing: border-box;
}

.form-input:focus {
  outline: none;
  border-color: #4CAF50;
}

.section-title {
  font-size: 16px;
  color: #333;
  margin-bottom: 16px;
  font-weight: 600;
}

.answer-image-placeholder {
  background-color: #fff;
  border: 2px dashed #ddd;
  border-radius: 8px;
  height: 250px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.action-buttons {
  display: flex;
  gap: 12px;
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

.action-btn.edit {
  background-color: #2196F3;
}

.action-btn.edit:hover {
  background-color: #1976D2;
}

.action-btn.save {
  background-color: #4CAF50;
}

.action-btn.save:hover {
  background-color: #45a049;
}

.action-btn.cancel {
  background-color: #9e9e9e;
}

.action-btn.cancel:hover {
  background-color: #757575;
}

.action-btn.delete {
  background-color: #f44336;
}

.action-btn.delete:hover {
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
