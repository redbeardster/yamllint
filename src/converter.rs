use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_yaml;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct ConversionResult {
    pub input_file: String,
    pub output_file: String,
    pub success: bool,
    pub error: Option<String>,
    pub size_before: u64,
    pub size_after: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversionExport {
    pub timestamp: String,
    pub summary: ConversionSummary,
    pub conversions: Vec<SingleConversion>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversionSummary {
    pub total_files: usize,
    pub successful: usize,
    pub failed: usize,
    pub total_size_before: u64,
    pub total_size_after: u64,
    pub average_size_change: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SingleConversion {
    pub input_file: String,
    pub output_file: String,
    pub success: bool,
    pub error: Option<String>,
    pub size_before: u64,
    pub size_after: u64,
    pub size_change_percent: f64,
}

pub struct YamlConverter;

impl YamlConverter {
    /// Конвертировать файл YAML в другой формат
    pub fn convert_file(
        input_path: &str,
        target_format: &crate::cli::TargetFormat,
        output_path: Option<&str>,
        pretty: bool,
    ) -> Result<ConversionResult> {
        let input_path_obj = Path::new(input_path);
        let content = fs::read_to_string(input_path_obj)?;
        let size_before = content.len() as u64;

        // Парсим YAML
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)?;

        // Определяем путь для выходного файла
        let output_path = match output_path {
            Some(path) => PathBuf::from(path),
            None => {
                let mut output = input_path_obj.to_path_buf();
                output.set_extension(match target_format {
                    crate::cli::TargetFormat::Json => "json",
                    crate::cli::TargetFormat::Yaml => "yaml",
                    crate::cli::TargetFormat::Toml => "toml",
                    crate::cli::TargetFormat::Xml => "xml",
                    crate::cli::TargetFormat::Hcl => "hcl",
                    crate::cli::TargetFormat::Ini => "ini",
                });
                output
            }
        };

        // Конвертируем в целевой формат
        let output_content = match target_format {
            crate::cli::TargetFormat::Json => {
                if pretty {
                    serde_json::to_string_pretty(&yaml_value)?
                } else {
                    serde_json::to_string(&yaml_value)?
                }
            }

            crate::cli::TargetFormat::Yaml => {
                if pretty {
                    serde_yaml::to_string(&yaml_value)?
                } else {
                    content
                }
            }

            crate::cli::TargetFormat::Toml => {
                Self::yaml_to_toml(&yaml_value, pretty)?
            }

            crate::cli::TargetFormat::Xml => {
                Self::yaml_to_xml(&yaml_value, pretty)?
            }

            crate::cli::TargetFormat::Hcl => {
                Self::yaml_to_hcl(&yaml_value, pretty)?
            }

            crate::cli::TargetFormat::Ini => {
                Self::yaml_to_ini(&yaml_value, pretty)?
            }
        };

        let size_after = output_content.len() as u64;

        // Сохраняем результат
        fs::write(&output_path, output_content)?;

        Ok(ConversionResult {
            input_file: input_path.to_string(),
            output_file: output_path.to_string_lossy().to_string(),
            success: true,
            error: None,
            size_before,
            size_after,
        })
    }

    /// Конвертировать директорию с YAML файлами
    pub fn convert_directory(
        input_dir: &str,
        target_format: &crate::cli::TargetFormat,
        output_dir: Option<&str>,
        preserve_structure: bool,
        pretty: bool,
    ) -> Result<Vec<ConversionResult>> {
        let mut results = vec![];
        let output_dir = output_dir.unwrap_or(input_dir);

        for entry in WalkDir::new(input_dir) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && Self::is_yaml_file(path) {
                let relative_path = if preserve_structure {
                    path.strip_prefix(input_dir)?.to_path_buf()
                } else {
                    PathBuf::from(path.file_name().unwrap())
                };

                let output_path = Path::new(output_dir).join(relative_path);

                // Создаем директорию для выходного файла если нужно
                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                let result = Self::convert_file(
                    path.to_str().unwrap(),
                    target_format,
                    Some(output_path.to_str().unwrap()),
                    pretty,
                )?;

                results.push(result);
            }
        }

        Ok(results)
    }

    /// Создать структуру для экспорта результатов конвертации
    pub fn create_export_data(results: &[ConversionResult]) -> ConversionExport {
        let total_files = results.len();
        let successful = results.iter().filter(|r| r.success).count();
        let failed = total_files - successful;

        let total_size_before: u64 = results.iter().map(|r| r.size_before).sum();
        let total_size_after: u64 = results.iter().map(|r| r.size_after).sum();

        let average_size_change = if total_size_before > 0 && successful > 0 {
            let successful_results: Vec<&ConversionResult> = results.iter().filter(|r| r.success).collect();
            let total_change: f64 = successful_results.iter()
                .map(|r| {
                    if r.size_before > 0 {
                        (r.size_after as f64 - r.size_before as f64) / r.size_before as f64 * 100.0
                    } else {
                        0.0
                    }
                })
                .sum();
            total_change / successful as f64
        } else {
            0.0
        };

        ConversionExport {
            timestamp: Utc::now().to_rfc3339(),
            summary: ConversionSummary {
                total_files,
                successful,
                failed,
                total_size_before,
                total_size_after,
                average_size_change,
            },
            conversions: results
                .iter()
                .map(|result| SingleConversion {
                    input_file: result.input_file.clone(),
                    output_file: result.output_file.clone(),
                    success: result.success,
                    error: result.error.clone(),
                    size_before: result.size_before,
                    size_after: result.size_after,
                    size_change_percent: if result.size_before > 0 && result.success {
                        (result.size_after as f64 - result.size_before as f64) / result.size_before as f64 * 100.0
                    } else {
                        0.0
                    },
                })
                .collect(),
        }
    }

    /// Конвертировать YAML в TOML
    fn yaml_to_toml(value: &serde_yaml::Value, _pretty: bool) -> Result<String> {
        // Используем serde для конвертации через промежуточный JSON
        let json_value: serde_json::Value = serde_yaml::from_value(value.clone())?;

        // Простая реализация - для сложных случаев нужно использовать toml crate
        Ok(Self::json_to_toml_string(&json_value, 0))
    }

    fn json_to_toml_string(value: &serde_json::Value, indent: usize) -> String {
        match value {
            serde_json::Value::Object(obj) => {
                let mut result = String::new();
                for (i, (key, val)) in obj.iter().enumerate() {
                    if i > 0 {
                        result.push('\n');
                    }
                    result.push_str(&" ".repeat(indent));

                    // Проверяем, нужен ли для значения отдельный блок
                    match val {
                        serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                            result.push_str(&format!("[{}]\n", key));
                            result.push_str(&Self::json_to_toml_string(val, indent + 2));
                        }
                        _ => {
                            result.push_str(&format!("{} = {}", key, Self::json_value_to_string(val)));
                        }
                    }
                }
                result
            }
            serde_json::Value::Array(arr) => {
                let mut result = String::new();
                result.push_str("[\n");
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        result.push_str(",\n");
                    }
                    result.push_str(&" ".repeat(indent + 2));
                    result.push_str(&Self::json_value_to_string(val));
                }
                result.push_str(&format!("\n{}]", " ".repeat(indent)));
                result
            }
            _ => Self::json_value_to_string(value),
        }
    }

    fn json_value_to_string(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => "null".to_string(),
            _ => "".to_string(),
        }
    }

    /// Конвертировать YAML в XML
    fn yaml_to_xml(value: &serde_yaml::Value, _pretty: bool) -> Result<String> {
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        Self::yaml_value_to_xml(value, "root", 0, &mut xml);
        Ok(xml)
    }

    fn yaml_value_to_xml(value: &serde_yaml::Value, tag: &str, indent: usize, xml: &mut String) {
        let indent_str = " ".repeat(indent * 2);

        match value {
            serde_yaml::Value::Mapping(map) => {
                xml.push_str(&format!("{}<{}>\n", indent_str, tag));

                for (key, val) in map {
                    if let serde_yaml::Value::String(key_str) = key {
                        Self::yaml_value_to_xml(val, key_str, indent + 1, xml);
                    }
                }

                xml.push_str(&format!("{}</{}>\n", indent_str, tag));
            }
            serde_yaml::Value::Sequence(seq) => {
                xml.push_str(&format!("{}<{}>\n", indent_str, tag));

                for (i, val) in seq.iter().enumerate() {
                    Self::yaml_value_to_xml(val, &format!("item_{}", i), indent + 1, xml);
                }

                xml.push_str(&format!("{}</{}>\n", indent_str, tag));
            }
            serde_yaml::Value::String(s) => {
                xml.push_str(&format!("{}<{}>{}</{}>\n", indent_str, tag, s, tag));
            }
            serde_yaml::Value::Number(n) => {
                xml.push_str(&format!("{}<{}>{}</{}>\n", indent_str, tag, n, tag));
            }
            serde_yaml::Value::Bool(b) => {
                xml.push_str(&format!("{}<{}>{}</{}>\n", indent_str, tag, b, tag));
            }
            serde_yaml::Value::Null => {
                xml.push_str(&format!("{}<{} />\n", indent_str, tag));
            }
            _ => {
                xml.push_str(&format!("{}<{} />\n", indent_str, tag));
            }
        }
    }

    /// Конвертировать YAML в HCL (HashiCorp Configuration Language)
    fn yaml_to_hcl(value: &serde_yaml::Value, _pretty: bool) -> Result<String> {
        let mut hcl = String::new();
        Self::yaml_value_to_hcl(value, 0, &mut hcl);
        Ok(hcl)
    }

    fn yaml_value_to_hcl(value: &serde_yaml::Value, indent: usize, hcl: &mut String) {
        let indent_str = " ".repeat(indent * 2);

        match value {
            serde_yaml::Value::Mapping(map) => {
                for (key, val) in map {
                    if let serde_yaml::Value::String(key_str) = key {
                        match val {
                            serde_yaml::Value::Mapping(_) => {
                                hcl.push_str(&format!("{}{} {{\n", indent_str, key_str));
                                Self::yaml_value_to_hcl(val, indent + 1, hcl);
                                hcl.push_str(&format!("{}}}\n", indent_str));
                            }
                            serde_yaml::Value::Sequence(seq) => {
                                hcl.push_str(&format!("{}{} = [\n", indent_str, key_str));
                                for item in seq {
                                    Self::yaml_value_to_hcl(item, indent + 1, hcl);
                                    hcl.push_str(",\n");
                                }
                                hcl.push_str(&format!("{}]\n", indent_str));
                            }
                            _ => {
                                hcl.push_str(&format!("{}{} = {}\n", indent_str, key_str,
                                    Self::yaml_value_to_hcl_string(val)));
                            }
                        }
                    }
                }
            }
            serde_yaml::Value::String(s) => {
                hcl.push_str(&format!("{}{}\n", indent_str, s));
            }
            _ => {
                hcl.push_str(&format!("{}{}\n", indent_str, Self::yaml_value_to_hcl_string(value)));
            }
        }
    }

    fn yaml_value_to_hcl_string(value: &serde_yaml::Value) -> String {
        match value {
            serde_yaml::Value::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            serde_yaml::Value::Number(n) => n.to_string(),
            serde_yaml::Value::Bool(b) => b.to_string(),
            serde_yaml::Value::Null => "null".to_string(),
            _ => "".to_string(),
        }
    }

    /// Конвертировать YAML в INI
    fn yaml_to_ini(value: &serde_yaml::Value, _pretty: bool) -> Result<String> {
        let mut ini = String::new();

        if let serde_yaml::Value::Mapping(map) = value {
            for (section_key, section_val) in map {
                if let serde_yaml::Value::String(section_name) = section_key {
                    ini.push_str(&format!("[{}]\n", section_name));

                    if let serde_yaml::Value::Mapping(entries) = section_val {
                        for (key, val) in entries {
                            if let serde_yaml::Value::String(key_str) = key {
                                let value_str = match val {
                                    serde_yaml::Value::String(s) => s.clone(),
                                    serde_yaml::Value::Number(n) => n.to_string(),
                                    serde_yaml::Value::Bool(b) => b.to_string(),
                                    _ => "".to_string(),
                                };

                                ini.push_str(&format!("{} = {}\n", key_str, value_str));
                            }
                        }
                    }

                    ini.push('\n');
                }
            }
        }

        Ok(ini)
    }

    /// Проверить, является ли файл YAML файлом
    fn is_yaml_file(path: &Path) -> bool {
        path.extension()
            .map(|ext| {
                let ext_str = ext.to_string_lossy().to_lowercase();
                ext_str == "yaml" || ext_str == "yml"
            })
            .unwrap_or(false)
    }

    /// Вывести статистику конвертации
    pub fn print_conversion_results(results: &[ConversionResult], verbose: bool) {
        use colored::*;

        let total_files = results.len();
        let successful = results.iter().filter(|r| r.success).count();
        let failed = total_files - successful;

        let total_size_before: u64 = results.iter().map(|r| r.size_before).sum();
        let total_size_after: u64 = results.iter().map(|r| r.size_after).sum();
        let size_change = if total_size_before > 0 {
            (total_size_after as f64 - total_size_before as f64) / total_size_before as f64 * 100.0
        } else {
            0.0
        };

        println!("{}", "=".repeat(60).blue());
        println!("{}", "Conversion Summary".bold());
        println!("{}", "=".repeat(60).blue());
        println!("Total files: {}", total_files);
        println!("Successful: {}", successful.to_string().green());
        println!("Failed: {}", failed.to_string().red());
        println!("Total size before: {} bytes", total_size_before);
        println!("Total size after: {} bytes", total_size_after);
        println!("Size change: {:.2}%", size_change);

        if verbose && failed > 0 {
            println!("\n{}", "Failed conversions:".yellow().bold());
            for result in results.iter().filter(|r| !r.success) {
                if let Some(error) = &result.error {
                    println!("  {}: {}", result.input_file.red(), error);
                }
            }
        }

        if verbose && successful > 0 {
            println!("\n{}", "Converted files:".green().bold());
            for result in results.iter().filter(|r| r.success) {
                println!("  {} -> {}",
                    result.input_file.blue(),
                    result.output_file.green());

                let change = if result.size_before > 0 {
                    (result.size_after as f64 - result.size_before as f64) /
                     result.size_before as f64 * 100.0
                } else {
                    0.0
                };

                println!("    Size: {} -> {} bytes ({:+.2}%)",
                    result.size_before,
                    result.size_after,
                    change);
            }
        }

        println!("{}", "=".repeat(60).blue());
    }
}
