//! 该模块负责管理题目视图（`show_view`），提供查询接口。
//! 题目视图是一个数据库视图，包含题目的基本信息和一些常用的元信息（如学科）。它的存在是为了简化查询，避免每次都要 join 多张表。
//! 视图的定义在 `src-tauri/src/db/migrate.rs` 中的迁移脚本里，查询接口在 `ViewDao` 中实现。
//! 题目管理器（`question_manager.rs`）会调用 `ViewDao` 来获取题目视图数据，并将其转换为领域模型 `Question`，供上层调用。
//! 主要功能为向UI层提供题目基本信息，但不包含图片等具体信息。
//! 只在题目管理与查询界面使用。

/// 按页显示题目
pub fn list_available_questions_page() {}

/// 按页显示已删除题目
pub fn list_deleted_questions_page() {}

/// 按页显示某科目下的题目
pub fn list_questions_by_subject_page() {}

/// 按页显示某状态下的题目
pub fn list_questions_by_state_page() {}

/// 显示按id查找题目的结果
pub fn find_question_by_id() {}

/// 显示按name查找题目的结果
pub fn find_question_by_name() {}
