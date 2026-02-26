//! 本模块负责管理题目的附件（如图片、音频等），提供上传、下载、删除等接口。
//! 除了在输入题目时上传附件外，还可利用本模块的函数在题目编辑后上传新的附件，或删除旧的附件。
//! 同时集成了附件的存储和访问功能，确保附件能够被正确地关联到题目，并且在需要时能够被访问和管理。
//!

/// 单图增添
pub fn add_image() {}

/// 单图逻辑删
pub fn remove_image() {}

/// 单图逻辑恢复
pub fn restore_image() {}

/// 单图物理删
pub fn delete_image_permanently() {}

/// 批量清理超期图片
pub fn clean_expired_images() {}
