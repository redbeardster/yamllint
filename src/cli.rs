use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "yamllint")]
#[command(about = "YAML linter and formatter", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true)]
    pub config_path: Option<String>,

    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Проверить файл или директорию
    Check {
        /// Путь к файлу или директории
        path: String,

        /// Автоматически исправлять найденные проблемы
        #[arg(short, long)]
        fix: bool,

        /// Выводить только ошибки
        #[arg(short, long)]
        quiet: bool,
    },

    /// Валидация с использованием JSON Schema
    Validate {
        /// Путь к файлу YAML
        path: String,

        /// Путь к схеме JSON Schema
        #[arg(short, long)]
        schema: Option<String>,
    },

    /// Форматировать YAML файлы
    Format {
        /// Путь к файлу или директории
        path: String,

        /// Форматировать файлы на месте
        #[arg(short, long)]
        in_place: bool,
    },

    /// Управление конфигурацией
    Config {
        /// Сгенерировать конфигурационный файл
        #[arg(short, long)]
        generate: bool,
    },
}
