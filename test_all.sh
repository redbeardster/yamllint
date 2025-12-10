#!/bin/bash
echo "=== Начинаем тестирование yamllint-rs ==="
echo ""

# 1. Тест конвертации
echo "1. Тестируем конвертацию YAML -> JSON"
echo 'test: value' > test1.yaml
cargo run --quiet -- convert test1.yaml -T json
if [ -f test1.json ]; then
    echo "✓ Конвертация успешна"
else
    echo "✗ Ошибка конвертации"
fi

# 2. Тест линтинга
echo ""
echo "2. Тестируем линтинг"
echo 'test:  value' > test2.yaml  # С пробелом после двоеточия
cargo run --quiet -- check test2.yaml > /dev/null 2>&1
if [ $? -ne 0 ]; then
    echo "✓ Линтинг обнаружил ошибку"
else
    echo "✗ Линтинг не обнаружил ошибку"
fi

# 3. Тест автоисправления
echo ""
echo "3. Тестируем автоисправление"
cargo run --quiet -- check test2.yaml -F
if grep -q 'test: value' test2.yaml; then
    echo "✓ Автоисправление успешно"
else
    echo "✗ Ошибка автоисправления"
fi

# 4. Тест форматирования
echo ""
echo "4. Тестируем форматирование"
echo -e 'test:\n  nested: value' > test3.yaml
cargo run --quiet -- format test3.yaml --in-place
if [ -s test3.yaml ]; then
    echo "✓ Форматирование успешно"
else
    echo "✗ Ошибка форматирования"
fi

# 5. Тест валидации
echo ""
echo "5. Тестируем валидацию"
echo 'apiVersion: v1\nkind: Pod' > test4.yaml
cargo run --quiet -- validate test4.yaml > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✓ Валидация успешна"
else
    echo "✗ Ошибка валидации"
fi

# 6. Тест разных форматов вывода
echo ""
echo "6. Тестируем форматы вывода"
cargo run --quiet -- check test1.yaml -O json > test_output.json 2>&1
if [ -s test_output.json ]; then
    echo "✓ JSON вывод работает"
else
    echo "✗ Ошибка JSON вывода"
fi

# Уборка
rm -f test*.yaml test*.json test_output.json
rm -f .yamllint.yaml

echo ""
echo "=== Тестирование завершено ==="
