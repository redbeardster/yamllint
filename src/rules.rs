use crate::config::{Config, Severity};
use regex::Regex;
use serde_yaml::{Value, Mapping};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct LintResult {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
    pub rule: String,
    pub message: String,
    pub snippet: String,
}

impl LintResult {
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }

    pub fn is_warning(&self) -> bool {
        self.severity == Severity::Warning
    }
}

pub struct RuleChecker {
    config: Config,
}

impl RuleChecker {
    pub fn new(config: Config) -> Self {
        RuleChecker { config }
    }

    pub fn check_file(&self, content: &str, file_path: &str) -> Vec<LintResult> {
        let mut results = vec![];

        // Проверка синтаксиса
        if let Err(e) = serde_yaml::from_str::<Value>(content) {
            results.push(LintResult {
                file: file_path.to_string(),
                line: 1,
                column: 1,
                severity: Severity::Error,
                rule: "syntax".to_string(),
                message: format!("Syntax error: {}", e),
                snippet: content.lines().next().unwrap_or("").to_string(),
            });
            return results;
        }

        // Базовые проверки на уровне текста
        results.extend(self.check_indentation(content, file_path));
        results.extend(self.check_trailing_spaces(content, file_path));
        results.extend(self.check_line_length(content, file_path));
        results.extend(self.check_empty_lines(content, file_path));

        // Семантические проверки на уровне AST
        if let Ok(value) = serde_yaml::from_str::<Value>(content) {
            results.extend(self.check_required_fields(&value, file_path));
            results.extend(self.check_value_types(&value, file_path));
            results.extend(self.check_duplicates(&value, file_path));
        }

        results
    }

    fn check_indentation(&self, content: &str, file_path: &str) -> Vec<LintResult> {
        let mut results = vec![];
        let expected_spaces = self.config.rules.indentation.spaces;
        let _space_str = " ".repeat(expected_spaces); // Сохраняем для возможного использования

        for (i, line) in content.lines().enumerate() {
            let line_num = i + 1;

            // Пропускаем пустые строки
            if line.trim().is_empty() {
                continue;
            }

            // Проверяем отступы
            if line.starts_with(' ') {
                let leading_spaces = line.len() - line.trim_start().len();
                if leading_spaces % expected_spaces != 0 {
                    results.push(LintResult {
                        file: file_path.to_string(),
                        line: line_num,
                        column: 1,
                        severity: Severity::Error,
                        rule: "indentation".to_string(),
                        message: format!("Indentation should be multiples of {} spaces", expected_spaces),
                        snippet: line.to_string(),
                    });
                }
            }
        }

        results
    }

    fn check_trailing_spaces(&self, content: &str, file_path: &str) -> Vec<LintResult> {
        let mut results = vec![];

        for (i, line) in content.lines().enumerate() {
            let line_num = i + 1;

            if line.ends_with(' ') || line.ends_with('\t') {
                results.push(LintResult {
                    file: file_path.to_string(),
                    line: line_num,
                    column: line.len(),
                    severity: self.config.rules.trailing_spaces.level.clone(),
                    rule: "trailing-spaces".to_string(),
                    message: "Trailing spaces are not allowed".to_string(),
                    snippet: line.to_string(),
                });
            }
        }

        results
    }

    fn check_line_length(&self, content: &str, file_path: &str) -> Vec<LintResult> {
        let mut results = vec![];
        let max_length = self.config.rules.line_length.max;

        for (i, line) in content.lines().enumerate() {
            let line_num = i + 1;

            if line.len() > max_length {
                results.push(LintResult {
                    file: file_path.to_string(),
                    line: line_num,
                    column: max_length + 1,
                    severity: Severity::Warning,
                    rule: "line-length".to_string(),
                    message: format!("Line too long ({} > {})", line.len(), max_length),
                    snippet: line.to_string(),
                });
            }
        }

        results
    }

    fn check_empty_lines(&self, content: &str, file_path: &str) -> Vec<LintResult> {
        let mut results = vec![];
        let lines: Vec<&str> = content.lines().collect();
        let mut consecutive_empty = 0;

        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;

            if line.trim().is_empty() {
                consecutive_empty += 1;

                if consecutive_empty > self.config.rules.empty_lines.max_consecutive {
                    results.push(LintResult {
                        file: file_path.to_string(),
                        line: line_num,
                        column: 1,
                        severity: Severity::Warning,
                        rule: "empty-lines".to_string(),
                        message: format!("Too many consecutive empty lines ({})", consecutive_empty),
                        snippet: "".to_string(),
                    });
                }
            } else {
                consecutive_empty = 0;
            }
        }

        // Проверка пустых строк в начале файла
        let mut start_empty = 0;
        for _line in lines.iter().take_while(|l| l.trim().is_empty()) {
            start_empty += 1;
        }

        if start_empty > self.config.rules.empty_lines.max_start {
            results.push(LintResult {
                file: file_path.to_string(),
                line: 1,
                column: 1,
                severity: Severity::Warning,
                rule: "empty-lines".to_string(),
                message: format!("Too many empty lines at start of file ({})", start_empty),
                snippet: "".to_string(),
            });
        }

        results
    }

    fn check_required_fields(&self, value: &Value, file_path: &str) -> Vec<LintResult> {
        let mut results = vec![];

        for (pattern, required_fields) in &self.config.rules.required_fields.paths {
            // Простая проверка паттерна (можно заменить на glob)
            if file_path.contains(pattern.trim_matches('*').trim_matches('/')) {
                self.check_required_in_value(value, required_fields, file_path, &mut results);
            }
        }

        results
    }

    fn check_required_in_value(&self, value: &Value, required_fields: &[String],
                               file_path: &str, results: &mut Vec<LintResult>) {
        if let Value::Mapping(mapping) = value {
            for field in required_fields {
                let parts: Vec<&str> = field.split('.').collect();
                self.check_nested_field(mapping, &parts, file_path, results);
            }
        }
    }

    fn check_nested_field(&self, mapping: &Mapping, parts: &[&str],
                          file_path: &str, results: &mut Vec<LintResult>) {
        if parts.is_empty() {
            return;
        }

        let key = parts[0];
        let key_value = Value::String(key.to_string());

        if !mapping.contains_key(&key_value) {
            results.push(LintResult {
                file: file_path.to_string(),
                line: 1,
                column: 1,
                severity: Severity::Error,
                rule: "required-fields".to_string(),
                message: format!("Missing required field: {}", key),
                snippet: "".to_string(),
            });
            return;
        }

        if parts.len() > 1 {
            if let Some(sub_value) = mapping.get(&key_value) {
                if let Value::Mapping(sub_mapping) = sub_value {
                    self.check_nested_field(sub_mapping, &parts[1..], file_path, results);
                }
            }
        }
    }

    fn check_value_types(&self, value: &Value, file_path: &str) -> Vec<LintResult> {
        let mut results = vec![];
        self.visit_value(value, file_path, &mut results);
        results
    }

    fn visit_value(&self, value: &Value, file_path: &str, results: &mut Vec<LintResult>) {
        match value {
            Value::String(s) => {
                // Проверка на boolean строки
                if self.config.rules.value_types.check_bool_values {
                    let lower = s.to_lowercase();
                    if lower == "true" || lower == "false" || lower == "yes" || lower == "no" {
                        results.push(LintResult {
                            file: file_path.to_string(),
                            line: 1,
                            column: 1,
                            severity: Severity::Warning,
                            rule: "value-types".to_string(),
                            message: format!("Boolean-like string: '{}'. Consider using boolean type.", s),
                            snippet: s.to_string(),
                        });
                    }
                }

                // Проверка на числовые строки
                if self.config.rules.value_types.strict_numbers {
                    if s.parse::<i64>().is_ok() || s.parse::<f64>().is_ok() {
                        results.push(LintResult {
                            file: file_path.to_string(),
                            line: 1,
                            column: 1,
                            severity: Severity::Warning,
                            rule: "value-types".to_string(),
                            message: format!("Number-like string: '{}'. Consider using number type.", s),
                            snippet: s.to_string(),
                        });
                    }
                }
            }

            Value::Mapping(mapping) => {
                for (_, v) in mapping {
                    self.visit_value(v, file_path, results);
                }
            }

            Value::Sequence(seq) => {
                for v in seq {
                    self.visit_value(v, file_path, results);
                }
            }

            _ => {}
        }
    }

    fn check_duplicates(&self, value: &Value, file_path: &str) -> Vec<LintResult> {
        let mut results = vec![];

        if let Value::Mapping(mapping) = value {
            let mut seen_keys = HashSet::new();

            for (k, _) in mapping {
                if let Value::String(s) = k {
                    if !seen_keys.insert(s) {
                        results.push(LintResult {
                            file: file_path.to_string(),
                            line: 1,
                            column: 1,
                            severity: self.config.rules.duplicates.level.clone(),
                            rule: "duplicates".to_string(),
                            message: format!("Duplicate key: '{}'", s),
                            snippet: s.to_string(),
                        });
                    }
                }
            }
        }

        results
    }
}
