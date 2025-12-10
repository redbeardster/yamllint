mod cli;
mod config;
mod linter;
mod rules;
mod formatter;

use anyhow::Result;
use clap::Parser;
use config::Config;
use linter::YamlLinter;
use std::path::Path;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    // Загружаем конфигурацию
    let config = match cli.config_path.as_ref() {
        Some(path) => Config::from_file(path)?,
        None => Config::default(),
    };

    let linter = YamlLinter::new(config);

    match cli.command {
        cli::Commands::Check { path, fix, quiet: _ } => {
            let results = if Path::new(&path).is_dir() {
                linter.lint_directory(&path)?
            } else {
                vec![linter.lint_file(&path)?]
            };

            if fix {
                formatter::auto_fix_files(&results, &linter.config)?;
            }

            linter.print_results(&results);

            if results.iter().any(|r| !r.passed) && !fix {
                std::process::exit(1);
            }
        }

        cli::Commands::Validate { path, schema: _ } => {
            let result = linter.validate_file(&path)?;
            linter.print_validation_results(&result);

            if !result.valid {
                std::process::exit(1);
            }
        }

        cli::Commands::Format { path, in_place } => {
            formatter::format_files(&path, in_place, &linter.config)?;
        }

        cli::Commands::Config { generate } => {
            if generate {
                let default_config = Config::default();
                let yaml = serde_yaml::to_string(&default_config)?;
                std::fs::write(".yamllint.yaml", yaml)?;
                println!("Generated default .yamllint.yaml");
            } else {
                println!("Current configuration:");
                println!("{:#?}", linter.config);
            }
        }
    }

    Ok(())
}
