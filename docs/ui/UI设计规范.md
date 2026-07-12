# UI 设计规范

> 截至 v1.0.0+。本规范是前端视图与组件的视觉与交互约定。

## 一、设计原则

### 1. 简洁干净

- 大面积留白，宁少勿多
- 单一焦点：每屏只突出一个主操作
- 颜色克制：主色 + 中性灰为主，**强调色仅在状态/警示用**
- 字体层级清晰：标题 / 副标题 / 正文 / 辅助文 四级
- 元素无装饰边框：圆角小、阴影浅、按需

### 2. **不用表情包（emoji）**

- 不在标签、按钮、占位符、列表项中用 `🗑️` / `📚` / `📊` / `🎯` 等 emoji，但允许在文本中适当使用
- 需要图标时**必须**用**纯色简笔 SVG**（参考 `StatsView` 的 4 个概览卡：书 / 靶心 / 循环箭头 / 柱状图）
- 装饰性字符（✓ / ✗ / → / •）OK，**emoji 不可**
- 已有违规示例 → 立即替换为 SVG 或文字

### 3. 状态而非装饰

- 用颜色表达**语义**：绿=正确/成功/进行中、橙=警告/已复习、红=错误/暂停、蓝=信息、灰=未激活
- 状态有"原因"才有意义（hover/tooltip 提示）

## 二、调色板

```
主色  主操作 / 通过   #4CAF50 (绿)
      信息 / 链接       #2196F3 (蓝)
      警告 / 已复习     #FF9800 (橙)
      错误 / 暂停       #f44336 (红)

中性  背景             #f5f5f5 (Sidebar、页面背景)
      卡片             #ffffff
      边框             #e0e0e0 / #ddd
      文本主色          #333
      文本次色          #666
      文本弱色          #888 / #999
      灰块             #fafafa / #f0f0f0

状态  NEW              #2196F3 (蓝)
      LEARNING         #FF9800 (橙)
      STABLE           #4CAF50 (绿)
      SUSPENDED        #9E9E9E (灰)
```

**注意**：

- **禁止**临时调色（业务调色板外）
- 错误率 / 准确率反馈用**主色 + 透明度**，不用冷色（蓝/绿）覆盖
- 暗色模式**未启用**——保持浅色极简，v1 范围不引入主题切换

## 三、字体

```
字体栈  -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto,
        Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue',
        sans-serif

不引入中文字体（系统字体已含中文）；不引入图标字体（统一 SVG）。
```

| 层级 | 字号 | 粗细 | 用途 |
|---|---|---|---|
| 标题 (h1) | 28 px | 700 | 页面标题 `page-title` |
| 副标题 (h2) | 18 px | 600 | 区块标题 `section-title` / `card-title` |
| 正文 | 14 px | 400 | 列表项 / 表格 |
| 强调 | 14 px | 600 | 状态 badge、按钮文案 |
| 辅助 | 12-13 px | 400 | 提示、描述、统计 label |

## 四、间距

```
4 / 8 / 12 / 16 / 20 / 24 / 32 px  (4 的倍数)

小元素内距  8 / 12 px
卡片内距    20 / 24 px
区块间距    16 / 24 px
页面外边距  30 / 40 px (main-content)
```

组件圆角：4 (小) / 6 (按钮) / 8 (输入) / 10 / 12 (卡片)

阴影：浅 `0 1px 4px rgba(0,0,0,0.05)` / 中 `0 2px 8px rgba(0,0,0,0.06)` / 仅卡片悬停态升一档

## 五、布局

### 5.1 主布局

```
<Sidebar />        固定 200px
<main>              margin-left: 200px
  <RouterView />   padding: 30 40
</main>
```

Sidebar 由 `App.vue` 渲染，根据 `route.path` 决定 active 态。`developerMode` 时额外显示"开发者中心"。

### 5.2 单视图布局

每个视图用 `<div class="X-container">` 包装，内部按"卡片堆叠"或"上下分区"组织：

```
[ 页面标题 ]

[ 区块 1 ]        // .settings-card / .section-card
[ 区块 2 ]
[ 区块 3 ]
```

区块内用 `setting-row` 网格（描述 + 控件）排版。

### 5.3 列表 / 表格

- 列表项：`<div class="item">` + hover 高亮 + 圆角 8
- 表格：`<table>` + 表头浅灰底 + 数字列右对齐 `text-align: right` + `font-variant-numeric: tabular-nums`

## 六、组件样式规范

### 6.1 按钮

- 主操作（CTA）：`background: #4CAF50; color: #fff; border: none; border-radius: 6; padding: 8-12 16-20;`
- 次操作：白底 + 灰边 + 灰文，hover 变绿边
- 危险：红 `#f44336`
- 禁用：`opacity: 0.6; cursor: not-allowed;`
- 文字按钮（无边框）：主色 hover 下划线

### 6.2 输入

- 数字输入（题数等）：`+ N - +N -` 按钮 + 数字框
- 范围输入（比例等）：`<input type="range">` + 实时显示数值
- 复选：原生 `<input type="checkbox">`，配 label
- **禁止**用 emoji 装饰

### 6.3 标签 (badge)

- 状态 badge 圆角 12、padding 4 10、字号 12、字粗 600
- 颜色按状态语义（见 §二 调色板）
- 文案简短："新题" / "学习中" / "已掌握" / "已暂停"（不用 emoji）

### 6.4 推荐理由标签 (StatsView / DevPreview)

- 圆角 12、内边距 4 10、字号 12
- 推荐理由 (绿色家族)：`background: #e8f5e9; color: #2E7D32`
- 错误率 (红色家族)：`background: #fdecea; color: #c62828`
- 落选 (灰色家族)：`background: #f5f5f5; color: #666`
- 文字为主，**无图标**

### 6.5 折线图 (LineChart)

- viewBox 800 × (height)，padding left 48 / right 16 / top 12 / bottom 32
- 网格线 `#eee` + 虚线 `stroke-dasharray="3 3"`
- 折线 `stroke-width="2"`，端点 `circle r="3"` 加白色描边
- 多 series 时用 §二 调色板循环
- 图例在下：彩色点 + 名称
- **禁止**用 emoji 数据点

## 七、空状态与错误

- 空状态：居中图标（SVG / 文字）+ 提示 + 主操作按钮
- 错误：红色边框 + 浅红底 + 重试按钮
- 加载中：inline 居中文字 "加载中..."，**不用**旋转 spinner（保持极简）

## 八、交互

- hover 状态：背景 + 边框色变化 + 微 `box-shadow` 升一档，200ms 过渡
- 点击：背景短暂加深（active 态）
- 焦点：input 焦点时 `border-color: #4CAF50; outline: none`
- 路由跳转：`<RouterLink>` 默认行为，**不**做花哨过渡

## 九、图表与数据展示

- 折线图（StatsView）固定显示整个月（1-末日），即使无数据也展示 X 轴
- Y 轴：复习题数取整到合理刻度（5 的倍数），准确率固定 0%-100%
- 表格右对齐数字 + 浅灰交替行
- **禁止**用 emoji 装饰图表

## 十、无障碍

- 所有交互元素 `<button>` / `<a>` / `<input>` 保持原生语义
- 颜色不**单独**传递信息（同时配图标或文字）
- 表单 `<label for>` 关联 `<input id>`
- 键盘焦点环可见（`outline: 2px solid #4CAF50` 备用样式）
- 中文界面不依赖 emoji 表达语义

## 十一、违规清单（自检）

- ❌ `<button>🗑️ 删除</button>`
- ❌ `<span>📚 题目管理</span>`
- ❌ 表格 `<th>📊 数量</th>`
- ❌ placeholder 用 emoji 而非文字

**发现违规 → 立即替换为 SVG / 文字 / 标点符号。**

## 十二、参考实现

- `Sidebar.vue`：纯文字 + 路由高亮，**无图标**（保持极简，未来若需图标用 SVG）
- `StatsView.vue`：4 个概览卡用纯色简笔 SVG
- `QuestionsView.vue`：题目列表 + 高频错题 badge 纯文字
- `SettingsView.vue`：高级设置折叠 (`v-if="showAdvanced"`)
- `ReviewSessionView.vue`：卡片答题 + 推荐理由标签

## 关联文档

- [`项目结构.md`](./项目结构.md) — 前端目录布局
- [`前端结构.md`](./前端结构.md) — 路由表、Pinia store、view 职责
- [`business/业务规则文档.md`](../business/业务规则文档.md) — reason / exclusion 标签的业务规则
