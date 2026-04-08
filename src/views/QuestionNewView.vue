<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { createQuestion, getImageBase64 } from '@/api/question'
import { open } from "@tauri-apps/plugin-dialog";

interface ImageItem {
  path: string;
  base64: string;
  sortOrder: number;
}

const router = useRouter()

const questionName = ref('')
const questionSubject = ref('')
const questionKnowledgePoints = ref<string[]>([])
const newKnowledgePoint = ref('')
const questionQuestionImages = ref<ImageItem[]>([])
const questionAnswerImages = ref<ImageItem[]>([])
const loading = ref(false)

// 大图预览
const previewVisible = ref(false)
const previewImage = ref('')

// 添加知识点
const addKnowledgePoint = () => {
  const kp = newKnowledgePoint.value.trim()
  if (kp && !questionKnowledgePoints.value.includes(kp)) {
    questionKnowledgePoints.value.push(kp)
    newKnowledgePoint.value = ''
  }
}

// 移除知识点
const removeKnowledgePoint = (index: number) => {
  questionKnowledgePoints.value.splice(index, 1)
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

// 将文件路径转换为 Base64 缩略图
const loadImagePreview = async (paths: string[]): Promise<ImageItem[]> => {
  const items: ImageItem[] = []
  for (let i = 0; i < paths.length; i++) {
    const base64 = await loadImageBase64(paths[i])
    items.push({
      path: paths[i],
      base64,
      sortOrder: i,
    })
  }
  return items
}

// 拖拽排序 - 开始拖拽
let draggedItem: ImageItem | null = null
let draggedArray: 'question' | 'answer' | null = null

const onDragStart = (item: ImageItem, arrayType: 'question' | 'answer') => {
  draggedItem = item
  draggedArray = arrayType
}

// 拖拽排序 - 拖拽中
const onDragOver = (e: DragEvent) => {
  e.preventDefault()
}

// 拖拽排序 - 放置
const onDrop = (targetItem: ImageItem, arrayType: 'question' | 'answer') => {
  if (!draggedItem || draggedArray !== arrayType) return

  const array = arrayType === 'question' ? questionQuestionImages : questionAnswerImages
  const oldIndex = array.value.findIndex(item => item.path === draggedItem!.path)
  const newIndex = array.value.findIndex(item => item.path === targetItem.path)

  if (oldIndex !== -1 && newIndex !== -1) {
    // 移除旧位置的元素
    const [removed] = array.value.splice(oldIndex, 1)
    // 插入到新位置
    array.value.splice(newIndex, 0, removed)
    // 更新排序
    array.value.forEach((item, index) => {
      item.sortOrder = index
    })
  }

  draggedItem = null
  draggedArray = null
}

// 拖拽排序 - 拖拽结束
const onDragEnd = () => {
  draggedItem = null
  draggedArray = null
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

const handleSubmit = async () => {
  if (!questionName.value.trim()) {
    alert('请输入题目名')
    return
  }

  if (!questionSubject.value.trim()) {
    alert('请输入科目')
    return
  }

  if (questionKnowledgePoints.value.length === 0) {
    alert('请至少添加一个知识点')
    return
  }

  if (questionQuestionImages.value.length === 0) {
    alert('请至少选择一张题目图片')
    return
  }

  if (questionAnswerImages.value.length === 0) {
    alert('请至少选择一张答案图片')
    return
  }

  try {
    console.log('handleSubmit start')
    console.log('question_image_paths:', questionQuestionImages.value.map(i => i.path))
    console.log('answer_image_paths:', questionAnswerImages.value.map(i => i.path))
    console.log('knowledge_points:', questionKnowledgePoints.value)
    loading.value = true
    await createQuestion({
      name: questionName.value,
      subject: questionSubject.value,
      knowledge_points: questionKnowledgePoints.value,
      question_image_paths: questionQuestionImages.value.map(i => i.path),
      answer_image_paths: questionAnswerImages.value.map(i => i.path),
    })

    // 清空表单
    questionName.value = ''
    questionSubject.value = ''
    questionKnowledgePoints.value = []
    newKnowledgePoint.value = ''
    questionQuestionImages.value = []
    questionAnswerImages.value = []

    alert('题目创建成功！')
    router.push({ path: '/questions', query: { r: String(Date.now()) } })
  } catch (e) {
    console.error(e)
    alert('创建题目失败：' + String(e))
  }

  finally {
    loading.value = false
    console.log('handleSubmit end')
  }

}

const handleCancel = () => {
  if (questionName.value || questionSubject.value || questionKnowledgePoints.value.length > 0 ||
      questionQuestionImages.value.length > 0 || questionAnswerImages.value.length > 0) {
    if (!confirm('确定要放弃当前编辑吗？')) {
      return
    }
  }
  router.push('/questions')
}

// 使用 Tauri dialog 选择多个图片（逐个选择，可以多次添加）
const selectQuestionImages = async () => {
  try {
    const res = await open({
      multiple: true,
      filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp'] }]
    })
    if (res) {
      // Tauri dialog 返回字符串数组或单个字符串
      const paths = Array.isArray(res) ? res : [res]
      console.log('[DEBUG] Selected question images:', paths)
      // 加载图片预览
      const newImages = await loadImagePreview(paths.map(p => p as string))
      // 追加到现有图片列表
      const startIndex = questionQuestionImages.value.length
      newImages.forEach((img, i) => {
        img.sortOrder = startIndex + i
      })
      questionQuestionImages.value = [...questionQuestionImages.value, ...newImages]
    }
  } catch (e) {
    console.error('Failed to select images:', e)
    alert('选择图片失败: ' + e)
  }
}

const selectAnswerImages = async () => {
  try {
    const res = await open({
      multiple: true,
      filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp'] }]
    })
    if (res) {
      const paths = Array.isArray(res) ? res : [res]
      console.log('[DEBUG] Selected answer images:', paths)
      // 加载图片预览
      const newImages = await loadImagePreview(paths.map(p => p as string))
      // 追加到现有图片列表
      const startIndex = questionAnswerImages.value.length
      newImages.forEach((img, i) => {
        img.sortOrder = startIndex + i
      })
      questionAnswerImages.value = [...questionAnswerImages.value, ...newImages]
    }
  } catch (e) {
    console.error('Failed to select images:', e)
    alert('选择图片失败: ' + e)
  }
}

const removeQuestionImage = (index: number) => {
  questionQuestionImages.value.splice(index, 1)
  // 更新排序
  questionQuestionImages.value.forEach((item, i) => {
    item.sortOrder = i
  })
}

const removeAnswerImage = (index: number) => {
  questionAnswerImages.value.splice(index, 1)
  // 更新排序
  questionAnswerImages.value.forEach((item, i) => {
    item.sortOrder = i
  })
}

// 向上移动图片
const moveQuestionImageUp = (index: number) => {
  if (index <= 0) return
  const temp = questionQuestionImages.value[index]
  questionQuestionImages.value[index] = questionQuestionImages.value[index - 1]
  questionQuestionImages.value[index - 1] = temp
  // 更新排序
  questionQuestionImages.value.forEach((item, i) => {
    item.sortOrder = i
  })
}

const moveQuestionImageDown = (index: number) => {
  if (index >= questionQuestionImages.value.length - 1) return
  const temp = questionQuestionImages.value[index]
  questionQuestionImages.value[index] = questionQuestionImages.value[index + 1]
  questionQuestionImages.value[index + 1] = temp
  // 更新排序
  questionQuestionImages.value.forEach((item, i) => {
    item.sortOrder = i
  })
}

const moveAnswerImageUp = (index: number) => {
  if (index <= 0) return
  const temp = questionAnswerImages.value[index]
  questionAnswerImages.value[index] = questionAnswerImages.value[index - 1]
  questionAnswerImages.value[index - 1] = temp
  // 更新排序
  questionAnswerImages.value.forEach((item, i) => {
    item.sortOrder = i
  })
}

const moveAnswerImageDown = (index: number) => {
  if (index >= questionAnswerImages.value.length - 1) return
  const temp = questionAnswerImages.value[index]
  questionAnswerImages.value[index] = questionAnswerImages.value[index + 1]
  questionAnswerImages.value[index + 1] = temp
  // 更新排序
  questionAnswerImages.value.forEach((item, i) => {
    item.sortOrder = i
  })
}
</script>

<template>
  <div class="new-container">
    <div class="back-section">
      <button class="back-link" @click="handleCancel">
        ← 返回列表
      </button>
    </div>

    <div class="new-card">
      <h1 class="page-title">新建题目</h1>

      <form @submit.prevent="handleSubmit" class="question-form">
        <!-- 题目名 -->
        <div class="form-group">
          <label class="form-label" for="name">题目名 *</label>
          <input
            id="name"
            v-model="questionName"
            type="text"
            class="form-input"
            placeholder="请输入题目名..."
            required
          />
        </div>

        <!-- 科目 -->
        <div class="form-group">
          <label class="form-label" for="subject">科目 *</label>
          <input
            id="subject"
            v-model="questionSubject"
            type="text"
            class="form-input"
            placeholder="请输入科目..."
            required
          />
        </div>

        <!-- 知识点 -->
        <div class="form-group">
          <label class="form-label">知识点 *（可多次添加）</label>
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
          <div v-if="questionKnowledgePoints.length > 0" class="knowledge-points-list">
            <div v-for="(kp, index) in questionKnowledgePoints" :key="index" class="knowledge-point-item">
              <span class="kp-text">{{ kp }}</span>
              <button type="button" class="remove-kp-btn" @click="removeKnowledgePoint(index)">×</button>
            </div>
          </div>
          <div v-else class="kp-empty-hint">暂无知识点，请添加</div>
        </div>

        <!-- 题目图 -->
        <div class="form-group">
          <label class="form-label">题目图 * (可多选)</label>
          <div class="image-upload-area">
            <div v-if="questionQuestionImages.length === 0" class="upload-placeholder" @click="selectQuestionImages">
              <span class="upload-icon">📷</span>
              <span class="upload-text">点击选择题目图片</span>
            </div>
            <div v-else class="upload-preview-list">
              <div
                v-for="(img, index) in questionQuestionImages"
                :key="'q'+index"
                class="upload-preview-item"
                draggable="true"
                @dragstart="onDragStart(img, 'question')"
                @dragover="onDragOver"
                @drop="onDrop(img, 'question')"
                @dragend="onDragEnd"
              >
                <div class="drag-handle">
                  <span class="sort-number">{{ index + 1 }}</span>
                </div>
                <img :src="img.base64" :alt="'题目图片 ' + (index + 1)" class="preview-thumbnail" @click="showPreview(img.base64)" />
                <div class="preview-actions">
                  <button type="button" class="move-btn" @click.stop="moveQuestionImageUp(index)" :disabled="index === 0" title="上移">↑</button>
                  <button type="button" class="move-btn" @click.stop="moveQuestionImageDown(index)" :disabled="index === questionQuestionImages.length - 1" title="下移">↓</button>
                  <button type="button" class="remove-btn" @click.stop="removeQuestionImage(index)">删除</button>
                </div>
              </div>
              <button type="button" class="add-more-btn" @click.stop="selectQuestionImages">+ 添加更多</button>
            </div>
          </div>
        </div>

        <!-- 答案图 -->
        <div class="form-group">
          <label class="form-label">答案图 * (可多选)</label>
          <div class="image-upload-area">
            <div v-if="questionAnswerImages.length === 0" class="upload-placeholder" @click="selectAnswerImages">
              <span class="upload-icon">📷</span>
              <span class="upload-text">点击选择答案图片</span>
            </div>
            <div v-else class="upload-preview-list">
              <div
                v-for="(img, index) in questionAnswerImages"
                :key="'a'+index"
                class="upload-preview-item"
                draggable="true"
                @dragstart="onDragStart(img, 'answer')"
                @dragover="onDragOver"
                @drop="onDrop(img, 'answer')"
                @dragend="onDragEnd"
              >
                <div class="drag-handle">
                  <span class="sort-number">{{ index + 1 }}</span>
                </div>
                <img :src="img.base64" :alt="'答案图片 ' + (index + 1)" class="preview-thumbnail" @click="showPreview(img.base64)" />
                <div class="preview-actions">
                  <button type="button" class="move-btn" @click.stop="moveAnswerImageUp(index)" :disabled="index === 0" title="上移">↑</button>
                  <button type="button" class="move-btn" @click.stop="moveAnswerImageDown(index)" :disabled="index === questionAnswerImages.length - 1" title="下移">↓</button>
                  <button type="button" class="remove-btn" @click.stop="removeAnswerImage(index)">删除</button>
                </div>
              </div>
              <button type="button" class="add-more-btn" @click.stop="selectAnswerImages">+ 添加更多</button>
            </div>
          </div>
        </div>

        <!-- 操作按钮 -->
        <div class="form-actions">
          <button type="button" class="btn cancel" @click="handleCancel">
            取消
          </button>
          <button type="submit" class="btn submit">
            创建题目
          </button>
        </div>
      </form>
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
.new-container {
  width: 100%;
  max-width: 900px;
  margin: 0;
}

.back-section {
  margin-bottom: 16px;
}

.back-link {
  background: none;
  border: none;
  color: #4CAF50;
  font-size: 14px;
  cursor: pointer;
  padding: 8px 0;
  transition: color 0.2s;
}

.back-link:hover {
  color: #45a049;
}

.new-card {
  background-color: #ffffff;
  border-radius: 12px;
  padding: 32px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
}

.page-title {
  font-size: 28px;
  color: #333;
  margin-bottom: 32px;
  text-align: center;
}

.question-form {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-label {
  color: #333;
  font-size: 14px;
  font-weight: 500;
}

.form-input,
.form-select {
  background-color: #fff;
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 12px 16px;
  color: #333;
  font-size: 15px;
  font-family: inherit;
}

.form-input:focus,
.form-select:focus {
  outline: none;
  border-color: #4CAF50;
}

.form-select {
  cursor: pointer;
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
  cursor: grab;
  transition: all 0.2s;
}

.upload-preview-item:hover {
  border-color: #45a049;
  box-shadow: 0 2px 8px rgba(76, 175, 80, 0.2);
}

.upload-preview-item:active {
  cursor: grabbing;
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

.form-actions {
  display: flex;
  gap: 12px;
  margin-top: 8px;
}

.btn {
  flex: 1;
  padding: 14px;
  border: none;
  border-radius: 8px;
  font-size: 15px;
  font-weight: 600;
  transition: all 0.2s;
  cursor: pointer;
}

.btn.cancel {
  background-color: #e0e0e0;
  color: #333;
}

.btn.cancel:hover {
  background-color: #d0d0d0;
}

.btn.submit {
  background-color: #4CAF50;
  color: #ffffff;
}

.btn.submit:hover {
  background-color: #45a049;
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
