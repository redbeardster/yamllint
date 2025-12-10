markdown
# 🚀 yamllint-rs

**Быстрый, надежный и многофункциональный линтер, форматтер и конвертер YAML на Rust**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/yourusername/yamllint-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/yamllint-rs/actions)

> Альтернатива Python yamllint и другим инструментам YAML обработки. В 5-10 раз быстрее, без зависимостей, один бинарник.

## ✨ Особенности

- ⚡ **Высокая производительность** - Обработка тысяч файлов за секунды
- 🛡️ **Безопасность памяти** - Никаких segfault, благодаря Rust
- 📦 **Один бинарник** - Нет зависимостей на Python/Node.js
- 🔧 **Многофункциональность** - Линтинг, форматирование, конвертация, валидация
- 📊 **Гибкий вывод** - JSON, YAML, JUnit, GitHub Actions, Simple
- 🔄 **Конвертация** - YAML ↔ JSON, XML, TOML, INI, HCL
- 🎯 **Точность** - Детальные проверки синтаксиса и семантики
- ⚙️ **Настраиваемость** - Конфигурационные файлы .yamllint.yaml

## 📦 Установка

### Из исходников
```bash
cargo install --git https://github.com/yourusername/yamllint-rs
git clone https://github.com/yourusername/yamllint-rs
cd yamllint-rs
cargo build --release
sudo cp target/release/yamllint /usr/local/bin/

# Linux
curl -L https://github.com/yourusername/yamllint-rs/releases/latest/download/yamllint-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv yamllint /usr/local/bin/

# macOS (Homebrew)
brew install yourusername/tap/yamllint-rs
```

## 🚀 Использование
Базовые команды

### Проверка файла или директории
```
yamllint check config.yaml
yamllint check ./kubernetes/
```
### Автоисправление проблем
```
yamllint check config.yaml -F
```
### Конвертация YAML в другие форматы
```
yamllint convert config.yaml -T json --pretty
yamllint convert config.yaml -T xml
yamllint convert config.yaml -T toml
```
### Форматирование YAML
```
yamllint format config.yaml --in-place
```
### Валидация с JSON Schema
```
yamllint validate deployment.yaml --schema k8s-schema.json
```
## Линтинг

#### Проверка с выводом ошибок
```
yamllint check config.yaml
```
#### Только ошибки (без предупреждений)
```
yamllint check config.yaml --quiet
```
#### Экспорт результатов в JSON
```
yamllint check config.yaml -O json --output-file results.json
```
#### Пакетная проверка директории
```
yamllint check ./configs/ -O github  # Для GitHub Actions
```
### Конвертация

#### YAML → JSON (компактный)
```
yamllint convert config.yaml -T json
```
#### YAML → JSON (красивый)
```
yamllint convert config.yaml -T json --pretty
```
#### YAML → XML
```
yamllint convert config.yaml -T xml
```
#### YAML → TOML
```
yamllint convert config.yaml -T toml
```
#### YAML → INI
```
yamllint convert config.yaml -T ini
```
#### Пакетная конвертация директории
```
yamllint convert ./yaml-configs/ -T json --preserve-structure
```
## Форматы вывода

#### Текстовый (по умолчанию)
```
yamllint check . -O text
```
#### JSON
```
yamllint check . -O json
```
#### YAML
```
yamllint check . -O yaml
```
#### JUnit XML (для CI/CD)
```
yamllint check . -O junit --output-file test-results.xml
```
#### GitHub Actions
```
yamllint check . -O github
```
#### Simple (только статистика)
```
yamllint check . -O simple
```

## ⚙️ Конфигурация

### Создайте .yamllint.yaml в корне проекта:
```
yaml
rules:
  indentation:
    spaces: 2
    check-multi-line-strings: true
    
  line-length:
    max: 120
    allow-non-breakable-words: true
    
  trailing-spaces:
    level: error
    
  empty-lines:
    max-start: 0
    max-end: 1
    max-consecutive: 2
    
  required-fields:
    paths:
      "**/k8s/*.yaml":
        - apiVersion
        - kind
        - metadata.name
      "**/docker-compose*.yaml":
        - version
        - services
        
  value-types:
    strict-numbers: true
    check-bool-values: true
    
  duplicates:
    level: error
    
  quotes:
    prefer-double: false
    
format:
  auto-fix: false
  backup-files: true
  indent-sequence: true
  
exclude:
  - "**/node_modules/"
  - "**/.git/"
  - "**/vendor/"
```

### Использование конфигурации:
```
yamllint --config-path .yamllint.yaml check .
```

### 🔧 Интеграции
### Pre-commit Hook
#### Установка pre-commit хука
```
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
echo "🔍 Running YAML lint..."
yamllint check . --quiet
EOF
chmod +x .git/hooks/pre-commit
```
### GitHub Actions
```
name: YAML Lint
on: [push, pull_request]
jobs:
  yaml-lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install yamllint-rs
        run: |
          curl -L https://github.com/yourusername/yamllint-rs/releases/latest/download/yamllint-x86_64-unknown-linux-gnu.tar.gz | tar xz
          sudo mv yamllint /usr/local/bin/
      - name: Lint YAML files
        run: yamllint check . --output github
      - name: Upload test results
        uses: actions/upload-artifact@v3
        with:
          name: yaml-lint-results
          path: yaml-lint-results.json
```
### GitLab CI
```
yamllint:
  image: rust:latest
  script:
    - cargo install yamllint-rs
    - yamllint check .
  artifacts:
    reports:
      junit: test-results.xml
```
