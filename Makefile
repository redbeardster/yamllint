.PHONY: lint format convert validate clean

# Линтинг всех YAML файлов
lint:
	@echo "🔍 Линтинг YAML файлов..."
	@cargo run -- check .

# Автоисправление
format:
	@echo "🎨 Форматирование YAML файлов..."
	@cargo run -- check . -F

# Конвертация всех YAML в JSON
convert:
	@echo "🔄 Конвертация YAML в JSON..."
	@find . -name "*.yaml" -o -name "*.yml" | grep -v target | while read file; do \
		echo "Конвертация $$file..."; \
		cargo run -- convert "$$file" -T json --pretty; \
	done

# Валидация Kubernetes манифестов
validate-k8s:
	@echo "✅ Валидация Kubernetes манифестов..."
	@find . -name "*.yaml" -o -name "*.yml" | xargs -I {} cargo run -- validate {}

# Очистка сгенерированных файлов
clean:
	@echo "🧹 Очистка..."
	@find . -name "*.json" -type f -delete
	@find . -name "*.xml" -type f -delete
	@find . -name "*.toml" -type f -delete
	@find . -name "*.ini" -type f -delete
	@find . -name "*.hcl" -type f -delete
	@find . -name "*.bak" -type f -delete
	@rm -f lint-report.json test-results.xml

# Создание отчета CI/CD
report:
	@echo "📊 Создание отчетов CI/CD..."
	@cargo run -- check . -O junit --output-file test-results.xml
	@cargo run -- check . -O json --output-file lint-report.json
	@echo "Отчеты созданы:"
	@echo "  - test-results.xml (JUnit)"
	@echo "  - lint-report.json (JSON)"

# Help
help:
	@echo "Доступные команды:"
	@echo "  make lint       - Проверить все YAML файлы"
	@echo "  make format     - Автоисправление YAML файлов"
	@echo "  make convert    - Конвертировать все YAML в JSON"
	@echo "  make validate-k8s - Валидация Kubernetes манифестов"
	@echo "  make report     - Создать отчеты для CI/CD"
	@echo "  make clean      - Очистить сгенерированные файлы"
	@echo "  make help       - Показать эту справку"
