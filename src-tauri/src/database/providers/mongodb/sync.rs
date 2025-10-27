use bson::{doc, Bson, Document};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;

use crate::database::error::{DatabaseError, DatabaseResult};

use super::MongoDBProvider;

/// Convert camelCase to snake_case for MongoDB field names
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }
    result
}

/// Push records to MongoDB database using upsert (replaceOne with upsert=true)
pub async fn push_records(
    provider: &MongoDBProvider,
    collection_name: &str,
    records: Vec<Value>,
) -> DatabaseResult<usize> {
    if records.is_empty() {
        return Ok(0);
    }

    let collection = provider.get_collection(collection_name).await?;
    let mut count = 0;

    for record in records {
        let doc = json_to_bson_document(&record)?;

        if let Some(Bson::String(id)) = doc.get("id") {
            let filter = doc! { "id": id };
            let options = mongodb::options::ReplaceOptions::builder()
                .upsert(true)
                .build();

            collection
                .replace_one(filter, doc, options)
                .await
                .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

            count += 1;
        }
    }

    Ok(count)
}

/// Pull records from MongoDB database modified since timestamp
pub async fn pull_records(
    provider: &MongoDBProvider,
    collection_name: &str,
    since: Option<DateTime<Utc>>,
) -> DatabaseResult<Vec<Value>> {
    let collection = provider.get_collection(collection_name).await?;

    let filter = if let Some(since_time) = since {
        doc! { "updated_at": { "$gt": since_time.to_rfc3339() } }
    } else {
        doc! {}
    };

    let options = mongodb::options::FindOptions::builder()
        .sort(doc! { "updated_at": 1 })
        .build();

    let mut cursor = collection
        .find(filter, options)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let mut records = Vec::new();
    while cursor
        .advance()
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
    {
        let doc = cursor
            .deserialize_current()
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        let json = bson_document_to_json(&doc)?;
        records.push(json);
    }

    Ok(records)
}

/// Get record versions for conflict detection
pub async fn get_record_versions(
    provider: &MongoDBProvider,
    collection_name: &str,
    ids: Vec<String>,
) -> DatabaseResult<HashMap<String, u64>> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }

    let collection = provider.get_collection(collection_name).await?;
    let filter = doc! { "id": { "$in": ids } };
    let options = mongodb::options::FindOptions::builder()
        .projection(doc! { "id": 1, "version": 1 })
        .build();

    let mut cursor = collection
        .find(filter, options)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let mut versions = HashMap::new();
    while cursor
        .advance()
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?
    {
        let doc = cursor
            .deserialize_current()
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        if let (Some(Bson::String(id)), Some(version)) = (doc.get("id"), doc.get("version")) {
            let version_u64 = match version {
                Bson::Int32(v) => *v as u64,
                Bson::Int64(v) => *v as u64,
                _ => 0,
            };
            versions.insert(id.clone(), version_u64);
        }
    }

    Ok(versions)
}

/// Helper function to convert JSON Value to BSON Document
fn json_to_bson_document(value: &Value) -> DatabaseResult<Document> {
    let mut doc = Document::new();

    if let Some(obj) = value.as_object() {
        for (key, val) in obj {
            let snake_key = to_snake_case(key);
            let bson_val: Bson = serde_json::to_string(val)
                .and_then(|s| serde_json::from_str(&s))
                .map_err(|e| DatabaseError::SerializationError(e))?;
            doc.insert(snake_key, bson_val);
        }
    }

    Ok(doc)
}

/// Helper function to convert BSON Document to JSON Value
fn bson_document_to_json(doc: &Document) -> DatabaseResult<Value> {
    let json = serde_json::to_value(doc).map_err(|e| DatabaseError::SerializationError(e))?;

    // Convert snake_case keys back to camelCase
    if let Some(obj) = json.as_object() {
        let mut camel_obj = serde_json::Map::new();
        for (key, val) in obj {
            let camel_key = to_camel_case(key);
            camel_obj.insert(camel_key, val.clone());
        }
        Ok(Value::Object(camel_obj))
    } else {
        Ok(json)
    }
}

/// Convert snake_case to camelCase
fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for ch in s.chars() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }

    result
}
