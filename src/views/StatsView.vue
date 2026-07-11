<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { getStats, getSubjectErrorStats, getReviewDailySeries, type StatsData, type SubjectStat, type DailySeriesPoint } from '@/api/review'
import { useSettingsStore } from '@/stores/settings'
import LineChart from '@/components/LineChart.vue'

const store = useSettingsStore()

const stats = ref<StatsData | null>(null)
const subjectStats = ref<SubjectStat[]>([])
const loading = ref(true)
const loadingSubjectStats = ref(true)
const loadingMonthly = ref(false)

// 折线图颜色（按 settings.activeSubjects 顺序循环）
const SUBJECT_COLORS: string[] = [
  '#2196F3',
  '#4CAF50',
  '#FF9800',
  '#9C27B0',
  '#00BCD4',
  '#E91E63',
  '#795548',
  '#607D8B',
]
function pickColor(i: number): string {
  return SUBJECT_COLORS[i % SUBJECT_COLORS.length] ?? '#666'
}

// === 当前月份（calendar year & 1-indexed month）===
const today = new Date()
const currentYear = ref(today.getFullYear())
const currentMonth = ref(today.getMonth() + 1) // 1..=12

// === 每天桶 ↔ 日历日 ===
// 图表的"日历日"按 0:00 切日（cutoff = 0），与用户设置中的 day_cutoff_hour 解耦；
// 用户设置仅影响"今日推荐"等业务口径，不影响折线图横轴的 1-末日 排列。
const chartOffsetSec = computed(() => store.timezoneOffsetHours * 3600)

function chartUnixToDayBucket(unix: number): number {
  return Math.floor((unix + chartOffsetSec.value) / 86400)
}

// 本月 1 号本地 00:00 与下月 1 号本地 00:00 对应的 unix sec
function monthRangeUnix(year: number, month1: number): { startUnix: number; endUnix: number } {
  const offsetMs = chartOffsetSec.value * 1000
  const startUtcMs = Date.UTC(year, month1 - 1, 1) - offsetMs
  const endUtcMs = Date.UTC(year, month1, 1) - offsetMs
  return {
    startUnix: Math.floor(startUtcMs / 1000),
    endUnix: Math.floor(endUtcMs / 1000),
  }
}

const lastDayOfMonth = computed(() => {
  // new Date(year, month, 0) 是"上个月最后一天"，正好是当月最后一天
  return new Date(currentYear.value, currentMonth.value, 0).getDate()
})

const monthDayBucketRange = computed(() => {
  const { startUnix, endUnix } = monthRangeUnix(currentYear.value, currentMonth.value)
  return {
    startBucket: chartUnixToDayBucket(startUnix),
    endBucket: Math.max(chartUnixToDayBucket(startUnix), chartUnixToDayBucket(endUnix - 1)),
  }
})

const monthXLabels = computed(() => {
  // 横轴固定为该月 1 号到末日，不受上月末/下月初影响
  const out: string[] = []
  for (let d = 1; d <= lastDayOfMonth.value; d++) {
    out.push(`${currentMonth.value}-${d}`)
  }
  return out
})

const monthlyDailySeries = ref<DailySeriesPoint[]>([])

async function loadMonthly() {
  loadingMonthly.value = true
  try {
    const { startBucket, endBucket } = monthDayBucketRange.value
    const data = await getReviewDailySeries({
      timezoneOffsetHours: store.timezoneOffsetHours,
      dayCutoffHour: 0, // 图表按 0:00 切日历日，与用户 cutoff 设置解耦
      startDayBucket: startBucket,
      endDayBucket: endBucket,
      subjectFilter: null,
    })
    monthlyDailySeries.value = data
  } catch (e) {
    console.error('加载月度复习序列失败', e)
  } finally {
    loadingMonthly.value = false
  }
}

// === 复习题数（折线图 1）===

const activeSubjects = computed(() => {
  const archived = new Set<string>()
  for (const [name, cfg] of Object.entries(store.subjectConfigs)) {
    if (cfg.archived) archived.add(name)
  }
  const seen = new Set<string>()
  const out: string[] = []
  for (const row of subjectStats.value) {
    if (row.subject === '__未分类__') continue
    if (archived.has(row.subject)) continue
    if (seen.has(row.subject)) continue
    seen.add(row.subject)
    out.push(row.subject)
  }
  return out
})

const reviewCountSeries = computed(() => {
  const colors = new Map<string, string>()
  activeSubjects.value.forEach((s, i) => {
    colors.set(s, pickColor(i))
  })
  const { startBucket, endBucket } = monthDayBucketRange.value
  const bySubjectDay = new Map<string, Map<number, number>>()
  for (const p of monthlyDailySeries.value) {
    if (!colors.has(p.subject)) continue
    let inner = bySubjectDay.get(p.subject)
    if (!inner) {
      inner = new Map()
      bySubjectDay.set(p.subject, inner)
    }
    inner.set(p.day_bucket, p.review_count)
  }
  return activeSubjects.value.map((subject) => {
    // 只在"实际有复习"的日子产出点：x 序号 = day - startBucket；没数据时跳过
    const points: { x: number; y: number }[] = []
    for (let b = startBucket; b <= endBucket; b++) {
      const v = bySubjectDay.get(subject)?.get(b) ?? 0
      if (v <= 0) continue
      points.push({ x: b - startBucket, y: v })
    }
    return { name: subject, color: pickColor(activeSubjects.value.indexOf(subject)), points }
  })
})

// === 准确率（折线图 2）===
const accuracySeries = computed(() => {
  const colors = new Map<string, string>()
  activeSubjects.value.forEach((s, i) => {
    colors.set(s, pickColor(i))
  })
  const { startBucket, endBucket } = monthDayBucketRange.value
  const bySubjectDay = new Map<string, Map<number, { correct: number; wrong: number }>>()
  for (const p of monthlyDailySeries.value) {
    if (!colors.has(p.subject)) continue
    let inner = bySubjectDay.get(p.subject)
    if (!inner) {
      inner = new Map()
      bySubjectDay.set(p.subject, inner)
    }
    inner.set(p.day_bucket, { correct: p.correct_count, wrong: p.wrong_count })
  }
  return activeSubjects.value.map((subject) => {
    // 只在"实际有 correct+wrong"的日子产出点；模糊单独不计入且为 0 时跳过
    const points: { x: number; y: number }[] = []
    for (let b = startBucket; b <= endBucket; b++) {
      const cell = bySubjectDay.get(subject)?.get(b)
      if (!cell) continue
      const denom = cell.correct + cell.wrong
      if (denom <= 0) continue
      points.push({ x: b - startBucket, y: cell.correct / denom })
    }
    return { name: subject, color: pickColor(activeSubjects.value.indexOf(subject)), points }
  })
})

// === 分科错误率表（活动学科）===
const activeSubjectStats = computed(() => {
  const archived = new Set<string>()
  for (const [name, cfg] of Object.entries(store.subjectConfigs)) {
    if (cfg.archived) archived.add(name)
  }
  return subjectStats.value
    .filter((r) => r.subject !== '__未分类__' && !archived.has(r.subject))
    .sort((a, b) => b.review_count - a.review_count)
})

function accuracyOf(stat: SubjectStat): number {
  const denom = stat.correct_count + stat.wrong_count
  if (denom <= 0) return 0
  return stat.correct_count / denom
}
function accuracyPct(stat: SubjectStat): string {
  return (accuracyOf(stat) * 100).toFixed(0) + '%'
}

// === 题目状态分布（保留）===
const stateStats = computed(() => {
  if (!stats.value) return { NEW: 0, LEARNING: 0, STABLE: 0 }
  return {
    NEW: stats.value.state_counts.new_count,
    LEARNING: stats.value.state_counts.learning_count,
    STABLE: stats.value.state_counts.stable_count,
  }
})

function getProgressWidth(count: number) {
  if (!stats.value || stats.value.total_questions === 0) return '0'
  return (count / stats.value.total_questions * 100).toFixed(1)
}

function prevMonth() {
  let m = currentMonth.value - 1
  let y = currentYear.value
  if (m < 1) {
    m = 12
    y -= 1
  }
  currentMonth.value = m
  currentYear.value = y
}

function nextMonth() {
  let m = currentMonth.value + 1
  let y = currentYear.value
  if (m > 12) {
    m = 1
    y += 1
  }
  currentMonth.value = m
  currentYear.value = y
}

function jumpToCurrentMonth() {
  currentYear.value = today.getFullYear()
  currentMonth.value = today.getMonth() + 1
}

onMounted(async () => {
  if (!store.loaded) {
    await store.loadSettings()
  }
  try {
    stats.value = await getStats()
  } catch (e) {
    console.error('加载统计失败:', e)
  } finally {
    loading.value = false
  }
  try {
    subjectStats.value = await getSubjectErrorStats()
  } catch (e) {
    console.error('加载分科错误率失败', e)
  } finally {
    loadingSubjectStats.value = false
  }
  await loadMonthly()
})

// 图表不依赖 day_cutoff_hour（横轴固定为 1-末日）；仅在月份或时区改变时重拉
watch(
  () => [currentYear.value, currentMonth.value, store.timezoneOffsetHours],
  () => {
    loadMonthly()
  }
)
</script>

<template>
  <div class="stats-container">
    <h1 class="page-title">统计</h1>

    <div v-if="loading" class="loading">加载中...</div>

    <template v-else-if="stats">
    <!-- 概览卡片 -->
    <div class="overview-cards">
      <div class="stat-card">
        <div class="stat-icon icon-blue">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <rect x="3" y="4" width="18" height="5" rx="0.8" />
            <rect x="3" y="11" width="18" height="5" rx="0.8" />
            <rect x="3" y="18" width="18" height="5" rx="0.8" />
            <line x1="6" y1="6.5" x2="14" y2="6.5" />
            <line x1="6" y1="13.5" x2="14" y2="13.5" />
            <line x1="6" y1="20.5" x2="14" y2="20.5" />
          </svg>
        </div>
        <div class="stat-info">
          <div class="stat-value">{{ stats.total_questions }}</div>
          <div class="stat-label">总题目数</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon icon-green">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <circle cx="12" cy="12" r="9" />
            <circle cx="12" cy="12" r="5.5" />
            <circle cx="12" cy="12" r="2" />
          </svg>
        </div>
        <div class="stat-info">
          <div class="stat-value">{{ stats.today_reviewed }}</div>
          <div class="stat-label">今日已复习</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon icon-orange">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M20 12a8 8 0 0 1 -14 5.3" />
            <polyline points="6 21 6 16 11 16" />
            <path d="M4 12a8 8 0 0 1 14 -5.3" />
            <polyline points="18 3 18 8 13 8" />
          </svg>
        </div>
        <div class="stat-info">
          <div class="stat-value">{{ stats.total_reviews }}</div>
          <div class="stat-label">总复习次数</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon icon-purple">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <line x1="5" y1="20" x2="20" y2="20" />
            <rect x="6" y="14" width="3.5" height="6" />
            <rect x="11" y="9" width="3.5" height="11" />
            <rect x="16" y="4" width="3.5" height="16" />
          </svg>
        </div>
        <div class="stat-info">
          <div class="stat-value">{{ stats.average_accuracy.toFixed(0) }}%</div>
          <div class="stat-label">平均准确率</div>
        </div>
      </div>
    </div>

    <!-- 题目状态分布 -->
    <div class="section-card">
      <h2 class="section-title">题目状态分布</h2>
      <div class="state-distribution">
        <div class="state-item">
          <div class="state-header">
            <span class="state-name new">新题</span>
            <span class="state-count">{{ stateStats.NEW }} 题</span>
          </div>
          <div class="progress-bar">
            <div
              class="progress-fill new"
              :style="{ width: getProgressWidth(stateStats.NEW) + '%' }"
            ></div>
          </div>
          <div class="state-percent">{{ getProgressWidth(stateStats.NEW) }}%</div>
        </div>

        <div class="state-item">
          <div class="state-header">
            <span class="state-name learning">学习中</span>
            <span class="state-count">{{ stateStats.LEARNING }} 题</span>
          </div>
          <div class="progress-bar">
            <div
              class="progress-fill learning"
              :style="{ width: getProgressWidth(stateStats.LEARNING) + '%' }"
            ></div>
          </div>
          <div class="state-percent">{{ getProgressWidth(stateStats.LEARNING) }}%</div>
        </div>

        <div class="state-item">
          <div class="state-header">
            <span class="state-name stable">已掌握</span>
            <span class="state-count">{{ stateStats.STABLE }} 题</span>
          </div>
          <div class="progress-bar">
            <div
              class="progress-fill stable"
              :style="{ width: getProgressWidth(stateStats.STABLE) + '%' }"
            ></div>
          </div>
          <div class="state-percent">{{ getProgressWidth(stateStats.STABLE) }}%</div>
        </div>
      </div>
    </div>

    <!-- 分科复习状况 -->
    <div class="section-card">
      <h2 class="section-title">分科复习状况</h2>
      <div v-if="loadingSubjectStats" class="loading-inline">加载中...</div>
      <table v-else class="subject-stats-table">
        <thead>
          <tr>
            <th>学科</th>
            <th class="num">总复习</th>
            <th class="num">记得</th>
            <th class="num">不记得</th>
            <th class="num">模糊</th>
            <th class="num">准确率</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in activeSubjectStats" :key="row.subject">
            <td>{{ row.subject }}</td>
            <td class="num">{{ row.review_count }}</td>
            <td class="num">{{ row.correct_count }}</td>
            <td class="num">{{ row.wrong_count }}</td>
            <td class="num">{{ row.fuzzy_count }}</td>
            <td class="num">{{ accuracyPct(row) }}</td>
          </tr>
          <tr v-if="activeSubjectStats.length === 0">
            <td colspan="6" class="empty-cell">暂无数据</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 复习行为统计：折线图 -->
    <div class="section-card">
      <h2 class="section-title">复习行为统计</h2>

      <div class="month-switcher">
        <button class="month-btn" @click="prevMonth">←</button>
        <div class="month-label">
          <span class="month-y">{{ currentYear }} 年</span>
          <span class="month-m">{{ currentMonth }} 月</span>
        </div>
        <button class="month-btn" @click="nextMonth">→</button>
        <button class="month-jump" @click="jumpToCurrentMonth">回到本月</button>
        <span class="month-active-info">
          共 <strong>{{ activeSubjects.length }}</strong> 个活动学科
        </span>
      </div>

      <h3 class="chart-title">各科复习题数 / 日</h3>
      <div v-if="loadingMonthly" class="loading-inline">加载中...</div>
      <LineChart
        v-else
        :series="reviewCountSeries"
        :x-labels="monthXLabels"
        :height="220"
      />

      <h3 class="chart-title">各科准确率 / 日</h3>
      <div v-if="loadingMonthly" class="loading-inline">加载中...</div>
      <LineChart
        v-else
        :series="accuracySeries"
        :x-labels="monthXLabels"
        :height="220"
        y-as-percent
      />
    </div>
    </template>
  </div>
</template>

<style scoped>
.stats-container {
  max-width: 800px;
  width: 100%;
  margin-left: 0;
}

.loading {
  text-align: center;
  color: #666;
  font-size: 16px;
  padding: 40px;
}

.loading-inline {
  color: #666;
  font-size: 13px;
  padding: 20px;
  text-align: center;
}

.page-title {
  font-size: 28px;
  color: #333;
  margin-bottom: 24px;
  text-align: center;
}

/* 概览卡片 */
.overview-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
}

.stat-card {
  background-color: #ffffff;
  border-radius: 12px;
  padding: 20px;
  display: flex;
  align-items: center;
  gap: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.stat-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.stat-icon svg {
  width: 100%;
  height: 100%;
}

.icon-blue { color: #2196F3; }
.icon-green { color: #4CAF50; }
.icon-orange { color: #FF9800; }
.icon-purple { color: #9C27B0; }

.stat-info { flex: 1; }

.stat-value {
  font-size: 28px;
  font-weight: bold;
  color: #333;
  line-height: 1.2;
}

.stat-label {
  color: #666;
  font-size: 13px;
  margin-top: 4px;
}

/* 区块卡片 */
.section-card {
  background-color: #ffffff;
  border-radius: 12px;
  padding: 24px;
  margin-bottom: 24px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.section-title {
  font-size: 18px;
  color: #333;
  margin-bottom: 12px;
  font-weight: 600;
}

/* 状态分布 */
.state-distribution {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.state-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.state-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.state-name {
  font-size: 14px;
  font-weight: 500;
}

.state-name.new     { color: #2196F3; }
.state-name.learning { color: #FF9800; }
.state-name.stable  { color: #4CAF50; }

.state-count { color: #666; font-size: 14px; }

.progress-bar {
  height: 8px;
  background-color: #f0f0f0;
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.3s ease;
}

.progress-fill.new     { background-color: #2196F3; }
.progress-fill.learning { background-color: #FF9800; }
.progress-fill.stable  { background-color: #4CAF50; }

.state-percent { text-align: right; color: #666; font-size: 13px; }

/* 分科错误率表 */
.subject-stats-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.subject-stats-table th,
.subject-stats-table td {
  padding: 8px 10px;
  border-bottom: 1px solid #eee;
  text-align: left;
}

.subject-stats-table th {
  background-color: #fafafa;
  font-weight: 600;
  color: #555;
}

.subject-stats-table .num {
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.subject-stats-table .empty-cell {
  text-align: center;
  color: #999;
  padding: 16px;
}

/* 月份切换 */
.month-switcher {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.month-btn {
  width: 32px;
  height: 32px;
  border-radius: 6px;
  border: 1px solid #ddd;
  background-color: #fff;
  cursor: pointer;
  font-size: 14px;
  color: #333;
}

.month-btn:hover {
  background-color: #f5f5f5;
}

.month-label {
  display: inline-flex;
  align-items: baseline;
  gap: 4px;
  font-weight: 600;
  font-size: 16px;
  color: #333;
  min-width: 110px;
  justify-content: center;
}

.month-y { font-size: 13px; color: #666; }
.month-m { font-size: 18px; }

.month-jump {
  padding: 6px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  background-color: #fff;
  cursor: pointer;
  font-size: 12px;
  color: #555;
}

.month-jump:hover { background-color: #f5f5f5; }

.month-active-info {
  color: #888;
  font-size: 12px;
  margin-left: auto;
}

/* 折线图 */
.chart-title {
  font-size: 14px;
  color: #444;
  margin: 18px 0 10px;
  font-weight: 600;
}
</style>
