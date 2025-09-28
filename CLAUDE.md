# Claude Code Instructions for Wrappy Project

## Developer Context

**Знання мов програмування:**

- ✅ C++, C#, JavaScript/TypeScript, Dart
- ❌ Rust (новачок)

**Важливо:** Завжди пояснюй Rust-специфічні концепції, синтаксис та ідіоми. Використовуй аналогії з відомими мовами де це можливо.

## Language Requirements

**ОБОВ'ЯЗКОВО:** Весь код має бути написаний англійською мовою:

- Назви змінних, функцій, структур, енумів
- Коментарі в коді
- Документація (doc comments)
- Повідомлення в помилках
- Назви файлів та модулів

**Винятки:** README.md та інші markdown файли можуть бути українською для кращого розуміння концепції.

## Architecture and file structure

Проєкт має 2 основних вхідних файли:
- `main.rs` - вхідна точка додатку
- `lib.rs` - описує модулі додатку

Крім цього додаток поділений на 2 основних модуля:
- `features` - фічі проєкту, які являють собою папки зі своєю структурою, містить нашу бізнес логіку та IO команди
- `core` - shared модулі, які можуть бути як папкою з `mod.rs` файлом, так і звичаним раст файлом, містить інфраструктурну логіку та часто використовувану логіку по фічам

Feature модуль структурується наступним чином:
- `mod.rs` - лише оголошення модулів, їх експорт тощо
- `commands.rs` - команди для обробки IO операцій (в нашому проєкті операцій з терміналом), команди також займаються мапингом вхідних даних
- `service.rs` - містить бізнес логіку та керує потоком даних (кординує Repository, Core тощо)
- `types.rs`
- `validators.rs` - містить логіку валідування (необов'язково)
- `mappers.rs` - містить логіку мапінгу з одного в інше (необов'язково)

Модуль може містити підмодулі, які мають ідентичну структуру. Також модулі можуть містити папки для групування чогось, якщо файли занадто великі і їх треба декомпозувати.

## Code Style Guidelines

### 1. Іменування

**Добре:**

```rust
// Функції та змінні: snake_case
fn create_container() {}
let container_name = "example";

// Структури та енуми: PascalCase
struct ContainerManifest {}
enum ContainerType {}

// Константи: SCREAMING_SNAKE_CASE
const MAX_CONTAINER_SIZE: usize = 1024;

// Модулі: snake_case
mod container_manager;
```

**Погано:**

```rust
// Неправильне іменування
fn CreateContainer() {}  // має бути snake_case
let containerName = "example";  // має бути snake_case
struct containerManifest {}  // має бути PascalCase
const maxContainerSize: usize = 1024;  // має бути SCREAMING_SNAKE_CASE
```

### 2. Обробка помилок

**Добре:**

```rust
// Використовуй Result для операцій що можуть провалитись
fn load_container(path: &Path) -> Result<Container, ContainerError> {
    let manifest = fs::read_to_string(path)
        .map_err(ContainerError::IoError)?;

    serde_json::from_str(&manifest)
        .map_err(ContainerError::ParseError)
}

// Кастомні типи помилок
#[derive(Debug)]
enum ContainerError {
    IoError(std::io::Error),
    ParseError(serde_json::Error),
    InvalidManifest(String),
}
```

**Погано:**

```rust
// Уникай panic! у продакшен коді
fn load_container(path: &Path) -> Container {
    let manifest = fs::read_to_string(path).unwrap();  // ❌ panic на помилці
    serde_json::from_str(&manifest).expect("Bad JSON")  // ❌ panic з повідомленням
}

// Не використовуй загальні типи помилок
fn load_container(path: &Path) -> Result<Container, Box<dyn Error>> {  // ❌ занадто загальне
    // ...
}
```

### 3. Ownership та Borrowing

**Добре:**

```rust
// Використовуй references для читання
fn validate_manifest(manifest: &ContainerManifest) -> bool {
    manifest.name.len() > 0 && manifest.version.is_valid()
}

// Повертай owned values коли потрібно
fn create_default_manifest(name: String) -> ContainerManifest {
    ContainerManifest {
        name,
        version: Version::new("1.0.0").unwrap(),
        dependencies: Vec::new(),
    }
}

// Використовуй Clone тільки коли необхідно
fn backup_manifest(manifest: &ContainerManifest) -> ContainerManifest {
    manifest.clone()  // Явно клонуємо коли потрібна копія
}
```

**Погано:**

```rust
// Не передавай ownership без потреби
fn validate_manifest(manifest: ContainerManifest) -> bool {  // ❌ забирає ownership
    manifest.name.len() > 0
}

// Уникай зайвого клонування
fn get_container_name(manifest: &ContainerManifest) -> String {
    manifest.name.clone()  // ❌ клонування для простого читання
}

// Краще повернути reference
fn get_container_name(manifest: &ContainerManifest) -> &str {
    &manifest.name
}
```

### 4. Структура модулів

**Добре:**

```rust
// src/lib.rs
pub mod core;
pub mod manager;
pub mod cli;

pub use core::{Container, ContainerError};
pub use manager::ContainerManager;

// src/core/mod.rs
mod container;
mod manifest;
mod validation;

pub use container::Container;
pub use manifest::ContainerManifest;
pub use validation::validate_container;

// Приватні implementation details
use validation::internal_validate;
```

**Погано:**

```rust
// Занадто глибока вкладеність
// src/core/container/management/runtime/execution/mod.rs  // ❌

// Експорт усього
pub use container::*;  // ❌ засмічує namespace

// Відсутність модульної організації
// Весь код в main.rs  // ❌
```

### 5. Документація

**Коли потрібна документація:**
- Складні функції та методи з неочевидною логікою
- Публічні API функції, які використовуватимуться зовні
- Незрозумілі поля структур та енумів
- Функції з потенціалом для помилок

**Фокус на "навіщо", а не "що":**
- Поясни **для чого** функція потрібна в контексті системи
- Опиши її **роль** у життєвому циклі
- Уникай очевидних описів

**Добре:**

```rust
/// Tracks container runtime state for lifecycle management and user reporting.
/// Enables monitoring execution status, process information, and error history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerRuntime {
    pub id: Uuid,
    pub status: ContainerStatus,
    pub pid: Option<u32>,
    // ... other fields don't need individual docs if obvious
}

/// Validates container directory structure to ensure proper deployment.
/// Prevents runtime failures by catching missing dependencies early.
fn validate_structure(path: &Path, manifest: &ContainerManifest) -> ContainerResult<()> {
    // implementation
}

/// Initializes container with validated structure and runtime state.
/// Core factory method for creating deployable container instances.
pub fn new(manifest: ContainerManifest, path: PathBuf) -> ContainerResult<Self> {
    // implementation
}

// Simple getters don't need documentation
pub fn name(&self) -> &str {
    &self.manifest.name
}
```

**Погано:**

```rust
// Over-documentation for obvious things ❌
/// Returns the container name.
/// 
/// # Arguments
/// * `self` - Reference to container
/// 
/// # Returns
/// Returns string slice containing the name
/// 
/// # Examples
/// ```
/// let name = container.name();
/// ```
pub fn name(&self) -> &str {
    &self.manifest.name
}

// Describing "what" instead of "why" ❌
/// Container status enum with different states
pub enum ContainerStatus { ... }

// Ukrainian documentation ❌
/// Створює контейнер з маніфестом
pub fn create_container(/* ... */) {
}

// Obvious field documentation ❌
pub struct Container {
    /// Container manifest  // ❌ obvious from type
    pub manifest: ContainerManifest,
    /// Container path      // ❌ obvious from type  
    pub path: PathBuf,
}
```

**Правила документації:**
- **Документуй навіщо, не що** - роль у системі, а не опис коду
- **Короткий доцільний опис** - одне-два речення про призначення
- **Arguments/Examples тільки при крайній необхідності** - для складних випадків
- **Не документуй очевидне** - прості getters, типові поля
- **Фокус на бізнес-логіці** - як це допомагає користувачу

### 6. Тести

**Добре:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_container_success() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let manifest = ContainerManifest::new("test-app");

        // Act
        let result = Container::create(manifest, temp_dir.path());

        // Assert
        assert!(result.is_ok());
        let container = result.unwrap();
        assert_eq!(container.name(), "test-app");
    }

    #[test]
    fn test_create_container_invalid_path() {
        // Arrange
        let manifest = ContainerManifest::new("test-app");
        let invalid_path = Path::new("/nonexistent/path");

        // Act
        let result = Container::create(manifest, invalid_path);

        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ContainerError::IoError(_)));
    }
}
```

**Погано:**

```rust
#[test]
fn test() {  // ❌ неінформативна назва
    let c = Container::create(/* ... */);  // ❌ неясні змінні
    assert!(c.is_ok());  // ❌ без контексту
}

#[test]
fn test_create_container() {
    // ❌ тест на все одразу, важко зрозуміти що саме провалилось
    let result1 = Container::create(/* case 1 */);
    let result2 = Container::create(/* case 2 */);
    let result3 = Container::create(/* case 3 */);
    assert!(result1.is_ok() && result2.is_err() && result3.is_ok());
}
```

### 7. Dependency Management

**Добре:**

```rust
// Cargo.toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
clap = { version = "4.0", features = ["derive"] }

[dev-dependencies]
tempfile = "3.0"
assert_matches = "1.5"

// Версії explicit та з features
// Групування dev-dependencies окремо
```

**Погано:**

```rust
// Cargo.toml
[dependencies]
serde = "*"  // ❌ небезпечна версія
tokio = "1"  // ❌ без необхідних features
unnecessary_crate = "2.0"  // ❌ невикористана залежність

// Відсутність dev-dependencies де потрібно
```

## Rust-специфічні поради

### Lifetime аннотації

```rust
// Поясни коли потрібні lifetimes
struct ContainerRef<'a> {
    name: &'a str,          // Reference має lifetime
    manifest: &'a Manifest, // Той самий lifetime
}
```

### Pattern Matching

```rust
// Використовуй match замість множинних if
match container_type {
    ContainerType::Application => handle_app(),
    ContainerType::Package => handle_package(),
    ContainerType::System => handle_system(),
}
```

### Iterator Chains

```rust
// Функціональний стиль замість циклів
let valid_containers: Vec<_> = containers
    .iter()
    .filter(|c| c.is_valid())
    .map(|c| c.name())
    .collect();
```
