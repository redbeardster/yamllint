use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub rules: RuleConfig,
    pub format: FormatConfig,
    pub exclude: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RuleConfig {
    pub indentation: IndentationRule,
    pub line_length: LineLengthRule,
    pub trailing_spaces: SeverityRule,
    pub empty_lines: EmptyLinesRule,
    pub required_fields: RequiredFieldsRule,
    pub value_types: ValueTypesRule,
    pub duplicates: SeverityRule,
    pub quotes: QuotesRule,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndentationRule {
    pub spaces: usize,
    pub check_multi_line_strings: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LineLengthRule {
    pub max: usize,
    pub allow_non_breakable_words: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmptyLinesRule {
    pub max_start: usize,
    pub max_end: usize,
    pub max_consecutive: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequiredFieldsRule {
    pub paths: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValueTypesRule {
    pub strict_numbers: bool,
    pub check_bool_values: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuotesRule {
    pub prefer_double: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FormatConfig {
    pub auto_fix: bool,
    pub backup_files: bool,
    pub indent_sequence: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeverityRule {
    pub level: Severity,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Severity {
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "off")]
    Off,
}

impl Default for Config {
    fn default() -> Self {
        let mut required_fields = HashMap::new();
        required_fields.insert(
            "**/k8s/*.yaml".to_string(),
            vec!["apiVersion".to_string(), "kind".to_string(), "metadata.name".to_string()]
        );

        Config {
            rules: RuleConfig {
                indentation: IndentationRule {
                    spaces: 2,
                    check_multi_line_strings: true,
                },
                line_length: LineLengthRule {
                    max: 120,
                    allow_non_breakable_words: true,
                },
                trailing_spaces: SeverityRule {
                    level: Severity::Error,
                },
                empty_lines: EmptyLinesRule {
                    max_start: 0,
                    max_end: 1,
                    max_consecutive: 2,
                },
                required_fields: RequiredFieldsRule {
                    paths: required_fields,
                },
                value_types: ValueTypesRule {
                    strict_numbers: true,
                    check_bool_values: true,
                },
                duplicates: SeverityRule {
                    level: Severity::Error,
                },
                quotes: QuotesRule {
                    prefer_double: false,
                },
            },
            format: FormatConfig {
                auto_fix: false,
                backup_files: true,
                indent_sequence: true,
            },
            exclude: vec![
                "**/node_modules/".to_string(),
                "**/.git/".to_string(),
                "**/vendor/".to_string(),
            ],
        }
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn should_exclude(&self, path: &str) -> bool {
        // Простая реализация исключений (можно улучшить с помощью glob)
        for pattern in &self.exclude {
            if path.contains(pattern.trim_end_matches('/').trim_end_matches('*')) {
                return true;
            }
        }
        false
    }
}
