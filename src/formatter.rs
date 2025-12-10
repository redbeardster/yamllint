use crate::config::Config;
use crate::linter::LintReport;
use std::fs;
use std::path::Path;

pub fn auto_fix_files(reports: &[LintReport], config: &Config) -> anyhow::Result<()> {
    for report in reports {
        if !report.results.is_empty() {
            auto_fix_file(&report.file, config)?;
        }
    }
    Ok(())
}

pub fn auto_fix_file<P: AsRef<Path>>(path: P, config: &Config) -> anyhow::Result<()> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)?;

    let fixed_content = fix_content(&content, config);

    if config.format.backup_files {
        let backup_path = path.with_extension("yaml.bak");
        fs::copy(path, backup_path)?;
    }

    fs::write(path, fixed_content)?;
    println!("Fixed: {}", path.display());

    Ok(())
}

fn fix_content(content: &str, config: &Config) -> String {
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // 1. Исправление отступов
    fix_indentation(&mut lines, config);

    // 2. Удаление trailing spaces
    fix_trailing_spaces(&mut lines);

    // 3. Исправление пустых строк
    fix_empty_lines(&mut lines, config);

    // 4. Форматирование кавычек (пока без regex)
    fix_quotes(&mut lines, config);

    // 5. Добавляем финальную новую строку
    lines.join("\n") + "\n"
}

fn fix_indentation(lines: &mut [String], config: &Config) {
    let expected_spaces = config.rules.indentation.spaces;

    for line in lines.iter_mut() {
        if line.trim().is_empty() {
            continue;
        }

        if line.starts_with(' ') {
            let leading_spaces = line.len() - line.trim_start().len();
            let new_indent = (leading_spaces / expected_spaces) * expected_spaces;
            let new_line = " ".repeat(new_indent) + line.trim_start();
            *line = new_line;
        }
    }
}

fn fix_trailing_spaces(lines: &mut [String]) {
    for line in lines.iter_mut() {
        *line = line.trim_end().to_string();
    }
}

fn fix_empty_lines(lines: &mut Vec<String>, config: &Config) {
    // Удаляем пустые строки в начале
    while !lines.is_empty() && lines[0].trim().is_empty() {
        lines.remove(0);
    }

    // Удаляем пустые строки в конце
    while !lines.is_empty() && lines.last().unwrap().trim().is_empty() {
        lines.pop();
    }

    // Удаляем лишние последовательные пустые строки
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim().is_empty() {
            let mut consecutive = 1;
            let mut j = i + 1;

            while j < lines.len() && lines[j].trim().is_empty() {
                consecutive += 1;
                j += 1;
            }

            if consecutive > config.rules.empty_lines.max_consecutive {
                let to_remove = consecutive - config.rules.empty_lines.max_consecutive;
                lines.drain(i..i + to_remove);
                i += config.rules.empty_lines.max_consecutive;
            } else {
                i = j;
            }
        } else {
            i += 1;
        }
    }

    // Добавляем одну пустую строку в конце, если нужно
    if config.rules.empty_lines.max_end > 0 {
        lines.push("".to_string());
    }
}

fn fix_quotes(lines: &mut [String], config: &Config) {
    // Упрощенная версия без regex
    for line in lines.iter_mut() {
        if config.rules.quotes.prefer_double {
            // Заменяем одинарные кавычки на двойные только в простых случаях
            *line = line.replace('\'', "\"");
        }
    }
}

pub fn format_files<P: AsRef<Path>>(path: P, in_place: bool, config: &Config) -> anyhow::Result<()> {
    use ignore::Walk;

    for entry in Walk::new(path) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
            let content = fs::read_to_string(path)?;
            let formatted = fix_content(&content, config);

            if content != formatted {
                if in_place {
                    if config.format.backup_files {
                        let backup_path = path.with_extension("yaml.bak");
                        fs::copy(path, backup_path)?;
                    }
                    fs::write(path, formatted)?;
                    println!("Formatted: {}", path.display());
                } else {
                    println!("// File: {}", path.display());
                    println!("{}", formatted);
                    println!("---");
                }
            }
        }
    }

    Ok(())
}
