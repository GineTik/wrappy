#!/bin/bash

echo "=== Запуск контейнера hello-world ==="
echo "Використовуючи змінну середовища: $MESSAGE"

# Використання auto-injected команд від пакетів
# Wrappy автоматично створює wrapper'и в bin/ директорії
CONTAINER_DIR="$(dirname "$(dirname "$(readlink -f "$0")")")"
export PATH="$CONTAINER_DIR/bin:$PATH"

echo "🔧 Використання injected commands з: $CONTAINER_DIR/bin"
echo "📝 Перевірка команд:"
ls -la "$CONTAINER_DIR/bin/" 2>/dev/null || echo "Немає injected команд"
which node 2>/dev/null && echo "✅ node знайдено" || echo "❌ node не знайдено"

echo "🚀 Запуск програми..."
cd content
node app.js

echo "=== Контейнер завершено ==="