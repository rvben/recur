use serde_json::Value;

/// Filter fields with an explicit ok status for the envelope.
pub fn filter_fields_with_status(value: &Value, fields_str: &str, ok: bool) -> Value {
    let fields: Vec<&str> = fields_str.split(',').map(str::trim).collect();

    if let Some(arr) = value.as_array() {
        let filtered: Vec<Value> = arr.iter().map(|item| pick(item, &fields)).collect();
        serde_json::json!({ "ok": ok, "data": filtered })
    } else if value.is_object() {
        let picked = pick(value, &fields);
        serde_json::json!({ "ok": ok, "data": picked })
    } else {
        serde_json::json!({ "ok": ok, "data": value })
    }
}

/// Pick specified fields from a JSON object.
pub fn pick(value: &Value, fields: &[&str]) -> Value {
    let mut result = serde_json::Map::new();

    for field in fields {
        let parts: Vec<&str> = field.splitn(2, '.').collect();
        let key = parts[0];

        if let Some(val) = value.get(key) {
            if parts.len() == 2 {
                if let Some(arr) = val.as_array() {
                    let nested: Vec<Value> =
                        arr.iter().map(|item| pick(item, &[parts[1]])).collect();
                    result.insert(key.to_string(), Value::Array(nested));
                } else {
                    result.insert(key.to_string(), pick(val, &[parts[1]]));
                }
            } else {
                result.insert(key.to_string(), val.clone());
            }
        }
    }

    Value::Object(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn pick_single_field() {
        let value = json!({"name": "test", "age": 30, "city": "Amsterdam"});
        let result = pick(&value, &["name"]);
        assert_eq!(result, json!({"name": "test"}));
    }

    #[test]
    fn pick_multiple_fields() {
        let value = json!({"name": "test", "age": 30, "city": "Amsterdam"});
        let result = pick(&value, &["name", "city"]);
        assert_eq!(result, json!({"name": "test", "city": "Amsterdam"}));
    }

    #[test]
    fn pick_missing_field_ignored() {
        let value = json!({"name": "test"});
        let result = pick(&value, &["name", "nonexistent"]);
        assert_eq!(result, json!({"name": "test"}));
    }

    #[test]
    fn pick_nested_dot_path() {
        let value =
            json!({"user": "root", "details": {"email": "root@localhost", "shell": "/bin/bash"}});
        let result = pick(&value, &["details.email"]);
        assert_eq!(result, json!({"details": {"email": "root@localhost"}}));
    }

    #[test]
    fn pick_nested_array_dot_path() {
        let value = json!({
            "issues": [
                {"severity": "Error", "message": "not found"},
                {"severity": "Warning", "message": "no redirect"},
            ]
        });
        let result = pick(&value, &["issues.severity"]);
        let issues = result["issues"].as_array().unwrap();
        assert_eq!(issues[0], json!({"severity": "Error"}));
        assert_eq!(issues[1], json!({"severity": "Warning"}));
    }

    #[test]
    fn filter_fields_array_input() {
        let value = json!([
            {"user": "root", "schedule": "* * * * *", "command": "echo hi"},
            {"user": "test", "schedule": "0 * * * *", "command": "echo bye"},
        ]);
        let result = filter_fields_with_status(&value, "user,schedule", true);
        let data = result["data"].as_array().unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], json!({"user": "root", "schedule": "* * * * *"}));
        assert!(!data[0].as_object().unwrap().contains_key("command"));
    }

    #[test]
    fn filter_fields_object_input() {
        let value = json!({"expression": "*/5 * * * *", "description": "every 5 minutes"});
        let result = filter_fields_with_status(&value, "description", true);
        assert_eq!(result["data"], json!({"description": "every 5 minutes"}));
        assert_eq!(result["ok"], true);
    }

    #[test]
    fn filter_fields_scalar_input() {
        let value = json!(42);
        let result = filter_fields_with_status(&value, "anything", true);
        assert_eq!(result["data"], json!(42));
        assert_eq!(result["ok"], true);
    }

    #[test]
    fn pick_all_fields_missing() {
        let value = json!({"a": 1});
        let result = pick(&value, &["x", "y", "z"]);
        assert_eq!(result, json!({}));
    }

    #[test]
    fn filter_fields_whitespace_trimmed() {
        let value = json!({"name": "test", "age": 30});
        let result = filter_fields_with_status(&value, " name , age ", true);
        assert_eq!(result["data"], json!({"name": "test", "age": 30}));
    }

    #[test]
    fn filter_fields_with_ok_false() {
        let value = json!({"issues": [{"severity": "Error"}]});
        let result = filter_fields_with_status(&value, "issues", false);
        assert_eq!(result["ok"], false);
        assert!(result["data"]["issues"].is_array());
    }

    #[test]
    fn filter_fields_with_ok_true() {
        let value = json!({"issues": []});
        let result = filter_fields_with_status(&value, "issues", true);
        assert_eq!(result["ok"], true);
    }
}
