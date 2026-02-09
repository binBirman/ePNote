use crate::path::sanitize;
use crate::path::PathBuilder;
use crate::path::StorageLayout;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use uuid::Uuid;

    #[test]
    fn test_sanitize_filename_valid() {
        let result = sanitize::sanitize_filename("png");
        assert_eq!(result.unwrap(), "png");
    }

    #[test]
    fn test_sanitize_filename_invalid() {
        assert!(sanitize::sanitize_filename("..png").is_err());
        assert!(sanitize::sanitize_filename("/png").is_err());
        assert!(sanitize::sanitize_filename("").is_err());
    }

    #[test]
    fn test_storage_layout_asset_file() {
        let root = PathBuf::from("/tmp/storage");
        let layout = StorageLayout::new(root.clone());
        let id = Uuid::from_bytes([0u8; 16]);
        let ext = "jpg";
        let file_path = layout.asset_file(&id, ext);
        let expected_dir = layout.asset_dir(&id);
        assert!(file_path.starts_with(&expected_dir));
        assert!(file_path.to_string_lossy().ends_with(&format!(".{}", ext)));
    }

    #[test]
    fn test_path_builder_build_asset_path() {
        let root = PathBuf::from("/tmp/storage");
        let layout = StorageLayout::new(root);
        let builder = PathBuilder::new(layout);
        let id = Uuid::from_bytes([1u8; 16]);
        let ext = "png";
        let result = builder.build_asset_path(&id, ext);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.as_path().to_string_lossy().ends_with(".png"));
    }
}
