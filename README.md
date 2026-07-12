# ePNote 电子错题本

> 一个基于间隔重复（Spaced Repetition）算法的错题管理与复习系统。  
> 帮助你系统性地记录、复习和掌握易错知识点。

## 功能特性

### 📝 题目管理
- 创建、编辑、删除题目，支持题目图片和答案图片附件
- 多知识点分类，按科目、题目状态筛选
- 批量操作与管理

### 🧠 智能复习系统
- **间隔重复算法**：根据掌握程度动态计算下次复习时间（`streak² / difficulty`）
- **五态状态机**：NEW → LEARNING → STABLE → DUE，支持 SUSPENDED（暂停）切换
- **复习推荐引擎**：按优先级排序待复习题目，附带原因标签（新题/到期/超期/上次出错/错误率）
- **高频错题标记**：`wrong_count ≥ 3` 且 `error_rate > 0.5` 自动标记

### 📊 数据统计
- 四张概览卡片：总题数、今日复习、总复习次数、正确率
- 分科复习状况表（正确/错误/模糊次数及正确率）
- 月度趋势折线图（复习次数 + 正确率，多科目对比）
- 科目分布环形图

### 🗑️ 回收站
- 误删恢复
- 永久删除
- 30 天自动清理

### ⚙️ 个性化设置
- 时区偏移量（默认 UTC+8）
- 日切时间（默认 03:00）
- 显示调试信息开关
- 开发者模式

## 下载安装

> **当前仅支持 Windows 10/11**，macOS / Linux 版本暂无计划。

前往 [GitHub Releases](https://github.com/binBirman/ePNote/releases) 页面下载最新安装包：

- **`.msi`** — 标准 Windows 安装包，自动配置环境
- **`.exe`** — 免安装便携版，双击即可运行

### 首次使用

1. 启动应用后选择数据存储目录
2. 点击"初始化系统"
3. 重启应用

### 题目状态体系

| 状态 | 说明 |
|------|------|
| NEW | 新建题目，尚未复习 |
| LEARNING | 学习中，尚未达到掌握阈值 |
| STABLE | 已连续答对 3 次进入稳定期 |
| DUE | 已到期，需要重新复习 |
| SUSPENDED | 暂停复习，手动切换 |

### 复习间隔算法

```
interval_days = streak² / difficulty

streak: 连续正确次数
difficulty: 难度系数（由题目自身决定）

FUZZY（模糊）结果：streak -= 1
WRONG（错误）结果：streak = 0 → 下次复习为 1 天
```

## 技术栈

| 层级 | 技术 | 版本要求 |
|------|------|----------|
| 前端框架 | Vue 3 + TypeScript | ^3.5 |
| 构建工具 | Vite | ^7 |
| 桌面框架 | Tauri | 2.x |
| 后端语言 | Rust | >= 1.77.2 |
| 数据库 | SQLite (rusqlite bundled) | — |
| 状态管理 | Pinia | — |
| 路由 | Vue Router | — |

## 项目结构

```
ePNote/
├── src/                         # 前端 Vue 应用
│   ├── api/                     # Tauri invoke 封装
│   ├── components/              # 公共组件
│   ├── router/                  # 路由配置（含 init 守卫）
│   ├── stores/                  # Pinia 状态管理
│   ├── types/                   # TypeScript 类型定义
│   ├── views/                   # 页面视图
│   └── mock/                    # 离线调试 mock 数据
├── src-tauri/                   # Tauri Rust 后端
│   ├── src/
│   │   ├── command/             # Tauri 命令处理器
│   │   ├── server/              # 业务逻辑服务层
│   │   ├── repo/                # 数据访问仓储层
│   │   ├── dao/                 # 数据访问基类
│   │   ├── domain/              # 领域模型 & 状态机
│   │   ├── db/                  # 数据库连接 & 迁移
│   │   └── app/                 # 应用配置 & 设置
│   └── tauri.conf.json
├── docs/                        # 技术文档（详见下文）
├── package.json
└── README.md
```

## 技术文档

详细设计文档位于 `docs/` 目录：

| 目录 | 内容 |
|------|------|
| `docs/architecture/` | 项目结构、前后端架构、数据目录说明 |
| `docs/business/` | 业务规则、状态转移模型、时间管理设计 |
| `docs/database/` | 数据库设计、迁移策略 |
| `docs/ui/` | UI 设计规范 |

## 开发环境要求

- Node.js >= 20.19.0
- Rust >= 1.77.2（通过 rustup 安装，需包含 `wasm32-unknown-unknown` target）
- Windows 10/11（当前支持平台）

## 快速开始

```bash
# 1. 安装前端依赖
npm install

# 2. 前端开发模式（仅 Vite，无 Rust 后端）
npm run dev

# 3. 全栈 Tauri 开发模式（启动 Rust 后端）
npm run tauri dev

# 4. 生产构建
npm run tauri build
```

## 可用脚本

| 命令 | 说明 |
|------|------|
| `npm run dev` | Vite 前端开发服务器 |
| `npm run tauri dev` | Tauri 全栈开发 |
| `npm run build` | 类型检查 + Vite 构建 |
| `npm run type-check` | vue-tsc 类型检查 |
| `npm run lint` | ESLint 代码检查（--fix --cache） |
| `cargo test` | Rust 单元测试 |
| `cargo check` | Rust 编译检查 |

## 配置

配置文件位于 `src-tauri/app_config.json`：

```json
{
  "root": "数据存储路径"
}
```

运行时设置（通过应用内 UI 调整）：
- 时区偏移量（默认 UTC+8）
- 日切时间（默认 03:00）
- 显示推荐调试信息
- 开发者模式

## License

MIT
