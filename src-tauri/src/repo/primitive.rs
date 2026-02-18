//!
//! 该文件定义了domain层的枚举类型转与对应基本类型间的转换相关的内容
//!

use crate::domain::enums::*;
use crate::domain::ids::*;
use crate::repo::error::ConvertError;

use uuid::Uuid;

// ------------------------------------------------------------
// QuestionId <-> i64
// ------------------------------------------------------------

impl From<QuestionId> for i64 {
    fn from(id: QuestionId) -> Self {
        let u = id.0.as_u128();
        let low = u as u64;
        low as i64
    }
}

impl From<i64> for QuestionId {
    fn from(n: i64) -> Self {
        let low = n as u64 as u128;
        QuestionId(Uuid::from_u128(low))
    }
}

// ------------------------------------------------------------
// QuestionState <-> string
// ------------------------------------------------------------

impl From<QuestionState> for String {
    fn from(state: QuestionState) -> Self {
        state.as_str().to_string()
    }
}

impl TryFrom<String> for QuestionState {
    type Error = ConvertError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        QuestionState::from_str(&s).ok_or(ConvertError::InvalidQuestionState(s))
    }
}

// ------------------------------------------------------------
// ReviewId <-> i64
// ------------------------------------------------------------

impl From<ReviewId> for i64 {
    fn from(id: ReviewId) -> Self {
        let u = id.0.as_u128();
        let low = u as u64;
        low as i64
    }
}

impl From<i64> for ReviewId {
    fn from(n: i64) -> Self {
        let low = n as u64 as u128;
        ReviewId(Uuid::from_u128(low))
    }
}

// ------------------------------------------------------------
// ReviewResult <-> string
// ------------------------------------------------------------

impl From<ReviewResult> for String {
    fn from(result: ReviewResult) -> Self {
        result.as_str().to_string()
    }
}

impl TryFrom<String> for ReviewResult {
    type Error = ConvertError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        ReviewResult::from_str(&s).ok_or(ConvertError::InvalidReviewResult(s))
    }
}

// ------------------------------------------------------------
// AssetId <-> i64
// ------------------------------------------------------------

impl From<AssetId> for i64 {
    fn from(id: AssetId) -> Self {
        let u = id.0.as_u128();
        let low = u as u64;
        low as i64
    }
}

impl From<i64> for AssetId {
    fn from(n: i64) -> Self {
        let low = n as u64 as u128;
        AssetId(Uuid::from_u128(low))
    }
}

// ------------------------------------------------------------
// AssetType <-> string
// ------------------------------------------------------------

impl From<AssetType> for String {
    fn from(asset_type: AssetType) -> Self {
        asset_type.as_str().to_string()
    }
}

impl TryFrom<String> for AssetType {
    type Error = ConvertError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        AssetType::from_str(&s).ok_or(ConvertError::InvalidAssetType(s))
    }
}

// ------------------------------------------------------------
// MetaId <-> i64
// ------------------------------------------------------------

impl From<MetaId> for i64 {
    fn from(id: MetaId) -> Self {
        let u = id.0.as_u128();
        let low = u as u64;
        low as i64
    }
}

impl From<i64> for MetaId {
    fn from(n: i64) -> Self {
        let low = n as u64 as u128;
        MetaId(Uuid::from_u128(low))
    }
}

// ------------------------------------------------------------
// MetaKey <-> string
// ------------------------------------------------------------

impl From<MetaKey> for String {
    fn from(meta_key: MetaKey) -> Self {
        meta_key.as_str()
    }
}

impl TryFrom<String> for MetaKey {
    type Error = ConvertError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        MetaKey::from_str(&s).ok_or(ConvertError::InvalidMetaKey(s))
    }
}

// // Generic trait to convert ids from/to `i64` using the embedding strategy.
// pub trait IdFromI64: Sized {
//     fn from_i64(n: i64) -> Self;
//     fn to_i64(self) -> i64;
// }

// impl IdFromI64 for QuestionId {
//     fn from_i64(n: i64) -> Self {
//         n.into()
//     }
//     fn to_i64(self) -> i64 {
//         self.into()
//     }
// }

// impl IdFromI64 for ReviewId {
//     fn from_i64(n: i64) -> Self {
//         n.into()
//     }
//     fn to_i64(self) -> i64 {
//         self.into()
//     }
// }

// impl IdFromI64 for AssetId {
//     fn from_i64(n: i64) -> Self {
//         n.into()
//     }
//     fn to_i64(self) -> i64 {
//         self.into()
//     }
// }

// impl IdFromI64 for MetaId {
//     fn from_i64(n: i64) -> Self {
//         n.into()
//     }
//     fn to_i64(self) -> i64 {
//         self.into()
//     }
// }
