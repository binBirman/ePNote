<script setup lang="ts">
import { computed } from 'vue'

interface SeriesPoint {
  x: number
  y: number
}

interface Series {
  name: string
  color: string
  points: SeriesPoint[]
}

interface Props {
  series: Series[]
  xLabels?: string[]
  height?: number
  /** 0~1 区间的 y 数值显示为百分比，否则保留原值 */
  yAsPercent?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  xLabels: () => [],
  height: 200,
  yAsPercent: false,
})

const W = 800
const PADDING_L = 48
const PADDING_R = 16
const PADDING_T = 12
const PADDING_B = 32
const PLOT_W = W - PADDING_L - PADDING_R
const PLOT_H = computed(() => props.height - PADDING_T - PADDING_B)

interface Bounds {
  xMin: number
  xMax: number
  yMin: number
  yMax: number
}

const bounds = computed<Bounds>(() => {
  let xMin = Infinity
  let xMax = -Infinity
  let yMin = Infinity
  let yMax = -Infinity
  let anyPoint = false
  for (const s of props.series) {
    for (const p of s.points) {
      anyPoint = true
      if (p.x < xMin) xMin = p.x
      if (p.x > xMax) xMax = p.x
      if (p.y < yMin) yMin = p.y
      if (p.y > yMax) yMax = p.y
    }
  }
  if (!anyPoint) {
    xMin = 0
    xMax = 1
    yMin = 0
    yMax = 1
  } else {
    if (xMin === xMax) {
      xMin -= 1
      xMax += 1
    }
    if (yMin === yMax) {
      yMin -= 1
      yMax += 1
    } else {
      const pad = (yMax - yMin) * 0.1
      yMax += pad
      yMin = Math.max(0, yMin - pad)
    }
  }
  return { xMin, xMax, yMin, yMax }
})

function scaleX(x: number, b: Bounds): number {
  if (b.xMax === b.xMin) return PADDING_L
  return PADDING_L + ((x - b.xMin) / (b.xMax - b.xMin)) * PLOT_W
}

function scaleY(y: number, b: Bounds): number {
  const h = PLOT_H.value
  if (b.yMax === b.yMin) return PADDING_T + h / 2
  return PADDING_T + (1 - (y - b.yMin) / (b.yMax - b.yMin)) * h
}

interface RenderedSeries {
  name: string
  color: string
  d: string
  circles: { cx: number; cy: number }[]
}

const rendered = computed<RenderedSeries[]>(() => {
  const b = bounds.value
  return props.series.map((s) => {
    let d = ''
    const circles: { cx: number; cy: number }[] = []
    for (let i = 0; i < s.points.length; i++) {
      const p = s.points[i]
      if (!p) continue
      const x = scaleX(p.x, b)
      const y = scaleY(p.y, b)
      d += (i === 0 ? 'M' : 'L') + x.toFixed(1) + ',' + y.toFixed(1) + ' '
      circles.push({ cx: x, cy: y })
    }
    return { name: s.name, color: s.color, d: d.trim(), circles }
  })
})

const yTicks = computed<{ y: number; label: string }[]>(() => {
  const b = bounds.value
  const ticks = 4
  const out: { y: number; label: string }[] = []
  for (let i = 0; i <= ticks; i++) {
    const yVal = b.yMin + ((b.yMax - b.yMin) * i) / ticks
    let label: string
    if (props.yAsPercent) {
      label = `${Math.round(yVal * 100)}%`
    } else if (Number.isInteger(yVal)) {
      label = String(Math.round(yVal))
    } else {
      label = yVal.toFixed(1)
    }
    out.push({ y: scaleY(yVal, b), label })
  }
  return out
})

const xTickStep = computed(() => {
  const b = bounds.value
  if (b.xMax === b.xMin) return 1
  const range = b.xMax - b.xMin + 1
  if (props.xLabels.length > 0) return Math.max(1, Math.ceil(range / 8))
  return Math.max(1, Math.ceil(range / 6))
})

const xTicks = computed<{ x: number; label: string }[]>(() => {
  const b = bounds.value
  if (props.xLabels.length === 0) {
    const out: { x: number; label: string }[] = []
    const range = b.xMax - b.xMin + 1
    const step = Math.max(1, Math.ceil(range / 6))
    for (let v = b.xMin; v <= b.xMax; v += step) {
      out.push({ x: scaleX(v, b), label: String(v) })
    }
    return out
  }
  const out: { x: number; label: string }[] = []
  const len = props.xLabels.length
  const step = xTickStep.value
  for (let i = 0; i < len; i++) {
    if (i % step !== 0 && i !== len - 1) continue
    const t = b.xMin + i
    const label = props.xLabels[i] ?? ''
    out.push({ x: scaleX(t, b), label })
  }
  return out
})

function viewBoxY(y: number) {
  return y
}

function viewBoxX(x: number) {
  return x
}
</script>

<template>
  <div class="line-chart" :style="{ height: (height + 4) + 'px' }">
    <svg
      class="line-chart-svg"
      :viewBox="`0 0 ${W} ${height}`"
      preserveAspectRatio="none"
      width="100%"
      :height="height"
    >
      <!-- y 轴 grid & ticks -->
      <g v-for="(tick, idx) in yTicks" :key="`yt-${idx}`" class="y-tick">
        <line
          :x1="PADDING_L"
          :x2="W - PADDING_R"
          :y1="viewBoxY(tick.y)"
          :y2="viewBoxY(tick.y)"
          stroke="#eee"
          stroke-dasharray="3 3"
        />
        <text
          :x="PADDING_L - 6"
          :y="viewBoxY(tick.y) + 4"
          text-anchor="end"
          font-size="11"
          fill="#888"
        >
          {{ tick.label }}
        </text>
      </g>

      <!-- x 轴 ticks -->
      <g v-for="(tick, idx) in xTicks" :key="`xt-${idx}`" class="x-tick">
        <text
          :x="viewBoxX(tick.x)"
          :y="height - 6"
          text-anchor="middle"
          font-size="11"
          fill="#888"
        >
          {{ tick.label }}
        </text>
      </g>

      <!-- 各 series -->
      <g
        v-for="(s, idx) in rendered"
        :key="`series-${idx}`"
        :stroke="s.color"
        fill="none"
      >
        <path
          v-if="s.d"
          :d="s.d"
          stroke-width="2"
          stroke-linejoin="round"
          stroke-linecap="round"
        />
        <circle
          v-for="(c, cIdx) in s.circles"
          :key="`p-${idx}-${cIdx}`"
          :cx="c.cx"
          :cy="c.cy"
          r="3"
          :fill="s.color"
          stroke="#fff"
          stroke-width="1"
        />
      </g>

      <!-- 空状态 -->
      <text
        v-if="series.length === 0 || series.every(s => s.points.length === 0)"
        :x="W / 2"
        :y="height / 2"
        text-anchor="middle"
        font-size="13"
        fill="#999"
      >
        暂无数据
      </text>
    </svg>

    <!-- 图例 -->
    <div v-if="series.length > 0" class="legend">
      <div v-for="s in series" :key="`leg-${s.name}`" class="legend-item">
        <span class="dot" :style="{ backgroundColor: s.color }" />
        <span>{{ s.name }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.line-chart {
  width: 100%;
  position: relative;
  display: flex;
  flex-direction: column;
}

.line-chart-svg {
  display: block;
  background-color: #fff;
  border-radius: 6px;
}

.legend {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  margin-top: 8px;
  font-size: 12px;
  color: #444;
}

.legend-item {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  display: inline-block;
}
</style>
