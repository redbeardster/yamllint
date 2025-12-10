// src/cli.rs
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "yamllint")]
#[command(about = "YAML linter, formatter, and converter", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true)]
    pub config_path: Option<String>,

    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[arg(short = 'O', long, global = true, value_enum, default_value = "text")]
    pub output_format: OutputFormat,  // Переименовываем output в output_format

    #[arg(long, global = true)]
    pub output_file: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Проверить файл или директорию
    Check {
        /// Путь к файлу или директории
        path: String,

        /// Автоматически исправлять найденные проблемы
        #[arg(short = 'F', long)]
        fix: bool,

        /// Выводить только ошибки
        #[arg(short, long)]
        quiet: bool,

        /// Формат вывода результатов
        #[arg(short = 't', long, value_enum)]
        format: Option<OutputFormat>,

        /// Файл для сохранения результатов
        #[arg(long)]
        output_file: Option<String>,
    },

    /// Валидация с использованием JSON Schema
    Validate {
        /// Путь к файлу YAML
        path: String,

        /// Путь к схеме JSON Schema
        #[arg(short, long)]
        schema: Option<String>,

        /// Формат вывода результатов
        #[arg(short = 't', long, value_enum)]
        format: Option<OutputFormat>,
    },

    /// Форматировать YAML файлы
    Format {
        /// Путь к файлу или директории
        path: String,

        /// Форматировать файлы на месте
        #[arg(short, long)]
        in_place: bool,
    },

    /// Конвертировать YAML в другие форматы
    Convert {
        /// Путь к файлу или директории YAML
        input: String,

        /// Целевой формат
        #[arg(short = 'T', long, value_enum)]
        target: TargetFormat,

        /// Формат вывода (только для конвертации)
        #[arg(short = 't', long, value_enum)]
        format: Option<OutputFormat>,

        /// Файл для сохранения результата
        #[arg(short = 'o', long)]
        output_file: Option<String>,  // Переименовываем output в output_file

        /// Сохранять структуру директорий
        #[arg(long)]
        preserve_structure: bool,

        /// Красивый (pretty) вывод
        #[arg(long)]
        pretty: bool,
    },

    /// Управление конфигурацией
    Config {
        /// Сгенерировать конфигурационный файл
        #[arg(short, long)]
        generate: bool,
    },
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum OutputFormat {
    /// Текстовый вывод (по умолчанию)
    Text,

    /// JSON формат
    Json,

    /// YAML формат
    Yaml,

    /// JUnit XML формат (для CI/CD)
    Junit,

    /// GitHub Actions формат
    Github,

    /// Простой вывод (только статистика)
    Simple,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum TargetFormat {
    /// Конвертировать в JSON
    Json,

    /// Конвертировать в YAML (переформатирование)
    Yaml,

    /// Конвертировать в TOML
    Toml,

    /// Конвертировать в XML
    Xml,

    /// Конвертировать в HCL (HashiCorp Configuration Language)
    Hcl,

    /// Конвертировать в INI
    Ini,
}
