# 本地数据目录（DataRoot）设计

**本设计作为错题本系统的持久化层基准方案。**所有数据均存放在 **用户可选择的位置**，避免依赖系统盘（如 C 盘）。

``` TEXT
DataRoot/
├── db.sqlite              # 核心数据库（唯一）
│
├── assets/                # 所有文件资源（Asset）
│   ├── question/          # 题目原图
│   ├── answer/            # 答案图
│   ├── explain/           # 讲解/批注图（未来）
│   └── other/             # 未分类 / 扩展
│
├── trash/                 # 逻辑删除的资源
│   ├── assets/
│   └── db\_snapshots/
│
├── exports/               # 导出内容（PDF / ZIP）
│
├── backups/               # 备份（可选）
│
└── README.md              # 给“未来的你”
```

## 路径存储原则

- Asset.path 存储 **相对路径**，如：

  - `assets/question/q\_12\_1.png`

- 禁止在数据库中存储绝对路径
- DataRoot 迁移不影响数据库内容
