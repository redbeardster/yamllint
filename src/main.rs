// src/main.rs
mod cli;
mod config;
mod linter;
mod rules;
mod formatter;
mod exporter;
mod converter;

use anyhow::Result;
use clap::Parser;
use config::Config;
use linter::YamlLinter;
use std::path::Path;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Check {
            path,
            fix,
            quiet,
            format,
            output_file,
        } => {
            let config = match cli.config_path.as_ref() {
                Some(path) => Config::from_file(path)?,
                None => Config::default(),
            };

            let linter = YamlLinter::new(config);
            let results = if Path::new(&path).is_dir() {
                linter.lint_directory(&path)?
            } else {
                vec![linter.lint_file(&path)?]
            };

            if fix {
                formatter::auto_fix_files(&results, &linter.config)?;
            }

            // Определяем формат вывода
            let output_format = format.unwrap_or(cli.output_format);  // Используем cli.output_format
            let output_file_path = output_file.as_ref().or(cli.output_file.as_ref());

            if output_format != cli::OutputFormat::Text {
                // Экспортируем в указанный формат
                exporter::Exporter::export_reports(&results, output_format, output_file_path)?;
            } else {
                // Выводим в текстовом формате (если не quiet)
                if !quiet {
                    linter.print_results(&results);
                }
            }

            if results.iter().any(|r| !r.passed) && !fix {
                std::process::exit(1);
            }
        }

        cli::Commands::Validate {
            path,
            schema: _,
            format,
        } => {
            let config = match cli.config_path.as_ref() {
                Some(path) => Config::from_file(path)?,
                None => Config::default(),
            };

            let linter = YamlLinter::new(config);
            let result = linter.validate_file(&path)?;

            // Определяем формат вывода
            let output_format = format.unwrap_or(cli.output_format);  // Используем cli.output_format

            if output_format != cli::OutputFormat::Text {
                // Экспортируем в указанный формат
                exporter::Exporter::export_validation(&result, output_format, cli.output_file.as_ref())?;
            } else {
                // Выводим в текстовом формате
                linter.print_validation_results(&result);
            }

            if !result.valid {
                std::process::exit(1);
            }
        }

        cli::Commands::Format { path, in_place } => {
            let config = match cli.config_path.as_ref() {
                Some(path) => Config::from_file(path)?,
                None => Config::default(),
            };

            formatter::format_files(&path, in_place, &config)?;
        }

        cli::Commands::Convert {
            input,
            target,
            format,
            output_file,  // Изменяем output на output_file
            preserve_structure,
            pretty,
        } => {
            let results = if Path::new(&input).is_dir() {
                converter::YamlConverter::convert_directory(
                    &input,
                    &target,
                    output_file.as_deref(),  // Используем output_file
                    preserve_structure,
                    pretty,
                )?
            } else {
                vec![converter::YamlConverter::convert_file(
                    &input,
                    &target,
                    output_file.as_deref(),  // Используем output_file
                    pretty,
                )?]
            };

            // Если указан формат вывода, экспортируем результаты
            if let Some(output_format) = format {
                // Создаем структуру для экспорта
                let export_data = converter::YamlConverter::create_export_data(&results);
                exporter::Exporter::export_conversion_results(&export_data, output_format, cli.output_file.as_ref())?;
            } else {
                // Выводим обычную статистику
                converter::YamlConverter::print_conversion_results(&results, cli.verbose);
            }
        }

        cli::Commands::Config { generate } => {
            if generate {
                let default_config = Config::default();
                let yaml = serde_yaml::to_string(&default_config)?;
                std::fs::write(".yamllint.yaml", yaml)?;
                println!("Generated default .yamllint.yaml");
            } else {
                let config = match cli.config_path.as_ref() {
                    Some(path) => Config::from_file(path)?,
                    None => Config::default(),
                };
                println!("Current configuration:");
                println!("{:#?}", config);
            }
        }
    }

    Ok(())
}
