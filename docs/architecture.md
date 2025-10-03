# Архітектура Wrappy

## Модульна структура

```
src/
├── lib.rs              # Описує модулі додатка
├── main.rs             # Вхідна точка додатка
├── cli/                # CLI інтерфейс
├── features/           # Бізнес-логіка по фічам
│   ├── container/      # Управління контейнерами
│   ├── bindings/       # Система біндингів
│   ├── manifest/       # Робота з маніфестами
│   └── version/        # Управління версіями
└── shared/             # Спільна інфраструктурна логіка
    └── error.rs        # Система помилок
```

## Принципи архітектури

### Feature-Driven Development
Проєкт організований навколо функціональних модулів (features), кожен з яких відповідає за окрему область бізнес-логіки:

- **container** - управління життєвим циклом контейнерів
- **bindings** - інтеграція контейнерів з хост-системою
- **manifest** - робота з конфігураційними файлами
- **version** - управління версіями

### Модульна структура Feature

Кожен feature-модуль структурується за стандартним шаблоном:

```
feature_name/
├── mod.rs              # Оголошення та експорт модулів
├── commands.rs         # CLI команди та обробка IO операцій
├── service.rs          # Бізнес-логіка та координація
├── types.rs            # Типи даних модуля
├── validators.rs       # Логіка валідування (опціонально)
└── mappers.rs          # Логіка мапінгу даних (опціонально)
```

#### Відповідальності модулів

**`commands.rs`**
- Обробка команд CLI
- Парсинг та валідація вхідних параметрів
- Мапінг CLI аргументів у внутрішні типи
- Виклик відповідних сервісів
- Форматування виводу для користувача

**`service.rs`**
- Координація бізнес-процесів
- Оркестрація взаємодії між різними компонентами
- Реалізація основної бізнес-логіки
- Управління транзакціями та станом

**`types.rs`**
- Визначення структур даних
- Енумерації для типізованих значень
- Трейти для абстракцій
- Серіалізація/десеріалізація

### Shared модулі

Shared модулі містять інфраструктурну логіку, яка використовується в декількох features:

**`error.rs`**
- Централізована система помилок
- Типізовані помилки для різних сценаріїв
- Конвертація між різними типами помилок

## Потоки даних

### CLI до Service

```
User Input (CLI)
    ↓
commands.rs (парсинг, валідація)
    ↓
service.rs (бізнес-логіка)
    ↓
Інші модулі (files, external commands)
    ↓
Result/Error → commands.rs
    ↓
Форматований вивід користувачу
```

### Міжмодульна взаємодія

```
container/service.rs
    ↓ використовує
manifest/service.rs (завантаження конфігурації)
    ↓ використовує
bindings/service.rs (створення біндингів)
    ↓ використовує
shared/error.rs (обробка помилок)
```

## Управління залежностями

### Внутрішні залежності

- Features можуть залежати від shared модулів
- Features можуть залежати один від одного через публічні API
- Уникаємо циклічних залежностей між features

### Зовнішні залежності

**Core dependencies:**
- `serde` - серіалізація/десеріалізація JSON
- `clap` - CLI парсинг з derive макросами
- `dirs` - робота з системними директоріями

**Development dependencies:**
- `tempfile` - тимчасові файли для тестів
- Standard Rust testing framework

## Обробка помилок

### Централізована система помилок

```rust
// shared/error.rs
pub enum ContainerError {
    IoError { path: PathBuf, source: std::io::Error },
    InvalidManifest(String),
    MissingDefaultScript,
    InvalidPath { path: PathBuf, reason: String },
    // ... інші типи помилок
}

pub type ContainerResult<T> = Result<T, ContainerError>;
```

### Конвертація помилок

- Автоматична конвертація системних помилок через `map_err`
- Контекстуальна інформація в повідомленнях про помилки
- Збереження stack trace через source errors

## Тестування

### Структура тестів

```
src/
├── features/
│   └── container/
│       ├── service.rs
│       └── service.rs         # unit tests в кінці файлу
└── tests/                     # integration tests
    ├── container_tests.rs
    └── bindings_tests.rs
```

### Принципи тестування

- **Unit tests** - в кінці кожного модуля (`#[cfg(test)]`)
- **Integration tests** - окремі файли в `tests/`
- **Arrange-Act-Assert** pattern для структури тестів
- Використання `tempfile` для тестування файлових операцій

## Конфігурація через Cargo.toml

### Features flags (майбутнє)

```toml
[features]
default = ["cli"]
cli = ["clap"]
api = ["serde_json"]
monitoring = ["metrics"]
```

### Розділення dev/prod залежностей

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
clap = { version = "4.0", features = ["derive"] }

[dev-dependencies]
tempfile = "3.0"
assert_matches = "1.5"
```

## Безпека

### Принципи безпечного коду

- **Path traversal prevention** - валідація всіх шляхів файлів
- **Permission management** - контроль прав доступу до створюваних файлів
- **Input validation** - валідація всіх користувацьких вводів
- **Error information leakage** - обережне розкриття інформації в помилках

### Управління правами

```rust
// Встановлення безпечних прав для wrapper скриптів
use std::os::unix::fs::PermissionsExt;

let mut perms = fs::metadata(&path)?.permissions();
perms.set_mode(0o755); // rwxr-xr-x
fs::set_permissions(&path, perms)?;
```

## Розширюваність

### Додавання нових features

1. Створити нову папку в `src/features/`
2. Реалізувати стандартну структуру модуля
3. Додати модуль в `src/features/mod.rs`
4. Експортувати публічні API через `src/lib.rs`

### Додавання нових типів біндингів

1. Розширити enum `BindingType` в `bindings/types.rs`
2. Реалізувати логіку в відповідному manager
3. Додати CLI команди в `bindings/commands.rs`
4. Оновити документацію

## Продуктивність

### Мінімізація алокацій

- Використання `&str` замість `String` де можливо
- Borrowing замість клонування для читання
- Lazy evaluation через closures

### Файлові операції

- Буферизоване читання великих файлів
- Atomic operations для критичних файлів
- Перевірка існування файлів перед операціями

## Підтримка платформ

### Unix-специфічний код

```rust
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(unix)]
fn set_executable_permissions(path: &Path) -> Result<()> {
    // Unix-specific implementation
}

#[cfg(not(unix))]
fn set_executable_permissions(path: &Path) -> Result<()> {
    // Windows fallback or error
}
```

### Кросплатформні шляхи

- Використання `PathBuf` та `Path` для всіх файлових операцій
- Уникання hardcoded `/` або `\` в шляхах
- Підтримка `~` expansion через `dirs` crate

## Майбутні розширення

### Планована архітектура

1. **Plugin система** - динамічне завантаження features
2. **API layer** - REST/gRPC інтерфейс
3. **Database layer** - збереження метаданих контейнерів
4. **Monitoring** - метрики та логування
5. **Registry** - централізоване сховище контейнерів