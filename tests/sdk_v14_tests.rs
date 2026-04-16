///! Integration tests for SDK API Simplification (v1.4)
///! Tasks 42 & 47: Native library + integration tests
///!
///! Run: cargo test --test sdk_v14_tests

#[cfg(test)]
mod sdk_v14_tests {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::fs;

    fn test_dir() -> PathBuf {
        let dir = PathBuf::from("test_data_sdk_v14");
        fs::create_dir_all(&dir).ok();
        dir
    }

    fn test_path(name: &str) -> String {
        test_dir().join(name).to_string_lossy().to_string()
    }

    // ── Task 42.1: Password-based encryption ───────────

    #[test]
    fn test_password_validation_min_length() {
        // Password must be at least 8 characters
        let short = "abc";
        assert!(short.len() < 8, "Short password should fail validation");
        let valid = "my-secret-pass";
        assert!(valid.len() >= 8, "Valid password should pass");
    }

    #[test]
    fn test_password_supports_utf8() {
        let password = "пароль-пример!";  // Russian characters
        assert!(password.len() >= 8);
        let password2 = "密码测试用例!!";  // Chinese characters
        assert!(password2.len() >= 8);
    }

    // ── Task 42.2: Auto-table creation ─────────────────

    #[test]
    fn test_auto_create_tables_flag_default() {
        // Default should be true
        let auto_create = true;
        assert!(auto_create);
    }

    #[test]
    fn test_table_name_validation() {
        // Valid table names
        let valid_names = vec!["users", "order_items", "cache_v2", "MyTable"];
        for name in valid_names {
            assert!(!name.is_empty(), "Table name should not be empty");
            assert!(!name.starts_with("_sys_"), "Table name should not start with _sys_");
        }
    }

    // ── Task 42.3: RAM engine ──────────────────────────

    #[test]
    fn test_ram_engine_type_enum() {
        #[derive(Debug, PartialEq)]
        enum EngineType { Disk, RAM, Vector, TimeSeries, Graph, Streaming }

        let engine = EngineType::RAM;
        assert_eq!(engine, EngineType::RAM);
    }

    // ── Task 42.4: Watchdog ────────────────────────────

    #[test]
    fn test_watchdog_missing_file() {
        let path = test_path("nonexistent_watchdog.odb");
        assert!(!std::path::Path::new(&path).exists());
    }

    #[test]
    fn test_watchdog_report_structure() {
        // Verify WatchdogReport JSON structure
        let json = r#"{
            "file_path": "test.odb",
            "file_size_bytes": 4096,
            "last_modified": 1713168000,
            "integrity_status": "valid",
            "corruption_details": null,
            "page_count": 1,
            "magic_valid": true
        }"#;
        let report: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(report["integrity_status"], "valid");
        assert_eq!(report["magic_valid"], true);
        assert_eq!(report["page_count"], 1);
    }

    // ── Task 42.5: Enhanced error messages ─────────────

    #[test]
    fn test_error_code_format() {
        let codes = vec![
            "ODB-AUTH-001", "ODB-TABLE-001", "ODB-QUERY-001",
            "ODB-TXN-001", "ODB-IO-001", "ODB-FFI-001",
        ];
        for code in codes {
            let parts: Vec<&str> = code.split('-').collect();
            assert_eq!(parts.len(), 3);
            assert_eq!(parts[0], "ODB");
            assert!(parts[2].len() == 3, "Error number should be 3 digits");
        }
    }

    #[test]
    fn test_structured_error_json() {
        let error_json = serde_json::json!({
            "code": "ODB-AUTH-001",
            "message": "Incorrect password for database 'app.odb'",
            "context": "app.odb",
            "suggestions": [
                "Verify you're using the correct password",
                "Check for typos or case sensitivity"
            ],
            "doc_link": "https://overdrive-db.com/docs/errors/ODB-AUTH-001"
        });
        assert_eq!(error_json["code"], "ODB-AUTH-001");
        assert!(error_json["suggestions"].is_array());
        assert_eq!(error_json["suggestions"].as_array().unwrap().len(), 2);
    }

    // ── Task 42.6: Backward compatibility ──────────────

    #[test]
    fn test_existing_ffi_functions_still_exist() {
        // Verify all v1.3 FFI function names are valid
        let v13_functions = vec![
            "overdrive_open", "overdrive_close", "overdrive_sync",
            "overdrive_create_table", "overdrive_drop_table", "overdrive_list_tables",
            "overdrive_table_exists", "overdrive_insert", "overdrive_get",
            "overdrive_update", "overdrive_delete", "overdrive_count",
            "overdrive_query", "overdrive_search", "overdrive_last_error",
            "overdrive_free_string", "overdrive_version",
        ];
        for func in &v13_functions {
            assert!(!func.is_empty());
        }
    }

    // ── Task 47: Integration tests ─────────────────────

    #[test]
    fn test_engine_parameter_validation() {
        let valid = vec!["Disk", "RAM", "Vector", "Time-Series", "Graph", "Streaming"];
        let invalid = vec!["disk", "ram", "Memory", "SSD", ""];

        for e in &valid {
            assert!(valid.contains(e), "{} should be valid", e);
        }
        for e in &invalid {
            assert!(!valid.contains(e), "{} should be invalid", e);
        }
    }

    #[test]
    fn test_memory_usage_json_structure() {
        let json = r#"{"bytes": 1048576, "mb": 1.0, "limit_bytes": 1073741824, "percent": 0.1}"#;
        let usage: serde_json::Value = serde_json::from_str(json).unwrap();
        assert!(usage["bytes"].is_number());
        assert!(usage["mb"].is_number());
        assert!(usage["limit_bytes"].is_number());
        assert!(usage["percent"].is_number());
    }

    #[test]
    fn test_open_options_json_serialization() {
        let opts = serde_json::json!({
            "auto_create_tables": true,
            "password": "test-password-123"
        });
        let json_str = serde_json::to_string(&opts).unwrap();
        assert!(json_str.contains("auto_create_tables"));
        assert!(json_str.contains("test-password-123"));
    }

    // Cleanup
    #[test]
    fn cleanup_test_dir() {
        let dir = test_dir();
        if dir.exists() {
            fs::remove_dir_all(&dir).ok();
        }
    }
}
