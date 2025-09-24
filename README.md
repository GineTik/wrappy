# Wrappy - Containers System

Система контейнерної абстракції файлової системи - новий підхід до взаємодії з додатками та файлами через контейнери.

## Структура проєкту

```
containers/
├── src/
│   ├── core/           # Основна логіка системи
│   ├── repository/     # Управління репозиторієм
│   ├── cli/           # CLI інтерфейс
│   ├── api/           # HTTP API
│   ├── types/         # Типи та схеми
│   └── utils/         # Допоміжні утиліти
├── test/              # Тести
├── examples/          # Приклади контейнерів
└── docs/             # Документація
```

## Встановлення

```bash
npm install
npm start
```

## Команди

```bash
# Встановити контейнер
containers install <name>

# Запустити контейнер
containers run <name> [script]

# Список контейнерів
containers list

# Видалити контейнер
containers remove <name>
```

## Структура контейнера

```
my-app/
├── manifest.json      # Маніфест з метаданими
├── scripts/
│   ├── default.sh     # Дефолтний скрипт
│   └── build.sh       # Додаткові скрипти
├── content/           # Файли додатку
├── config/
│   ├── permissions.json
│   └── environment.json
└── dependencies.json  # Залежності
```
