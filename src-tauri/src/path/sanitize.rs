use crate::path::error::SanitizeError;

pub(crate) fn sanitize_filename(input: &str) -> Result<String, SanitizeError> {
    if input.contains("..") {
        return Err(SanitizeError::PathTraversal);
    }

    if input.contains('/') || input.contains('\\') {
        return Err(SanitizeError::SeparatorNotAllowed);
    }

    if input.is_empty() {
        return Err(SanitizeError::Empty);
    }

    Ok(input.to_string())
}
