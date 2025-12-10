use crate::cli::OutputFormat;
use crate::converter::ConversionExport;
use crate::linter::{LintReport, ValidationResult};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportData {
    pub timestamp: String,
    pub summary: ExportSummary,
    pub reports: Vec<ExportReport>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportSummary {
    pub total_files: usize,
    pub files_with_errors: usize,
    pub files_with_warnings: usize,
    pub total_errors: usize,
    pub total_warnings: usize,
    pub success_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportReport {
    pub file: String,
    pub errors: usize,
    pub warnings: usize,
    pub passed: bool,
    pub issues: Vec<ExportIssue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportIssue {
    pub line: usize,
    pub column: usize,
    pub severity: String,
    pub rule: String,
    pub message: String,
    pub snippet: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationExport {
    pub timestamp: String,
    pub file: String,
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub struct Exporter;

impl Exporter {
    pub fn export_reports(
        reports: &[LintReport],
        format: OutputFormat,
        output_file: Option<&String>,
    ) -> anyhow::Result<()> {
        let export_data = Self::prepare_export_data(reports);

        let output = match format {
            OutputFormat::Json => Self::to_json(&export_data)?,
            OutputFormat::Yaml => Self::to_yaml(&export_data)?,
            OutputFormat::Junit => Self::to_junit(&export_data)?,
            OutputFormat::Github => Self::to_github_actions(&export_data)?,
            OutputFormat::Simple => Self::to_simple(&export_data)?,
            OutputFormat::Text => return Ok(()), // Обрабатывается в linter.rs
        };

        match output_file {
            Some(path) => {
                fs::write(path, output)?;
                println!("Results exported to: {}", path);
            }
            None => {
                println!("{}", output);
            }
        }

        Ok(())
    }

    pub fn export_validation(
        result: &ValidationResult,
        format: OutputFormat,
        output_file: Option<&String>,
    ) -> anyhow::Result<()> {
        let export_data = ValidationExport {
            timestamp: Utc::now().to_rfc3339(),
            file: result.file.clone(),
            valid: result.valid,
            errors: result.errors.clone(),
            warnings: result.warnings.clone(),
        };

        let output = match format {
            OutputFormat::Json => serde_json::to_string_pretty(&export_data)?,
            OutputFormat::Yaml => serde_yaml::to_string(&export_data)?,
            OutputFormat::Text => return Ok(()), // Обрабатывается в linter.rs
            _ => return Err(anyhow::anyhow!("Format not supported for validation")),
        };

        match output_file {
            Some(path) => {
                fs::write(path, output)?;
                println!("Validation results exported to: {}", path);
            }
            None => {
                println!("{}", output);
            }
        }

        Ok(())
    }

    pub fn export_conversion_results(
        data: &ConversionExport,
        format: OutputFormat,
        output_file: Option<&String>,
    ) -> anyhow::Result<()> {
        let output = match format {
            OutputFormat::Json => serde_json::to_string_pretty(data)?,
            OutputFormat::Yaml => serde_yaml::to_string(data)?,
            OutputFormat::Text => return Ok(()), // Обрабатывается в converter.rs
            _ => return Err(anyhow::anyhow!("Format not supported for conversion results")),
        };

        match output_file {
            Some(path) => {
                fs::write(path, output)?;
                println!("Conversion results exported to: {}", path);
            }
            None => {
                println!("{}", output);
            }
        }

        Ok(())
    }

    fn prepare_export_data(reports: &[LintReport]) -> ExportData {
        let total_errors = reports
            .iter()
            .flat_map(|r| &r.results)
            .filter(|r| r.is_error())
            .count();

        let total_warnings = reports
            .iter()
            .flat_map(|r| &r.results)
            .filter(|r| r.is_warning())
            .count();

        let files_with_errors = reports
            .iter()
            .filter(|r| r.results.iter().any(|issue| issue.is_error()))
            .count();

        let files_with_warnings = reports
            .iter()
            .filter(|r| r.results.iter().any(|issue| issue.is_warning()))
            .count();

        let success_rate = if reports.is_empty() {
            100.0
        } else {
            (reports.iter().filter(|r| r.passed).count() as f64 / reports.len() as f64) * 100.0
        };

        ExportData {
            timestamp: Utc::now().to_rfc3339(),
            summary: ExportSummary {
                total_files: reports.len(),
                files_with_errors,
                files_with_warnings,
                total_errors,
                total_warnings,
                success_rate,
            },
            reports: reports
                .iter()
                .map(|report| ExportReport {
                    file: report.file.clone(),
                    errors: report.results.iter().filter(|r| r.is_error()).count(),
                    warnings: report.results.iter().filter(|r| r.is_warning()).count(),
                    passed: report.passed,
                    issues: report
                        .results
                        .iter()
                        .map(|issue| ExportIssue {
                            line: issue.line,
                            column: issue.column,
                            severity: format!("{:?}", issue.severity).to_lowercase(),
                            rule: issue.rule.clone(),
                            message: issue.message.clone(),
                            snippet: issue.snippet.clone(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }

    fn to_json(data: &ExportData) -> anyhow::Result<String> {
        serde_json::to_string_pretty(data).map_err(|e| e.into())
    }

    fn to_yaml(data: &ExportData) -> anyhow::Result<String> {
        serde_yaml::to_string(data).map_err(|e| e.into())
    }

    fn to_junit(data: &ExportData) -> anyhow::Result<String> {
        let mut xml = String::new();

        xml.push_str(&format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="yamllint" timestamp="{}" tests="{}" failures="{}" errors="{}">"#,
            data.timestamp,
            data.summary.total_files,
            data.summary.files_with_errors,
            data.summary.total_errors
        ));

        for report in &data.reports {
            xml.push_str(&format!(
                r#"
  <testsuite name="{}" tests="1" failures="{}" errors="{}" skipped="0" timestamp="{}">"#,
                Self::escape_xml(&report.file),
                if report.passed { 0 } else { 1 },
                report.errors,
                data.timestamp
            ));

            xml.push_str(&format!(
                r#"
    <testcase name="yaml_lint" classname="{}" time="0">"#,
                Self::escape_xml(&report.file)
            ));

            if !report.passed {
                for issue in &report.issues {
                    if issue.severity == "error" {
                        xml.push_str(&format!(
                            r#"
      <failure message="{}" type="{}">Line {}: {}</failure>"#,
                            Self::escape_xml(&issue.message),
                            issue.rule,
                            issue.line,
                            Self::escape_xml(&issue.snippet)
                        ));
                    }
                }
            }

            xml.push_str(r#"
    </testcase>
  </testsuite>"#);
        }

        xml.push_str("\n</testsuites>");
        Ok(xml)
    }

    fn to_github_actions(data: &ExportData) -> anyhow::Result<String> {
        let mut output = String::new();

        for report in &data.reports {
            for issue in &report.issues {
                let level = if issue.severity == "error" {
                    "error"
                } else {
                    "warning"
                };

                output.push_str(&format!(
                    "::{} file={},line={},col={},title={}::{}\n",
                    level,
                    report.file,
                    issue.line,
                    issue.column,
                    issue.rule,
                    issue.message
                ));
            }
        }

        // Добавляем итоговую аннотацию
        if data.summary.total_errors > 0 {
            output.push_str(&format!(
                "::error::Found {} errors and {} warnings in {} files\n",
                data.summary.total_errors,
                data.summary.total_warnings,
                data.summary.total_files
            ));
        }

        Ok(output)
    }

    fn to_simple(data: &ExportData) -> anyhow::Result<String> {
        let mut output = String::new();

        output.push_str(&format!(
            "Files: {}, Errors: {}, Warnings: {}, Success: {:.1}%\n",
            data.summary.total_files,
            data.summary.total_errors,
            data.summary.total_warnings,
            data.summary.success_rate
        ));

        for report in &data.reports {
            if !report.passed {
                output.push_str(&format!(
                    "  {}: {} errors, {} warnings\n",
                    report.file, report.errors, report.warnings
                ));
            }
        }

        Ok(output)
    }

    fn escape_xml(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}
