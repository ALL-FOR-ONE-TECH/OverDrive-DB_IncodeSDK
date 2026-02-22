//! Embedded SQL Query Engine for OverDrive InCode SDK
//! 
//! Parses and executes SQL queries directly against the embedded database.
//! No server needed — runs entirely in-process.
//! 
//! ## Supported SQL
//! 
//! - `SELECT [columns] FROM <table> [WHERE ...] [ORDER BY ...] [LIMIT n] [OFFSET n]`
//! - `INSERT INTO <table> {json}`
//! - `UPDATE <table> SET {json} [WHERE ...]`
//! - `DELETE FROM <table> [WHERE ...]`
//! - `CREATE TABLE <name>`
//! - `DROP TABLE <name>`
//! - `SHOW TABLES`
//! - `SELECT COUNT(*) FROM <table>` (and other aggregates)

use crate::{OverDriveDB, QueryResult};
use crate::result::{SdkResult, SdkError};
use serde_json::Value;
use overdrive_db::query::aggregation::{AggregateFunc, Aggregator};
use overdrive_db::query::orderby::{OrderBy, SortDirection, Sorter};

/// Execute an SQL query against the embedded database
pub fn execute(db: &mut OverDriveDB, sql: &str) -> SdkResult<QueryResult> {
    let sql = sql.trim().trim_end_matches(';').trim();
    
    if sql.is_empty() {
        return Ok(QueryResult::empty());
    }
    
    // Tokenize: split into words respecting quoted strings and braces
    let tokens = tokenize(sql);
    
    if tokens.is_empty() {
        return Ok(QueryResult::empty());
    }
    
    let first = tokens[0].to_uppercase();
    
    match first.as_str() {
        "SELECT" => execute_select(db, &tokens),
        "INSERT" => execute_insert(db, &tokens, sql),
        "UPDATE" => execute_update(db, &tokens, sql),
        "DELETE" => execute_delete(db, &tokens),
        "CREATE" => execute_create(db, &tokens),
        "DROP"   => execute_drop(db, &tokens),
        "SHOW"   => execute_show(db, &tokens),
        _ => Err(SdkError::InvalidQuery(format!("Unsupported SQL command: {}", first))),
    }
}

/// Simple tokenizer that respects quoted strings and JSON braces
fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        
        // Quoted string
        if c == '\'' || c == '"' {
            let quote = c;
            chars.next(); // consume opening quote
            let mut s = String::new();
            while let Some(&ch) = chars.peek() {
                if ch == quote {
                    chars.next(); // consume closing quote
                    break;
                }
                if ch == '\\' {
                    chars.next();
                    if let Some(&escaped) = chars.peek() {
                        s.push(escaped);
                        chars.next();
                        continue;
                    }
                }
                s.push(ch);
                chars.next();
            }
            tokens.push(format!("'{}'", s));
            continue;
        }
        
        // JSON object or array
        if c == '{' || c == '[' {
            let (close, open) = if c == '{' { ('}', '{') } else { (']', '[') };
            let mut depth = 0;
            let mut s = String::new();
            while let Some(&ch) = chars.peek() {
                s.push(ch);
                if ch == open { depth += 1; }
                if ch == close { depth -= 1; }
                chars.next();
                if depth == 0 { break; }
            }
            tokens.push(s);
            continue;
        }
        
        // Operators
        if c == '>' || c == '<' || c == '!' || c == '=' {
            let mut op = String::new();
            op.push(c);
            chars.next();
            if let Some(&next) = chars.peek() {
                if next == '=' {
                    op.push(next);
                    chars.next();
                }
            }
            tokens.push(op);
            continue;
        }
        
        // Comma, parentheses, star
        if c == ',' || c == '(' || c == ')' || c == '*' {
            tokens.push(c.to_string());
            chars.next();
            continue;
        }
        
        // Word or number
        let mut word = String::new();
        while let Some(&ch) = chars.peek() {
            if ch.is_whitespace() || ch == ',' || ch == '(' || ch == ')' 
               || ch == '{' || ch == '[' || ch == '>' || ch == '<' 
               || ch == '=' || ch == '!' {
                break;
            }
            word.push(ch);
            chars.next();
        }
        if !word.is_empty() {
            tokens.push(word);
        }
    }
    
    tokens
}

/// Execute SELECT query
fn execute_select(db: &mut OverDriveDB, tokens: &[String]) -> SdkResult<QueryResult> {
    // Parse: SELECT <columns> FROM <table> [WHERE ...] [ORDER BY ...] [LIMIT n] [OFFSET n]
    let mut pos = 1; // skip "SELECT"
    
    // Parse columns
    let mut columns: Vec<String> = Vec::new();
    let mut aggregates: Vec<AggregateFunc> = Vec::new();
    
    while pos < tokens.len() {
        let upper = tokens[pos].to_uppercase();
        if upper == "FROM" {
            break;
        }
        
        let col = tokens[pos].trim_end_matches(',').to_string();
        
        // Check for aggregate functions
        if let Some(agg) = try_parse_aggregate(&tokens, &mut pos) {
            aggregates.push(agg);
        } else {
            columns.push(col);
        }
        
        // Skip commas
        if pos < tokens.len() && tokens[pos] == "," {
            pos += 1;
        } else {
            pos += 1;
        }
    }
    
    // Expect FROM
    if pos >= tokens.len() || tokens[pos].to_uppercase() != "FROM" {
        return Err(SdkError::InvalidQuery("Expected FROM keyword".to_string()));
    }
    pos += 1;
    
    // Table name
    if pos >= tokens.len() {
        return Err(SdkError::InvalidQuery("Expected table name after FROM".to_string()));
    }
    let table = &tokens[pos];
    pos += 1;
    
    // Get all data from table
    let engine_db = db.db()?;
    let mut data = engine_db.scan_json(table)?;
    
    // Parse WHERE clause
    if pos < tokens.len() && tokens[pos].to_uppercase() == "WHERE" {
        pos += 1;
        data = apply_where_filter(data, tokens, &mut pos);
    }
    
    // Handle aggregates
    if !aggregates.is_empty() {
        let mut result_row = serde_json::Map::new();
        for agg in &aggregates {
            let value = Aggregator::execute(agg, &data);
            let key = format!("{:?}", agg);
            result_row.insert(key, value);
        }
        return Ok(QueryResult {
            rows: vec![Value::Object(result_row)],
            columns: vec![],
            rows_affected: 0,
            execution_time_ms: 0.0,
        });
    }
    
    // Parse ORDER BY
    if pos < tokens.len() && tokens[pos].to_uppercase() == "ORDER" {
        pos += 1; // skip ORDER
        if pos < tokens.len() && tokens[pos].to_uppercase() == "BY" {
            pos += 1; // skip BY
        }
        if pos < tokens.len() {
            let col = tokens[pos].trim_end_matches(',').to_string();
            pos += 1;
            let dir = if pos < tokens.len() && tokens[pos].to_uppercase() == "DESC" {
                pos += 1;
                SortDirection::Desc
            } else {
                if pos < tokens.len() && tokens[pos].to_uppercase() == "ASC" {
                    pos += 1;
                }
                SortDirection::Asc
            };
            let order = OrderBy::new(&col, dir);
            Sorter::sort(&mut data, &order);
        }
    }
    
    // Parse LIMIT
    let mut limit: Option<usize> = None;
    if pos < tokens.len() && tokens[pos].to_uppercase() == "LIMIT" {
        pos += 1;
        if pos < tokens.len() {
            limit = tokens[pos].parse().ok();
            pos += 1;
        }
    }
    
    // Parse OFFSET
    let mut offset: usize = 0;
    if pos < tokens.len() && tokens[pos].to_uppercase() == "OFFSET" {
        pos += 1;
        if pos < tokens.len() {
            offset = tokens[pos].parse().unwrap_or(0);
            let _ = pos; // suppress unused warning
        }
    }
    
    // Apply offset and limit
    if offset > 0 {
        if offset >= data.len() {
            data.clear();
        } else {
            data = data.into_iter().skip(offset).collect();
        }
    }
    
    if let Some(lim) = limit {
        data.truncate(lim);
    }
    
    // Column projection (if not SELECT *)
    let is_select_all = columns.len() == 1 && columns[0] == "*";
    if !is_select_all && !columns.is_empty() {
        data = data.into_iter().map(|row| {
            if let Value::Object(map) = &row {
                let mut projected = serde_json::Map::new();
                for col in &columns {
                    let col_clean = col.trim_end_matches(',');
                    if let Some(val) = map.get(col_clean) {
                        projected.insert(col_clean.to_string(), val.clone());
                    }
                }
                Value::Object(projected)
            } else {
                row
            }
        }).collect();
    }
    
    Ok(QueryResult {
        rows: data,
        columns,
        rows_affected: 0,
        execution_time_ms: 0.0,
    })
}

/// Try to parse an aggregate function like COUNT(*), SUM(col), AVG(col)
fn try_parse_aggregate(tokens: &[String], pos: &mut usize) -> Option<AggregateFunc> {
    let upper = tokens[*pos].to_uppercase();
    let func_names = ["COUNT", "SUM", "AVG", "MIN", "MAX"];
    
    if !func_names.contains(&upper.as_str()) {
        return None;
    }
    
    // Check for opening paren
    if *pos + 1 >= tokens.len() || tokens[*pos + 1] != "(" {
        // Could be COUNT(*) already combined
        let combined = upper.clone();
        if combined.contains('(') {
            return AggregateFunc::parse(&combined);
        }
        return None;
    }
    
    // Reconstruct: FUNC(arg)
    let func_name = &tokens[*pos];
    *pos += 1; // skip func name
    *pos += 1; // skip (
    
    let arg = if *pos < tokens.len() {
        let a = tokens[*pos].clone();
        *pos += 1;
        a
    } else {
        return None;
    };
    
    // skip closing )
    if *pos < tokens.len() && tokens[*pos] == ")" {
        *pos += 1;
    }
    
    let full = format!("{}({})", func_name, arg);
    AggregateFunc::parse(&full)
}

/// Apply WHERE filtering to data
fn apply_where_filter(data: Vec<Value>, tokens: &[String], pos: &mut usize) -> Vec<Value> {
    // Parse simple conditions: col op value [AND/OR col op value ...]
    let mut conditions: Vec<(String, String, String)> = Vec::new(); // (col, op, val)
    let mut logical_ops: Vec<String> = Vec::new();
    
    while *pos < tokens.len() {
        let upper = tokens[*pos].to_uppercase();
        // Stop at ORDER, LIMIT, OFFSET, GROUP
        if ["ORDER", "LIMIT", "OFFSET", "GROUP"].contains(&upper.as_str()) {
            break;
        }
        
        // Parse: column operator value
        if *pos + 2 < tokens.len() {
            let col = tokens[*pos].clone();
            let op = tokens[*pos + 1].clone();
            let val = tokens[*pos + 2].clone();
            conditions.push((col, op, val));
            *pos += 3;
            
            // Check for AND/OR
            if *pos < tokens.len() {
                let next_upper = tokens[*pos].to_uppercase();
                if next_upper == "AND" || next_upper == "OR" {
                    logical_ops.push(next_upper);
                    *pos += 1;
                }
            }
        } else {
            break;
        }
    }
    
    if conditions.is_empty() {
        return data;
    }
    
    data.into_iter().filter(|row| {
        let mut result = evaluate_condition(row, &conditions[0]);
        
        for i in 0..logical_ops.len() {
            if i + 1 < conditions.len() {
                let next_result = evaluate_condition(row, &conditions[i + 1]);
                result = match logical_ops[i].as_str() {
                    "AND" => result && next_result,
                    "OR" => result || next_result,
                    _ => result,
                };
            }
        }
        
        result
    }).collect()
}

/// Evaluate a single WHERE condition against a row
fn evaluate_condition(row: &Value, condition: &(String, String, String)) -> bool {
    let (col, op, val) = condition;
    
    let field_val = match row.get(col) {
        Some(v) => v,
        None => return false,
    };
    
    // Strip quotes from value
    let clean_val = val.trim_matches('\'').trim_matches('"');
    
    match op.as_str() {
        "=" | "==" => {
            if let Some(n) = clean_val.parse::<f64>().ok() {
                field_val.as_f64().map_or(false, |fv| (fv - n).abs() < f64::EPSILON)
            } else {
                field_val.as_str().map_or(false, |s| s == clean_val)
                    || field_val.to_string().trim_matches('"') == clean_val
            }
        }
        "!=" | "<>" => {
            if let Some(n) = clean_val.parse::<f64>().ok() {
                field_val.as_f64().map_or(true, |fv| (fv - n).abs() >= f64::EPSILON)
            } else {
                field_val.as_str().map_or(true, |s| s != clean_val)
            }
        }
        ">" => compare_values(field_val, clean_val) > 0,
        "<" => compare_values(field_val, clean_val) < 0,
        ">=" => compare_values(field_val, clean_val) >= 0,
        "<=" => compare_values(field_val, clean_val) <= 0,
        _ => false,
    }
}

/// Compare a JSON value with a string value
fn compare_values(field: &Value, val: &str) -> i32 {
    if let Ok(n) = val.parse::<f64>() {
        if let Some(fv) = field.as_f64() {
            return if fv > n { 1 } else if fv < n { -1 } else { 0 };
        }
    }
    if let Some(s) = field.as_str() {
        return s.cmp(val) as i32;
    }
    0
}

/// Execute INSERT query
fn execute_insert(db: &mut OverDriveDB, tokens: &[String], raw_sql: &str) -> SdkResult<QueryResult> {
    // INSERT INTO <table> {json}
    if tokens.len() < 3 || tokens[1].to_uppercase() != "INTO" {
        return Err(SdkError::InvalidQuery("Expected INSERT INTO <table> {json}".to_string()));
    }
    
    let table = &tokens[2];
    
    // Find the JSON part — look for { in the raw sql
    let json_str = if let Some(brace_pos) = raw_sql.find('{') {
        &raw_sql[brace_pos..]
    } else {
        return Err(SdkError::InvalidQuery("Expected JSON object after table name".to_string()));
    };
    
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| SdkError::InvalidQuery(format!("Invalid JSON: {}", e)))?;
    
    let id = db.insert(table, &value)?;
    
    Ok(QueryResult {
        rows: vec![serde_json::json!({"_id": id})],
        columns: vec!["_id".to_string()],
        rows_affected: 1,
        execution_time_ms: 0.0,
    })
}

/// Execute UPDATE query
fn execute_update(db: &mut OverDriveDB, tokens: &[String], raw_sql: &str) -> SdkResult<QueryResult> {
    // UPDATE <table> SET {json} [WHERE ...]
    if tokens.len() < 3 {
        return Err(SdkError::InvalidQuery("Expected UPDATE <table> SET {json}".to_string()));
    }
    
    let table = tokens[1].clone();
    
    // Find SET keyword
    let set_pos = tokens.iter().position(|t| t.to_uppercase() == "SET")
        .ok_or_else(|| SdkError::InvalidQuery("Expected SET keyword".to_string()))?;
    
    // Find JSON object after SET
    let json_str = if let Some(brace_pos) = raw_sql[..].find('{') {
        // Find the matching closing brace
        let sub = &raw_sql[brace_pos..];
        let mut depth = 0;
        let mut end = 0;
        for (i, c) in sub.chars().enumerate() {
            if c == '{' { depth += 1; }
            if c == '}' { depth -= 1; }
            if depth == 0 { end = i + 1; break; }
        }
        &raw_sql[brace_pos..brace_pos + end]
    } else {
        return Err(SdkError::InvalidQuery("Expected {updates} after SET".to_string()));
    };
    
    let updates: Value = serde_json::from_str(json_str)
        .map_err(|e| SdkError::InvalidQuery(format!("Invalid JSON: {}", e)))?;
    
    // Get all data to find matching rows
    let engine_db = db.db()?;
    let all_data = engine_db.scan_json(&table)?;
    
    // Parse WHERE if present
    let mut where_pos = set_pos + 1;
    // Skip past the JSON token
    while where_pos < tokens.len() && tokens[where_pos].to_uppercase() != "WHERE" {
        where_pos += 1;
    }
    
    let matched_ids: Vec<String>;
    if where_pos < tokens.len() && tokens[where_pos].to_uppercase() == "WHERE" {
        where_pos += 1;
        let filtered = apply_where_filter(all_data, tokens, &mut where_pos);
        matched_ids = filtered.iter()
            .filter_map(|r| r.get("_id").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .collect();
    } else {
        matched_ids = all_data.iter()
            .filter_map(|r| r.get("_id").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .collect();
    }
    
    let mut affected = 0;
    for id in &matched_ids {
        if db.update(&table, id, &updates)? {
            affected += 1;
        }
    }
    
    Ok(QueryResult {
        rows: Vec::new(),
        columns: Vec::new(),
        rows_affected: affected,
        execution_time_ms: 0.0,
    })
}

/// Execute DELETE query
fn execute_delete(db: &mut OverDriveDB, tokens: &[String]) -> SdkResult<QueryResult> {
    // DELETE FROM <table> [WHERE ...]
    if tokens.len() < 3 || tokens[1].to_uppercase() != "FROM" {
        return Err(SdkError::InvalidQuery("Expected DELETE FROM <table>".to_string()));
    }
    
    let table = tokens[2].clone();
    
    // Get all data
    let engine_db = db.db()?;
    let all_data = engine_db.scan_json(&table)?;
    
    // Parse WHERE clause
    let mut pos = 3;
    let matched: Vec<Value>;
    if pos < tokens.len() && tokens[pos].to_uppercase() == "WHERE" {
        pos += 1;
        matched = apply_where_filter(all_data, tokens, &mut pos);
    } else {
        matched = all_data;
    }
    
    let ids: Vec<String> = matched.iter()
        .filter_map(|r| r.get("_id").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .collect();
    
    let mut affected = 0;
    for id in &ids {
        if db.delete(&table, id)? {
            affected += 1;
        }
    }
    
    Ok(QueryResult {
        rows: Vec::new(),
        columns: Vec::new(),
        rows_affected: affected,
        execution_time_ms: 0.0,
    })
}

/// Execute CREATE TABLE
fn execute_create(db: &mut OverDriveDB, tokens: &[String]) -> SdkResult<QueryResult> {
    if tokens.len() < 3 {
        return Err(SdkError::InvalidQuery("Expected CREATE TABLE <name>".to_string()));
    }
    let kw = tokens[1].to_uppercase();
    if kw != "TABLE" && kw != "TB" {
        return Err(SdkError::InvalidQuery("Expected CREATE TABLE".to_string()));
    }
    let name = &tokens[2];
    db.create_table(name)?;
    
    Ok(QueryResult {
        rows: Vec::new(),
        columns: Vec::new(),
        rows_affected: 0,
        execution_time_ms: 0.0,
    })
}

/// Execute DROP TABLE
fn execute_drop(db: &mut OverDriveDB, tokens: &[String]) -> SdkResult<QueryResult> {
    if tokens.len() < 3 {
        return Err(SdkError::InvalidQuery("Expected DROP TABLE <name>".to_string()));
    }
    let kw = tokens[1].to_uppercase();
    if kw != "TABLE" && kw != "TB" {
        return Err(SdkError::InvalidQuery("Expected DROP TABLE".to_string()));
    }
    let name = &tokens[2];
    db.drop_table(name)?;
    
    Ok(QueryResult {
        rows: Vec::new(),
        columns: Vec::new(),
        rows_affected: 0,
        execution_time_ms: 0.0,
    })
}

/// Execute SHOW TABLES
fn execute_show(db: &mut OverDriveDB, tokens: &[String]) -> SdkResult<QueryResult> {
    if tokens.len() < 2 {
        return Err(SdkError::InvalidQuery("Expected SHOW TABLES".to_string()));
    }
    let kw = tokens[1].to_uppercase();
    if kw != "TABLES" && kw != "TABLE" && kw != "TB" {
        return Err(SdkError::InvalidQuery("Expected SHOW TABLES".to_string()));
    }
    
    let tables = db.list_tables()?;
    let rows: Vec<Value> = tables.iter()
        .map(|t| serde_json::json!({"table_name": t}))
        .collect();
    
    Ok(QueryResult {
        rows,
        columns: vec!["table_name".to_string()],
        rows_affected: 0,
        execution_time_ms: 0.0,
    })
}
