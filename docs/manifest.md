# Container Manifest - Документація

## Загальний опис

Маніфест контейнера (`container.json`) - це центральний конфігураційний файл, який описує всі аспекти контейнера: метадані, залежності, скрипти виконання та налаштування біндингів.

## Структура маніфеста

### Базовий приклад

```json
{
  "name": "my-application",
  "version": "1.0.0",
  "description": "My awesome containerized application",
  "author": "Developer Name",
  "scripts": {
    "default": "scripts/default.sh",
    "install": "scripts/install.sh",
    "test": "scripts/test.sh"
  },
  "dependencies": [
    {
      "name": "base-runtime",
      "version": "2.1.0",
      "optional": false
    }
  ],
  "environment": {
    "NODE_ENV": "production",
    "LOG_LEVEL": "info"
  },
  "bindings": {
    "executables": [],
    "configs": [],
    "data": []
  }
}
```

## Поля маніфеста

### Обов'язкові поля

#### `name` (string)
Унікальне ім'я контейнера в системі.

**Правила валідації:**
- Не може бути порожнім
- Може містити лише алфавітно-цифрові символи, дефіси (-) та підкреслення (_)
- Використовується для ідентифікації контейнера в системі

**Приклади:**
```json
"name": "web-server"           // ✅ Правильно
"name": "my_app_v2"           // ✅ Правильно
"name": "app@latest"          // ❌ Неправильно (містить @)
"name": ""                    // ❌ Неправильно (порожнє)
```

#### `version` (string)
Семантична версія контейнера у форматі SemVer.

**Формат:** `MAJOR.MINOR.PATCH` або `MAJOR.MINOR.PATCH-PRERELEASE`

**Приклади:**
```json
"version": "1.0.0"            // ✅ Стабільна версія
"version": "2.1.3"            // ✅ Патч версія
"version": "1.0.0-alpha.1"    // ✅ Пре-реліз
"version": "1.0"              // ❌ Неправильно (неповний формат)
```

#### `scripts` (object)
Колекція скриптів для різних операцій з контейнером.

**Обов'язковий скрипт:** `default` - основний скрипт виконання контейнера.

**Структура:**
```json
"scripts": {
  "default": "scripts/default.sh",    // Обов'язковий
  "install": "scripts/install.sh",    // Опціональний
  "test": "scripts/test.sh",          // Опціональний
  "cleanup": "scripts/cleanup.sh"     // Опціональний
}
```

**Правила:**
- Скрипт `default` завжди має бути присутнім
- Шляхи скриптів відносні до кореня контейнера
- Шляхи не можуть бути порожніми

### Опціональні поля

#### `description` (string, default: "")
Опис призначення та функціональності контейнера.

```json
"description": "Web server with Node.js runtime and nginx proxy"
```

#### `author` (string, default: "")
Автор або команда розробників контейнера.

```json
"author": "John Doe <john@example.com>"
```

#### `dependencies` (array, default: [])
Масив залежностей від інших контейнерів.

**Структура залежності:**
```json
{
  "name": "dependency-name",
  "version": "1.0.0",
  "optional": false
}
```

**Поля залежності:**
- `name` - ім'я контейнера-залежності (обов'язкове, не порожнє)
- `version` - версія у форматі SemVer (обов'язкове, не порожнє)
- `optional` - чи є залежність опціональною (default: false)

**Приклад:**
```json
"dependencies": [
  {
    "name": "database-postgres",
    "version": "13.2.0",
    "optional": false
  },
  {
    "name": "cache-redis",
    "version": "6.0.0",
    "optional": true
  }
]
```

#### `environment` (object, default: {})
Змінні середовища для контейнера.

```json
"environment": {
  "NODE_ENV": "production",
  "PORT": "3000",
  "DATABASE_URL": "postgresql://localhost:5432/mydb",
  "ENABLE_LOGGING": "true"
}
```

#### `bindings` (object, default: {})
Конфігурація біндингів для інтеграції з хост-системою.

Детальний опис дивіться в [документації біндингів](bindings.md).

```json
"bindings": {
  "executables": [
    {
      "source": "bin/app",
      "target": "~/.local/bin/myapp",
      "binding_type": "wrapper",
      "display_name": "My Application"
    }
  ],
  "configs": [
    {
      "source": "config",
      "target": "~/.config/myapp",
      "binding_type": "symlink",
      "backup_existing": true
    }
  ],
  "data": [
    {
      "source": "data",
      "target": "~/.local/share/myapp",
      "binding_type": "symlink",
      "backup_existing": false
    }
  ]
}
```

## Валідація маніфеста

Система автоматично валідує маніфест при завантаженні та збереженні.

### Правила валідації

1. **Ім'я контейнера:**
   - Не порожнє
   - Містить лише дозволені символи (a-z, A-Z, 0-9, -, _)

2. **Версія:**
   - Відповідає формату SemVer
   - Валідується через парсер версій

3. **Скрипти:**
   - Обов'язковий скрипт `default`
   - Усі шляхи не порожні

4. **Залежності:**
   - Імена не порожні
   - Версії у форматі SemVer
   - Версії можуть бути парсировані

### Помилки валідації

Типові помилки та їх значення:

```rust
// Порожнє ім'я
ContainerError::ManifestValidation("Container name cannot be empty")

// Неправильні символи в імені
ContainerError::ManifestValidation("Container name can only contain alphanumeric characters, hyphens, and underscores")

// Відсутній default скрипт
ContainerError::MissingDefaultScript

// Порожній шлях скрипта
ContainerError::ManifestValidation("Script 'script-name' has empty path")

// Порожнє ім'я залежності
ContainerError::InvalidDependency { 
    package: "", 
    reason: "Dependency name cannot be empty" 
}

// Неправильна версія залежності
ContainerError::InvalidDependency { 
    package: "dep-name", 
    reason: "Invalid version format: invalid-version" 
}
```

## Робота з маніфестом

### Створення нового маніфеста

```rust
use wrappy::ContainerManifest;
use wrappy::Version;

let manifest = ContainerManifest::new(
    "my-app".to_string(),
    Version::parse("1.0.0").unwrap()
);
```

### Завантаження з файлу

```rust
let manifest = ContainerManifest::from_file("container.json")?;
```

### Збереження у файл

```rust
manifest.to_file("container.json")?;
```

### Додавання скриптів

```rust
manifest.add_script("build".to_string(), "scripts/build.sh".to_string());
```

### Додавання залежностей

```rust
use wrappy::Dependency;

let dependency = Dependency {
    name: "base-runtime".to_string(),
    version: "1.0.0".to_string(),
    optional: false,
};

manifest.add_dependency(dependency);
```

## Приклади маніфестів

### Простий додаток

```json
{
  "name": "hello-world",
  "version": "1.0.0",
  "description": "Simple hello world application",
  "author": "Developer",
  "scripts": {
    "default": "scripts/default.sh"
  },
  "bindings": {
    "executables": [
      {
        "source": "bin/hello",
        "target": "~/.local/bin/hello"
      }
    ]
  }
}
```

### Веб-сервер з залежностями

```json
{
  "name": "web-server",
  "version": "2.1.0",
  "description": "Production web server with database support",
  "author": "Web Team <team@company.com>",
  "scripts": {
    "default": "scripts/start.sh",
    "install": "scripts/install.sh",
    "migrate": "scripts/migrate.sh",
    "test": "scripts/test.sh"
  },
  "dependencies": [
    {
      "name": "postgresql-client",
      "version": "13.0.0",
      "optional": false
    },
    {
      "name": "monitoring-tools",
      "version": "1.2.0",
      "optional": true
    }
  ],
  "environment": {
    "NODE_ENV": "production",
    "PORT": "8080",
    "LOG_LEVEL": "info"
  },
  "bindings": {
    "executables": [
      {
        "source": "bin/server",
        "target": "~/.local/bin/webserver",
        "binding_type": "wrapper",
        "display_name": "Web Server"
      },
      {
        "source": "bin/admin",
        "target": "~/.local/bin/webserver-admin",
        "binding_type": "wrapper",
        "display_name": "Admin Tools"
      }
    ],
    "configs": [
      {
        "source": "config",
        "target": "~/.config/webserver",
        "binding_type": "symlink",
        "backup_existing": true
      }
    ],
    "data": [
      {
        "source": "data",
        "target": "~/.local/share/webserver",
        "binding_type": "symlink",
        "backup_existing": false
      }
    ]
  }
}
```

### Інструмент розробки

```json
{
  "name": "dev-tools",
  "version": "3.2.1",
  "description": "Development tools and utilities",
  "author": "DevOps Team",
  "scripts": {
    "default": "scripts/default.sh",
    "setup": "scripts/setup.sh",
    "update": "scripts/update.sh"
  },
  "dependencies": [
    {
      "name": "git-tools",
      "version": "2.0.0",
      "optional": false
    }
  ],
  "environment": {
    "EDITOR": "vim",
    "PAGER": "less"
  },
  "bindings": {
    "executables": [
      {
        "source": "bin/build",
        "target": "~/.local/bin/build",
        "binding_type": "wrapper"
      },
      {
        "source": "bin/deploy",
        "target": "~/.local/bin/deploy",
        "binding_type": "wrapper"
      },
      {
        "source": "bin/lint",
        "target": "~/.local/bin/lint",
        "binding_type": "symlink"
      }
    ],
    "configs": [
      {
        "source": "config/editor",
        "target": "~/.config/dev-tools/editor",
        "binding_type": "copy",
        "backup_existing": true
      }
    ]
  }
}
```

## Міграція та версіонування

При зміні структури маніфеста система підтримує зворотну сумісність:

1. **Додавання нових полів** - старі маніфести продовжують працювати
2. **Зміна значень за замовчуванням** - явно вказані значення зберігаються
3. **Валідація** - нові правила застосовуються тільки при збереженні

## Найкращі практики

1. **Використовуйте семантичне версіонування** для контролю сумісності
2. **Додавайте описові поля** (description, author) для документування
3. **Групуйте логічно пов'язані скрипти** (install, test, cleanup)
4. **Визначайте мінімальні необхідні залежності** замість включення всього
5. **Використовуйте змінні середовища** для конфігурації замість жорсткого кодування
6. **Плануйте біндинги заздалегідь** для зручності користувачів