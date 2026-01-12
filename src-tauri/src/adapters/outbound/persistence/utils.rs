//! Persistence Utilities
//!
//! Common utilities for database operations including error conversion,
//! datetime handling, and binary serialization.

use chrono::{DateTime, TimeZone, Utc};
use diesel::result::Error as DieselError;

use crate::domain::errors::DomainError;

/// Convert Diesel error to DomainError
pub fn map_diesel_error(e: DieselError) -> DomainError {
    match e {
        DieselError::NotFound => DomainError::NotFound("Record not found".to_string()),
        DieselError::DatabaseError(kind, info) => {
            DomainError::DatabaseError(format!("{:?}: {}", kind, info.message()))
        }
        DieselError::InvalidCString(nul_error) => {
            DomainError::DatabaseError(format!("Invalid C string: {}", nul_error))
        }
        DieselError::QueryBuilderError(msg) => {
            DomainError::DatabaseError(format!("Query builder error: {}", msg))
        }
        DieselError::DeserializationError(e) => {
            DomainError::DatabaseError(format!("Deserialization error: {}", e))
        }
        DieselError::SerializationError(e) => {
            DomainError::DatabaseError(format!("Serialization error: {}", e))
        }
        DieselError::RollbackErrorOnCommit { rollback_error, commit_error } => {
            DomainError::DatabaseError(format!(
                "Rollback error on commit - Rollback: {:?}, Commit: {:?}",
                rollback_error, commit_error
            ))
        }
        DieselError::RollbackTransaction => {
            DomainError::DatabaseError("Transaction rolled back".to_string())
        }
        DieselError::AlreadyInTransaction => {
            DomainError::DatabaseError("Already in transaction".to_string())
        }
        DieselError::NotInTransaction => {
            DomainError::DatabaseError("Not in transaction".to_string())
        }
        DieselError::BrokenTransactionManager => {
            DomainError::DatabaseError("Broken transaction manager".to_string())
        }
        _ => DomainError::DatabaseError(format!("Database error: {}", e)),
    }
}

/// Convert chrono DateTime to Unix timestamp (i64)
pub fn datetime_to_timestamp(dt: &DateTime<Utc>) -> i64 {
    dt.timestamp()
}

/// Convert Unix timestamp (i64) to chrono DateTime
pub fn timestamp_to_datetime(timestamp: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(timestamp, 0).unwrap()
}

/// Convert Option<DateTime> to Option<i64>
pub fn optional_datetime_to_timestamp(dt: &Option<DateTime<Utc>>) -> Option<i64> {
    dt.as_ref().map(datetime_to_timestamp)
}

/// Convert Option<i64> to Option<DateTime>
pub fn optional_timestamp_to_datetime(timestamp: Option<i64>) -> Option<DateTime<Utc>> {
    timestamp.map(timestamp_to_datetime)
}

/// Serialize f32 vector to binary (for embeddings)
pub fn serialize_embedding(embedding: &[f32]) -> Vec<u8> {
    embedding
        .iter()
        .flat_map(|&f| f.to_le_bytes())
        .collect()
}

/// Deserialize binary to f32 vector (for embeddings)
pub fn deserialize_embedding(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| {
            let array: [u8; 4] = chunk.try_into().unwrap();
            f32::from_le_bytes(array)
        })
        .collect()
}

/// Convert boolean to SQLite integer (0 or 1)
pub fn bool_to_i32(b: bool) -> i32 {
    if b { 1 } else { 0 }
}

/// Convert SQLite integer to boolean
pub fn i32_to_bool(i: i32) -> bool {
    i != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_datetime_conversion() {
        let now = Utc::now();
        let timestamp = datetime_to_timestamp(&now);
        let converted = timestamp_to_datetime(timestamp);

        // Should be equal within a second (timestamp precision)
        assert_eq!(now.timestamp(), converted.timestamp());
    }

    #[test]
    fn test_optional_datetime_conversion() {
        let some_dt = Some(Utc::now());
        let timestamp = optional_datetime_to_timestamp(&some_dt);
        assert!(timestamp.is_some());

        let none_dt: Option<DateTime<Utc>> = None;
        let timestamp = optional_datetime_to_timestamp(&none_dt);
        assert!(timestamp.is_none());
    }

    #[test]
    fn test_embedding_serialization() {
        let original = vec![1.0, 2.5, -3.7, 0.0];
        let bytes = serialize_embedding(&original);
        let deserialized = deserialize_embedding(&bytes);

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_bool_conversion() {
        assert_eq!(bool_to_i32(true), 1);
        assert_eq!(bool_to_i32(false), 0);
        assert_eq!(i32_to_bool(1), true);
        assert_eq!(i32_to_bool(0), false);
        assert_eq!(i32_to_bool(42), true); // Non-zero is true
    }
}
