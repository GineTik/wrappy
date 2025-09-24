#!/bin/bash

echo "=== Запуск контейнера hello-world ==="
echo "Використовуючи змінну середовища: $MESSAGE"

# Налаштування PATH для використання пакета node-18
# В реальній системі це буде робити ContainerManager
PACKAGES_DIR="${PACKAGES_DIR:-../packages}"
export PATH="$PACKAGES_DIR/node-18/bin:$PATH"

echo "🔧 Налаштовано PATH для node-18: $PATH"
echo "📝 Перевірка версії node:"
which node

echo "🚀 Запуск програми..."
cd content
node app.js

echo "=== Контейнер завершено ==="