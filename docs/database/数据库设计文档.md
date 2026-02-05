# 错题本系统 · 数据库设计文档（V1）

> 本文档定义错题本系统的\*\*数据库结构设计\*\*，用于支撑题目管理、复习状态机、复习统计等核心业务逻辑。
>
> 当前版本聚焦于：
>
> - 题目命名规则
> - 题目与复习状态存储
> - 复习记录事件模型

---

## 1\. 设计原则

1. **Review 是事实，Question 是状态**

   - 复习记录只增不改，作为系统的唯一事实来源
   - 题目表保存当前可变的派生状态（缓存）

2. **状态可重算，字段不过度绑定策略**

   - 复习策略参数（阈值、间隔）不写死在表结构中

3. **字段语义清晰，避免冗余**

   - 一个字段只承担一个业务含义

4. **支持长期演进**

   - 允许未来新增复习策略、统计维度而不破坏现有数据

---

## 2\. 核心实体与关系概览

``` UML
Question (1) ──── (N) Review
```

- Question：题目主体 + 当前复习状态
- Review：每一次复习行为的不可变记录

---

## 3\. questions 表（题目表）

题目表是**系统状态机的落点**，保存题目的当前复习状态与基础信息。

### 3.1 表结构

|字段名|类型|说明|
|-|-|-|
|id|INTEGER (PK)|题目唯一标识|
|name|TEXT|题目名称（可为空）|
|state|TEXT|当前复习状态|
|created\_at|INTEGER|创建时间|
|deleted\_at|INTEGER|逻辑删除时间（可为空）|

#### 复习状态相关字段视图

|字段名|类型|说明|
|-|-|-|
|last\_review\_at|INTEGER|最近一次复习时间|
|last\_result|TEXT|最近一次复习结果（correct / wrong / fuzzy）|
|correct\_streak|INTEGER|连续答对次数|
|wrong\_count|INTEGER|累计错误次数|
|due\_at|INTEGER|下次建议复习时间|

---

### 3.2 题目命名规则（业务约定）

- 题目**允许用户命名**，用于增强可读性与检索
- `name` 字段可为空
- UI 层展示规则：

  - 若 `name` 非空：显示 `name + 题目信息`
  - 若 `name` 为空：默认展示为 `题目 #<id>`

> 数据库层面不做自动填充，保持字段语义纯净

---

### 3.3 state 字段定义

state 字段用于表达题目在复习状态机中的当前位置。

允许值：

- `NEW`：新题，尚未复习
- `LEARNING`：学习中，理解不稳定
- `STABLE`：稳定掌握，低频复习
- `DUE`：已到建议复习时间
- `SUSPENDED`：用户暂停复习

---

## 4\. reviews 表（复习记录表）

复习记录表是**事件源**，记录每一次真实发生的复习行为。

### 4.1 表结构

|字段名|类型|说明|
|-|-|-|
|id|INTEGER (PK)|复习记录唯一标识|
|question\_id|INTEGER (FK)|对应题目 ID|
|result|TEXT|复习结果（correct / wrong / fuzzy）|
|reviewed\_at|INTEGER|复习发生时间|

---

### 4.2 设计约束

- 每一次复习行为 **必须生成一条记录**
- Review 记录 **不可修改、不可覆盖**
- 不在 Review 表中存储任何状态字段

---

## 5\. 状态字段与状态机映射说明

|状态|关键判定依据|
|-|-|
|NEW|last\_review\_at IS NULL|
|LEARNING|最近结果为 wrong / fuzzy，或连续正确次数不足|
|STABLE|连续正确次数达到阈值，且未到期|
|DUE|当前时间 ≥ due\_at|
|SUSPENDED|state = SUSPENDED|

> state 字段作为当前状态的显式标记，用于 UI 与推荐逻辑

---

## 6\. 一次复习行为的数据更新原则（摘要）

1. 插入一条 Review 记录
2. 更新 Question 表中的：

   - last\_review\_at
   - last\_result
   - correct\_streak / wrong\_count
   - due\_at

3. 根据规则更新 state

> 所有更新应在同一事务中完成

---

## 7\. 资源（Asset）与本地数据目录设计

本系统采用 **文件系统 + 数据库引用** 的资源管理方式：

- 数据库存储资源“元信息与关联关系”
- 文件系统存储真实图片文件

---

## 7.1 Asset 表（资源表）

Asset 表用于描述所有与题目相关的文件资源（图片为主）。

### 7.1.1 表结构

|字段名|类型|说明|
|-|-|-|
|id|INTEGER (PK)|资源唯一标识|
|question\_id|INTEGER (FK)|所属题目|
|type|TEXT|资源类型|
|path|TEXT|相对 DataRoot 的路径|
|created\_at|INTEGER|创建时间|
|deleted\_at|INTEGER|逻辑删除时间（可为空）|

#### type 字段约定值

- `question`：题目原图
- `answer`：答案图
- `explain`：讲解 / 批注图（预留）
- `other`：未分类或扩展用途

> type 是语义标签，不决定文件实际存放位置，但应与目录结构保持一致

---

## 7.2 Question / Review / Asset 关系说明

``` UML
Question (1)
   ├── (N) Review   // 复习历史事件
   └── (N) Asset    // 图片 / 资源文件
```

- 一个题目可有多张题目图、答案图
- Review 与 Asset 之间 **无直接关系**

---

## 8\. Tag / Meta（元数据）设计

用于存储题目的可扩展属性（试卷、科目、知识点等）。

### 8.1 meta 表结构

|字段名|类型|说明|
|-|-|-|
|id|INTEGER (PK)|唯一标识|
|question\_id|INTEGER (FK)|所属题目|
|key|TEXT|元数据键|
|value|TEXT|元数据值|

### 8.2 设计说明

- 采用 **EAV（Entity-Attribute-Value）** 模式
- 支持灵活扩展而无需改表
- 常见 key 示例：

  - `subject`：科目
  - `paper`：试卷
  - `knowledge`：知识点

---

## 8.3 Meta.key 命名空间规范

为避免 key 混乱，Meta.key 采用 **命名空间约定**，而非数据库强约束。

### 8.3.1 key 分类

|类型|前缀|说明|
|-|-|-|
|系统保留|`sys.`|系统核心语义字段|
|官方扩展|`ext.`|系统预置但可选字段|
|用户自定义|`user.`|用户自行创建|

---

### 8.3.2 系统保留 key（sys.*）

这些 key 在业务中具有明确语义，**UI 与逻辑可依赖**。

|key|含义|示例 value|
|-|-|-|
|sys.subject|科目|数学 / 英语|
|sys.paper|试卷|2023 模拟卷一|
|sys.knowledge|知识点|二次函数|
|sys.source|题目来源|真题 / 模拟|

规则：

- sys.* 由系统创建
- 不允许用户随意重命名 key
- value 仍为自由文本

---

### 8.3.3 官方扩展 key（ext.*）

用于系统内置但不参与核心逻辑的增强字段。

|key|含义|
|-|-|
|ext.difficulty|主观难度|
|ext.import_batch|导入批次|
|ext.note|简要备注|

规则：

- ext.* 可在 UI 中隐藏或显示
- 不影响复习状态机

---

### 8.3.4 用户自定义 key（user.*）

完全由用户定义，用于个性化整理。

示例：

- user.teacher = 张老师
- user.mistake_type = 计算错误
- user.tag = 易错

规则：

- 系统不理解其语义
- 仅用于筛选、展示

---

## 8.4 value 取值规范（软约定）

- value 一律使用 TEXT
- 多值采用分隔符或多行 Meta 记录（推荐后者）

示例（多知识点）：

``` Shell
sys.knowledge = 导数
sys.knowledge = 极值
```

---

## 8.5 UI 行为约定（非数据库约束）

- sys.*

  - 固定表单控件（下拉 / 自动补全）
- ext.*

  - 可配置是否展示
- user.*

  - 键值对自由编辑

---

## 8.6 索引与性能建议

建议建立联合索引：

- (question_id, key)
- (key, value)

用于支持：

- 按题目快速加载 Meta
- 按条件筛选题目

---

---|------|------|
| id | INTEGER (PK) | 唯一标识 |
| question_id | INTEGER (FK) | 所属题目 |
| key | TEXT | 元数据键 |
| value | TEXT | 元数据值 |

## 9\. 删除与回收策略（摘要）

- 删除题目：

  - 标记 Question.deleted\_at
  - 关联 Asset 移动至 trash/assets

- 数据库快照可存入 trash/db\_snapshots

---

## 10\. 总结（V1 范围）

- Asset 表统一管理所有图片与文件
- 文件系统结构清晰、可迁移、可备份
- Meta 表支持长期扩展的标签体系
