use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::Row;
use std::collections::HashMap;

use crate::database::error::{DatabaseError, DatabaseResult};

use super::MySQLProvider;

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

fn convert_datetime_value(value: &Value) -> Value {
    if let Some(s) = value.as_str() {
        if s.contains('T') && (s.ends_with('Z') || s.contains('+') || s.contains('-')) {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                return Value::String(dt.format("%Y-%m-%d %H:%M:%S").to_string());
            }
        }
    }
    value.clone()
}

pub async fn push_records(
    provider: &MySQLProvider,
    table: &str,
    records: Vec<Value>,
) -> DatabaseResult<usize> {
    if records.is_empty() {
        return Ok(0);
    }

    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    let mut count = 0;

    for (_idx, record) in records.iter().enumerate() {
        let obj = record
            .as_object()
            .ok_or_else(|| DatabaseError::QueryFailed("Expected JSON object".to_string()))?;

        let column_mapping: Vec<(String, String)> = obj
            .keys()
            .map(|k| (k.to_string(), to_snake_case(k)))
            .collect();

        let db_columns: Vec<String> = column_mapping
            .iter()
            .map(|(_, db_col)| db_col.clone())
            .collect();
        let placeholders: Vec<String> = (0..db_columns.len()).map(|_| "?".to_string()).collect();
        let updates: Vec<String> = db_columns
            .iter()
            .filter(|c| c.as_str() != "id")
            .map(|c| format!("{} = VALUES({})", c, c))
            .collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({}) ON DUPLICATE KEY UPDATE {}",
            table,
            db_columns.join(", "),
            placeholders.join(", "),
            updates.join(", ")
        );

        let mut query = sqlx::query(&sql);
        for (json_key, _) in &column_mapping {
            let value = &obj[json_key];
            let converted_value = convert_datetime_value(value);
            query = bind_value(query, &converted_value);
        }

        let _result = query
            .execute(&*pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        count += 1;
    }

    Ok(count)
}

pub async fn pull_records(
    provider: &MySQLProvider,
    table: &str,
    since: Option<DateTime<Utc>>,
) -> DatabaseResult<Vec<Value>> {
    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    let sql = if let Some(_since_time) = since {
        format!(
            "SELECT * FROM {} WHERE updated_at > ? ORDER BY updated_at ASC",
            table
        )
    } else {
        format!("SELECT * FROM {} ORDER BY updated_at ASC", table)
    };

    let rows = if let Some(since_time) = since {
        sqlx::query(&sql)
            .bind(since_time.to_rfc3339())
            .fetch_all(&*pool)
            .await
    } else {
        sqlx::query(&sql).fetch_all(&*pool).await
    }
    .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let mut records = Vec::new();
    for row in rows {
        let record = row_to_json(&row)?;
        records.push(record);
    }

    Ok(records)
}

pub async fn get_record_versions(
    provider: &MySQLProvider,
    table: &str,
    ids: Vec<String>,
) -> DatabaseResult<HashMap<String, u64>> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }

    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    let placeholders: Vec<String> = (0..ids.len()).map(|_| "?".to_string()).collect();
    let sql = format!(
        "SELECT id, version FROM {} WHERE id IN ({})",
        table,
        placeholders.join(", ")
    );

    let mut query = sqlx::query(&sql);
    for id in ids {
        query = query.bind(id);
    }

    let rows = query
        .fetch_all(&*pool)
        .await
        .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

    let mut versions = HashMap::new();
    for row in rows {
        let id: String = row
            .try_get("id")
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
        let version: i64 = row
            .try_get("version")
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
        versions.insert(id, version as u64);
    }

    Ok(versions)
}

fn bind_value<'q>(
    query: sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments>,
    value: &Value,
) -> sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments> {
    match value {
        Value::Null => query.bind(Option::<String>::None),
        Value::Bool(b) => query.bind(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                query.bind(i)
            } else if let Some(u) = n.as_u64() {
                query.bind(u as i64)
            } else if let Some(f) = n.as_f64() {
                query.bind(f)
            } else {
                query.bind(n.to_string())
            }
        }
        Value::String(s) => query.bind(s.clone()),
        Value::Array(_) | Value::Object(_) => query.bind(value.to_string()),
    }
}

fn row_to_json(row: &sqlx::mysql::MySqlRow) -> DatabaseResult<Value> {
    use sqlx::Column;

    let mut obj = serde_json::Map::new();

    for (idx, col) in row.columns().iter().enumerate() {
        let col_name = col.name().to_string();
        let value: Option<String> = row.try_get(idx).ok();

        if let Some(v) = value {
            obj.insert(col_name, Value::String(v));
        } else {
            obj.insert(col_name, Value::Null);
        }
    }

    Ok(Value::Object(obj))
}
