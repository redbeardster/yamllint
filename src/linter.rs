use crate::config::Config;
use crate::rules::{RuleChecker, LintResult};
use ignore::Walk;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct LintReport {
    pub file: String,
    pub results: Vec<LintResult>,
    pub passed: bool,
}

#[derive(Debug)]
pub struct ValidationResult {
    pub file: String,
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub struct YamlLinter {
    pub config: Config,
    checker: RuleChecker,
}

impl YamlLinter {
    pub fn new(config: Config) -> Self {
        let checker = RuleChecker::new(config.clone());
        YamlLinter { config, checker }
    }

    pub fn lint_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<LintReport> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;

        let results = self.checker.check_file(&content, path.to_str().unwrap_or(""));

        Ok(LintReport {
            file: path.to_string_lossy().to_string(),
            results: results.clone(),
            passed: !results.iter().any(|r| r.is_error()),
        })
    }

    pub fn lint_directory<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<Vec<LintReport>> {
        let mut reports = vec![];

        for entry in Walk::new(path) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
                let path_str = path.to_string_lossy().to_string();

                if self.config.should_exclude(&path_str) {
                    continue;
                }

                match self.lint_file(path) {
                    Ok(report) => reports.push(report),
                    Err(e) => eprintln!("Error processing {}: {}", path_str, e),
                }
            }
        }

        Ok(reports)
    }

    pub fn print_results(&self, reports: &[LintReport]) {
        use colored::*;

        let mut total_errors = 0;
        let mut total_warnings = 0;

        for report in reports {
            if report.results.is_empty() {
                println!("{} {}: {}", "✓".green(), report.file, "OK".green());
                continue;
            }

            println!("\n{}:", report.file);

            for result in &report.results {
                let (icon, color) = match result.severity {
                    crate::config::Severity::Error => ("✗", Color::Red),
                    crate::config::Severity::Warning => ("!", Color::Yellow),
                    crate::config::Severity::Info => ("i", Color::Blue),
                    crate::config::Severity::Off => continue,
                };

                println!("  {} {}:{}:{} {}",
                    icon.color(color),
                    result.line,
                    result.column,
                    result.rule.color(color),
                    result.message
                );

                if !result.snippet.is_empty() {
                    println!("      {}", result.snippet.dimmed());
                }

                match result.severity {
                    crate::config::Severity::Error => total_errors += 1,
                    crate::config::Severity::Warning => total_warnings += 1,
                    _ => {}
                }
            }
        }

        println!("\n{}", "=".repeat(50));
        println!("Summary:");
        println!("  Files checked: {}", reports.len());
        println!("  Errors: {}", total_errors);
        println!("  Warnings: {}", total_warnings);

        if total_errors == 0 && total_warnings == 0 {
            println!("  {} All checks passed!", "✓".green());
        }
    }

    pub fn validate_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<ValidationResult> {
        let report = self.lint_file(path)?;

        Ok(ValidationResult {
            file: report.file,
            valid: report.passed,
            errors: report.results.iter()
                .filter(|r| r.is_error())
                .map(|r| r.message.clone())
                .collect(),
            warnings: report.results.iter()
                .filter(|r| r.is_warning())
                .map(|r| r.message.clone())
                .collect(),
        })
    }

    pub fn print_validation_results(&self, result: &ValidationResult) {
        use colored::*;

        println!("Validation for: {}", result.file);

        if result.valid {
            println!("{} Valid", "✓".green());
        } else {
            println!("{} Invalid", "✗".red());

            if !result.errors.is_empty() {
                println!("\nErrors:");
                for error in &result.errors {
                    println!("  • {}", error.red());
                }
            }

            if !result.warnings.is_empty() {
                println!("\nWarnings:");
                for warning in &result.warnings {
                    println!("  • {}", warning.yellow());
                }
            }
        }
    }
}
