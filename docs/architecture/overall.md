# 项目结构

``` TEXT
project-root/
│
├── src/                    # Vue 前端
│   ├── app/                # 应用壳（路由、布局）
│   │
│   ├── domains/            # ⭐ 业务域（最重要）
│   │   ├── question/
│   │   │   ├── QuestionList.vue
│   │   │   ├── QuestionDetail.vue
│   │   │   ├── question.store.ts
│   │   │   ├── question.service.ts
│   │   │   └── types.ts
│   │   │
│   │   ├── review/
│   │   │   ├── ReviewPanel.vue
│   │   │   ├── review.service.ts
│   │   │   └── strategy.ts
│   │   │
│   │   ├── asset/
│   │   │   ├── ImageViewer.vue
│   │   │   ├── asset.service.ts
│   │   │   └── types.ts
│   │   │
│   │   └── meta/
│   │       ├── MetaEditor.vue
│   │       ├── meta.service.ts
│   │       └── keys.ts      # sys./ext./user 常量
│   │
│   ├── shared/             # 跨域复用
│   │   ├── components/
│   │   ├── ui/
│   │   └── utils/
│   │
│   └── main.ts
│
├── src-tauri/                # Rust 后端（核心）
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       │
│       ├── app/              # 应用生命周期 & 入口
│       │   ├── mod.rs
│       │   └── init.rs       # DataRoot / DB / migrate
│       │
│       ├── db/               # 数据库子系统（重点）
│       │   ├── mod.rs
│       │   ├── conn.rs       # 打开 SQLite
│       │   ├── migrate.rs    # 迁移系统（你刚加的）
│       │   └── schema.rs     # 表结构 & SQL
│       │
│       ├── model/            # 数据模型（与表对应）
│       │   ├── mod.rs
│       │   ├── question.rs
│       │   ├── review.rs
│       │   ├── asset.rs
│       │   └── meta.rs
│       │
│       ├── repo/             # 数据访问（CRUD）
│       │   ├── mod.rs
│       │   ├── question_repo.rs
│       │   ├── review_repo.rs
│       │   └── asset_repo.rs
│       │
│       ├── service/          # 业务逻辑（Phase 1 可很薄）
│       │   ├── mod.rs
│       │   └── review_service.rs
│       │
│       └── error.rs          # 统一错误定义
│
├── docs/                    # 设计文档
│
├── scripts/                 # 辅助脚本（导入 / 校验）
│
└── README.md                # 项目说明（开发者）
```
