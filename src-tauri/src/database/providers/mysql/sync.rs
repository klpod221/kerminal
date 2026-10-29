// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::Row;
use std::collections::HashMap;

use crate::database::error::{DatabaseError, DatabaseResult};

use super::MySQLProvider;

const ALLOWED_TABLES: &[&str] = &[
    "ssh_profiles",
    "ssh_groups",
    "ssh_keys",
    "ssh_tunnels",
    "saved_commands",
    "saved_command_groups",
];

fn validate_table_name(table: &str) -> DatabaseResult<()> {
    if ALLOWED_TABLES.contains(&table) {
        Ok(())
    } else {
        Err(DatabaseError::ValidationError(format!("Invalid table name: {}", table)))
    }
}

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

fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    for ch in s.chars() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            for c in ch.to_uppercase() {
                result.push(c);
            }
            capitalize_next = false;
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
    validate_table_name(table)?;
    if records.is_empty() {
        return Ok(0);
    }

    let pool = provider.get_pool()?;
    let pool = pool.read().await;

    let mut count = 0;

    for record in records.iter() {
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
    validate_table_name(table)?;
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
    validate_table_name(table)?;
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
        let camel_key = to_camel_case(&col_name);

        let val = if let Ok(Some(b)) = row.try_get::<Option<bool>, _>(idx) {
            Value::Bool(b)
        } else if let Ok(Some(i)) = row.try_get::<Option<i64>, _>(idx) {
            Value::Number(serde_json::Number::from(i))
        } else if let Ok(Some(i)) = row.try_get::<Option<i32>, _>(idx) {
            Value::Number(serde_json::Number::from(i))
        } else if let Ok(Some(f)) = row.try_get::<Option<f64>, _>(idx) {
            serde_json::Number::from_f64(f)
                .map(Value::Number)
                .unwrap_or(Value::Null)
        } else if let Ok(Some(s)) = row.try_get::<Option<String>, _>(idx) {
            if (s.starts_with('{') && s.ends_with('}')) || (s.starts_with('[') && s.ends_with(']')) {
                serde_json::from_str(&s).unwrap_or(Value::String(s))
            } else {
                Value::String(s)
            }
        } else {
            Value::Null
        };

        obj.insert(camel_key, val.clone());
        if !obj.contains_key(&col_name) {
            obj.insert(col_name, val);
        }
    }

    Ok(Value::Object(obj))
}
