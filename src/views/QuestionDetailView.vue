<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { getQuestionData, deleteQuestion, updateQuestion, getImageBase64, addQuestionImages, deleteQuestionImage, updateImageSortOrder } from '@/api/question'
import type { QuestionInfo, QuestionImage } from '@/types/question'
import type { QuestionState } from '@/types/question'
import { open } from "@tauri-apps/plugin-dialog";

interface ImageItem {
  path: string;
  base64: string;
  asset_id: string;
  sortOrder: number;
}

const router = useRouter()
const route = useRoute()

const question = ref<QuestionInfo | null>(null)
const isLoading = ref(true)
const isDeleting = ref(false)
const isEditing = ref(false)
const error = ref<string | null>(null)
const answerVisible = ref(false)

// 图片 base64 数据
const questionImages = ref<ImageItem[]>([])
const answerImages = ref<ImageItem[]>([])

// 编辑状态
const editForm = ref({
  name: '',
  subject: '',
  knowledge_points: [] as string[],
})
const newKnowledgePoint = ref('')

// 大图预览
const previewVisible = ref(false)
const previewImage = ref('')

// 添加知识点
const addKnowledgePoint = () => {
  const kp = newKnowledgePoint.value.trim()
  if (kp && !editForm.value.knowledge_points.includes(kp)) {
    editForm.value.knowledge_points.push(kp)
    newKnowledgePoint.value = ''
  }
}

// 移除知识点
const removeKnowledgePoint = (index: number) => {
  editForm.value.knowledge_points.splice(index, 1)
}

// 处理知识点输入框的回车事件
const handleKnowledgePointKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Enter') {
    e.preventDefault()
    addKnowledgePoint()
  }
}

// 加载图片 Base64
const loadImageBase64 = async (path: string): Promise<string> => {
  try {
    return await getImageBase64(path)
  } catch {
    return ''
  }
}

// 加载图片列表
const loadImages = async () => {
  if (!question.value) return

  const qImages = await Promise.all(
    question.value.question_images.map(async (img: QuestionImage) => ({
      path: img.path,
      base64: await loadImageBase64(img.path).catch(() => ''),
      asset_id: img.asset_id || '',
      sortOrder: img.sort_order || 0,
    }))
  )
  // 按 sortOrder 排序
  questionImages.value = qImages.filter((img: ImageItem) => img.base64).sort((a, b) => a.sortOrder - b.sortOrder)

  const aImages = await Promise.all(
    question.value.answer_images.map(async (img: QuestionImage) => ({
      path: img.path,
      base64: await loadImageBase64(img.path).catch(() => ''),
      asset_id: img.asset_id || '',
      sortOrder: img.sort_order || 0,
    }))
  )
  // 按 sortOrder 排序
  answerImages.value = aImages.filter((img: ImageItem) => img.base64).sort((a, b) => a.sortOrder - b.sortOrder)
}

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
      await loadImages()
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
  newKnowledgePoint.value = ''
  editForm.value = {
    name: question.value.name || '',
    subject: question.value.subject || '',
    knowledge_points: question.value.knowledge_points || [],
  }
  isEditing.value = true
}

// 预览大图
const showPreview = (base64: string) => {
  previewImage.value = base64
  previewVisible.value = true
}

// 关闭预览
const closePreview = () => {
  previewVisible.value = false
  previewImage.value = ''
}

// 向上移动图片
const moveQuestionImageUp = async (index: number) => {
  if (index <= 0) return
  // 交换数组中的位置
  const temp = questionImages.value[index]
  questionImages.value[index] = questionImages.value[index - 1]
  questionImages.value[index - 1] = temp
  // 更新排序
  questionImages.value.forEach((item, i) => {
    item.sortOrder = i
  })
  // 同步到后端
  await syncQuestionImageOrder()
}

const moveQuestionImageDown = async (index: number) => {
  if (index >= questionImages.value.length - 1) return
  const temp = questionImages.value[index]
  questionImages.value[index] = questionImages.value[index + 1]
  questionImages.value[index + 1] = temp
  // 更新排序
  questionImages.value.forEach((item, i) => {
    item.sortOrder = i
  })
  // 同步到后端
  await syncQuestionImageOrder()
}

const moveAnswerImageUp = async (index: number) => {
  if (index <= 0) return
  const temp = answerImages.value[index]
  answerImages.value[index] = answerImages.value[index - 1]
  answerImages.value[index - 1] = temp
  // 更新排序
  answerImages.value.forEach((item, i) => {
    item.sortOrder = i
  })
  // 同步到后端
  await syncAnswerImageOrder()
}

const moveAnswerImageDown = async (index: number) => {
  if (index >= answerImages.value.length - 1) return
  const temp = answerImages.value[index]
  answerImages.value[index] = answerImages.value[index + 1]
  answerImages.value[index + 1] = temp
  // 更新排序
  answerImages.value.forEach((item, i) => {
    item.sortOrder = i
  })
  // 同步到后端
  await syncAnswerImageOrder()
}

// 同步题目图片排序到后端
const syncQuestionImageOrder = async () => {
  if (!question.value) return
  const updates = questionImages.value.map((img, i) => ({
    asset_id: img.asset_id,
    sort_order: i + 1,
  }))
  try {
    await updateImageSortOrder(question.value.id, 'QUESTION', updates)
  } catch (e) {
    console.error('Failed to update question image order:', e)
  }
}

// 同步答案图片排序到后端
const syncAnswerImageOrder = async () => {
  if (!question.value) return
  const updates = answerImages.value.map((img, i) => ({
    asset_id: img.asset_id,
    sort_order: i + 1,
  }))
  try {
    await updateImageSortOrder(question.value.id, 'ANSWER', updates)
  } catch (e) {
    console.error('Failed to update answer image order:', e)
  }
}

// 选择并添加题目图片
const selectQuestionImages = async () => {
  if (!question.value) return
  try {
    const res = await open({
      multiple: true,
      filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp'] }]
    })
    if (res) {
      const paths = Array.isArray(res) ? res : [res]
      const pathArray = paths.map(p => p as string)

      // 调用后端添加图片
      await addQuestionImages(question.value.id, pathArray, 'question')

      // 重新加载数据
      question.value = await getQuestionData(question.value.id)
      await loadImages()
    }
  } catch (e) {
    console.error('Failed to add images:', e)
    alert('添加图片失败: ' + e)
  }
}

// 选择并添加答案图片
const selectAnswerImages = async () => {
  if (!question.value) return
  try {
    const res = await open({
      multiple: true,
      filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp'] }]
    })
    if (res) {
      const paths = Array.isArray(res) ? res : [res]
      const pathArray = paths.map(p => p as string)

      // 调用后端添加图片
      await addQuestionImages(question.value.id, pathArray, 'answer')

      // 重新加载数据
      question.value = await getQuestionData(question.value.id)
      await loadImages()
    }
  } catch (e) {
    console.error('Failed to add images:', e)
    alert('添加图片失败: ' + e)
  }
}

// 删除题目图片
const removeQuestionImage = async (index: number) => {
  const img = questionImages.value[index]
  if (!img.asset_id) return

  if (!confirm('确定要删除这张图片吗？')) return

  try {
    await deleteQuestionImage(img.asset_id)
    // 从列表中移除
    questionImages.value.splice(index, 1)
    // 更新排序
    questionImages.value.forEach((item, i) => {
      item.sortOrder = i
    })
  } catch (e) {
    console.error('Failed to delete image:', e)
    alert('删除图片失败: ' + e)
  }
}

// 删除答案图片
const removeAnswerImage = async (index: number) => {
  const img = answerImages.value[index]
  if (!img.asset_id) return

  if (!confirm('确定要删除这张图片吗？')) return

  try {
    await deleteQuestionImage(img.asset_id)
    // 从列表中移除
    answerImages.value.splice(index, 1)
    // 更新排序
    answerImages.value.forEach((item, i) => {
      item.sortOrder = i
    })
  } catch (e) {
    console.error('Failed to delete image:', e)
    alert('删除图片失败: ' + e)
  }
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
    await loadImages()
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
  newKnowledgePoint.value = ''
  // 恢复原始数据
  if (question.value) {
    editForm.value = {
      name: question.value.name || '',
      subject: question.value.subject || '',
      knowledge_points: question.value.knowledge_points || [],
    }
    // 重新加载图片
    loadImages()
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
            <label class="form-label">知识点（可多次添加）</label>
            <div class="knowledge-point-input-group">
              <input
                v-model="newKnowledgePoint"
                type="text"
                class="form-input"
                placeholder="输入知识点后点击添加"
                @keydown="handleKnowledgePointKeydown"
              />
              <button type="button" class="add-kp-btn" @click="addKnowledgePoint">添加</button>
            </div>
            <div v-if="editForm.knowledge_points.length > 0" class="knowledge-points-list">
              <div v-for="(kp, index) in editForm.knowledge_points" :key="index" class="knowledge-point-item">
                <span class="kp-text">{{ kp }}</span>
                <button type="button" class="remove-kp-btn" @click="removeKnowledgePoint(index)">×</button>
              </div>
            </div>
            <div v-else class="kp-empty-hint">暂无知识点，请添加</div>
          </div>

          <!-- 题目图管理 -->
          <div class="form-group">
            <label class="form-label">题目图片</label>
            <div class="image-upload-area">
              <div v-if="questionImages.length === 0" class="upload-placeholder" @click="selectQuestionImages">
                <span class="upload-icon">📷</span>
                <span class="upload-text">点击添加题目图片</span>
              </div>
              <div v-else class="upload-preview-list">
                <div
                  v-for="(img, index) in questionImages"
                  :key="'q'+index"
                  class="upload-preview-item"
                >
                  <div class="drag-handle">
                    <span class="sort-number">{{ index + 1 }}</span>
                  </div>
                  <img :src="img.base64" :alt="'题目图片 ' + (index + 1)" class="preview-thumbnail" @click="showPreview(img.base64)" />
                  <div class="preview-actions">
                    <button type="button" class="move-btn" @click.stop="moveQuestionImageUp(index)" :disabled="index === 0" title="上移">↑</button>
                    <button type="button" class="move-btn" @click.stop="moveQuestionImageDown(index)" :disabled="index === questionImages.length - 1" title="下移">↓</button>
                    <button type="button" class="remove-btn" @click.stop="removeQuestionImage(index)">删除</button>
                  </div>
                </div>
                <button type="button" class="add-more-btn" @click.stop="selectQuestionImages">+ 添加更多</button>
              </div>
            </div>
          </div>

          <!-- 答案图管理 -->
          <div class="form-group">
            <label class="form-label">答案图片</label>
            <div class="image-upload-area">
              <div v-if="answerImages.length === 0" class="upload-placeholder" @click="selectAnswerImages">
                <span class="upload-icon">📷</span>
                <span class="upload-text">点击添加答案图片</span>
              </div>
              <div v-else class="upload-preview-list">
                <div
                  v-for="(img, index) in answerImages"
                  :key="'a'+index"
                  class="upload-preview-item"
                >
                  <div class="drag-handle">
                    <span class="sort-number">{{ index + 1 }}</span>
                  </div>
                  <img :src="img.base64" :alt="'答案图片 ' + (index + 1)" class="preview-thumbnail" @click="showPreview(img.base64)" />
                  <div class="preview-actions">
                    <button type="button" class="move-btn" @click.stop="moveAnswerImageUp(index)" :disabled="index === 0" title="上移">↑</button>
                    <button type="button" class="move-btn" @click.stop="moveAnswerImageDown(index)" :disabled="index === answerImages.length - 1" title="下移">↓</button>
                    <button type="button" class="remove-btn" @click.stop="removeAnswerImage(index)">删除</button>
                  </div>
                </div>
                <button type="button" class="add-more-btn" @click.stop="selectAnswerImages">+ 添加更多</button>
              </div>
            </div>
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
              <img :src="img.base64" :alt="'题目图片 ' + (index + 1)" class="question-img" @click="showPreview(img.base64)" />
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
              <img :src="img.base64" :alt="'答案图片 ' + (index + 1)" class="answer-img" @click="showPreview(img.base64)" />
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

    <!-- 大图预览弹窗 -->
    <div v-if="previewVisible" class="preview-modal" @click="closePreview">
      <div class="preview-modal-content" @click.stop>
        <img :src="previewImage" alt="预览大图" class="preview-modal-image" />
        <button class="preview-modal-close" @click="closePreview">×</button>
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
  cursor: pointer;
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

/* 知识点输入样式 */
.knowledge-point-input-group {
  display: flex;
  gap: 8px;
}

.knowledge-point-input-group .form-input {
  flex: 1;
}

.add-kp-btn {
  padding: 10px 20px;
  background-color: #4CAF50;
  border: none;
  border-radius: 8px;
  color: #ffffff;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.add-kp-btn:hover {
  background-color: #45a049;
}

.knowledge-points-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 8px;
}

.knowledge-point-item {
  display: flex;
  align-items: center;
  gap: 6px;
  background-color: #e8f5e9;
  border: 1px solid #4CAF50;
  border-radius: 16px;
  padding: 6px 12px;
}

.kp-text {
  color: #2e7d32;
  font-size: 14px;
}

.remove-kp-btn {
  background: none;
  border: none;
  color: #4CAF50;
  font-size: 18px;
  cursor: pointer;
  padding: 0;
  line-height: 1;
}

.remove-kp-btn:hover {
  color: #d32f2f;
}

.kp-empty-hint {
  color: #999;
  font-size: 13px;
  margin-top: 8px;
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

/* 图片上传区域样式 */
.image-upload-area {
  width: 100%;
}

.upload-placeholder {
  background-color: #fafafa;
  border: 2px dashed #ddd;
  border-radius: 8px;
  height: 150px;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.upload-placeholder:hover {
  border-color: #4CAF50;
  background-color: #f0f8f0;
}

.upload-icon {
  font-size: 32px;
}

.upload-text {
  color: #999;
  font-size: 14px;
}

.upload-preview-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.upload-preview-item {
  background-color: #fff;
  border: 2px solid #4CAF50;
  border-radius: 8px;
  padding: 8px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.drag-handle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background-color: #4CAF50;
  border-radius: 4px;
  flex-shrink: 0;
}

.sort-number {
  color: #fff;
  font-size: 14px;
  font-weight: 600;
}

.preview-thumbnail {
  width: 120px;
  height: 90px;
  object-fit: cover;
  border-radius: 4px;
  cursor: pointer;
  border: 1px solid #eee;
}

.preview-actions {
  display: flex;
  gap: 8px;
  margin-left: auto;
}

.add-more-btn {
  padding: 8px 16px;
  background-color: #f0f0f0;
  border: 2px dashed #ccc;
  border-radius: 8px;
  color: #666;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.add-more-btn:hover {
  border-color: #4CAF50;
  color: #4CAF50;
  background-color: #f8f8f8;
}

.move-btn {
  padding: 6px 12px;
  background-color: #2196F3;
  border: none;
  border-radius: 4px;
  color: #ffffff;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.move-btn:hover:not(:disabled) {
  background-color: #1976D2;
}

.move-btn:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.remove-btn {
  padding: 6px 16px;
  background-color: #f44336;
  border: none;
  border-radius: 6px;
  color: #ffffff;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.remove-btn:hover {
  background-color: #d32f2f;
}

/* 大图预览弹窗样式 */
.preview-modal {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.preview-modal-content {
  position: relative;
  max-width: 90%;
  max-height: 90%;
}

.preview-modal-image {
  max-width: 100%;
  max-height: 90vh;
  object-fit: contain;
  border-radius: 4px;
}

.preview-modal-close {
  position: absolute;
  top: -40px;
  right: 0;
  background: none;
  border: none;
  color: #fff;
  font-size: 32px;
  cursor: pointer;
  padding: 4px 12px;
}

.preview-modal-close:hover {
  color: #ccc;
}
</style>
