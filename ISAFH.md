# Ізоляція та Flathub інтеграція (ISAFH)

## 1. Ізоляція без VM та з мінімальними накладами

### Linux Namespaces (найкращий варіант)
```bash
# Mount namespace - ізоляція файлової системи
unshare --mount --pid --fork chroot /path/to/container

# Network namespace - ізоляція мережі  
unshare --net

# User namespace - ізоляція користувачів
unshare --user --map-root-user
```

**Переваги:**
- Нативна підтримка в Linux kernel
- Мінімальні накладні витрати
- Реальна ізоляція процесів, файлів, мережі

### Chroot + cgroups
```bash
# Ізоляція файлової системи
chroot /container/root /bin/bash

# Обмеження ресурсів
echo $PID > /sys/fs/cgroup/memory/container/cgroup.procs
```

### LD_PRELOAD хуки (для часткової ізоляції)
```c
// Перехоплення системних викликів
int open(const char *pathname, int flags) {
    // Перенаправлення шляхів в контейнер
    char *real_path = remap_path(pathname);
    return real_open(real_path, flags);
}
```

## 2. Інтеграція з Flathub

### Архітектура адаптера
```rust
// src/features/flathub/mod.rs
pub struct FlathubAdapter {
    client: HttpClient,
    cache_dir: PathBuf,
}

impl FlathubAdapter {
    pub async fn download_app(&self, app_id: &str) -> Result<FlatpakBundle> {
        // 1. Запит метаданих з Flathub API
        let metadata = self.fetch_metadata(app_id).await?;
        
        // 2. Скачування .flatpak файлу
        let bundle = self.download_bundle(&metadata.download_url).await?;
        
        // 3. Аналіз залежностей
        let deps = self.analyze_dependencies(&bundle)?;
        
        Ok(bundle)
    }
    
    pub fn convert_to_wrappy(&self, bundle: FlatpakBundle) -> ContainerManifest {
        // Конвертація Flatpak в наш формат
    }
}
```

### Обробка залежностей Flatpak

**Стратегія 1: Плоска структура**
```json
{
  "dependencies": [
    "org.freedesktop.Platform//22.08",
    "org.freedesktop.Sdk//22.08", 
    "org.gtk.Gtk3theme.Adwaita-dark"
  ]
}
```

**Стратегія 2: Вкладені контейнери**
```
wrappy-firefox/
├── content/           # Firefox файли
├── deps/
│   ├── platform/      # Freedesktop Platform
│   ├── gtk3/          # GTK3 runtime
│   └── webkit/        # WebKit runtime
```

## 3. Комплексне рішення ізоляції

### Поетапна ізоляція
```rust
pub enum IsolationLevel {
    None,           // Тільки PATH підміна
    Partial,        // chroot + basic namespaces  
    Full,           // повні namespaces + cgroups
    Paranoid,       // + seccomp + AppArmor
}

pub struct ContainerRuntime {
    isolation: IsolationLevel,
    root_fs: PathBuf,
    bind_mounts: Vec<BindMount>,
}

impl ContainerRuntime {
    pub fn create_sandbox(&self) -> Result<Sandbox> {
        match self.isolation {
            IsolationLevel::Partial => {
                // chroot + mount namespace
                self.setup_chroot()?;
                self.setup_mount_namespace()?;
            }
            IsolationLevel::Full => {
                // + PID, NET, USER namespaces
                self.setup_full_namespaces()?;
                self.setup_cgroups()?;
            }
            // ...
        }
    }
}
```

### Файлова система контейнера
```
container/
├── merged/            # Overlay mount point
├── layers/
│   ├── base/          # Base system files
│   ├── deps/          # Dependencies layer  
│   └── app/           # Application layer
├── work/              # Overlay work dir
├── scripts/
└── config/
```

## 4. Flathub інтеграція - практична реалізація

### CLI команда
```bash
wrappy install --from-flathub org.mozilla.firefox
wrappy install --from-flathub com.spotify.Client
```

### Процес конвертації
```rust
pub async fn install_from_flathub(app_id: &str) -> ContainerResult<()> {
    // 1. Скачування
    let bundle = flathub_adapter.download_app(app_id).await?;
    
    // 2. Аналіз метаданих
    let flatpak_info = bundle.parse_metadata()?;
    
    // 3. Створення wrappy-контейнера
    let manifest = ContainerManifest {
        name: flatpak_info.name,
        version: Version::new(&flatpak_info.version)?,
        dependencies: convert_flatpak_deps(&flatpak_info.dependencies),
        scripts: create_launcher_scripts(&flatpak_info),
        isolation: IsolationConfig {
            level: IsolationLevel::Partial,
            bind_mounts: flatpak_info.required_paths,
        }
    };
    
    // 4. Підготовка файлової системи  
    setup_container_fs(&manifest, &bundle)?;
    
    // 5. Створення launcher скриптів
    create_flatpak_launcher(&manifest)?;
}
```

### Launcher скрипт
```bash
#!/bin/bash
# scripts/default.sh

# Активація нашої ізоляції
export WRAPPY_CONTAINER_ROOT="/containers/firefox"  
export PATH="$WRAPPY_CONTAINER_ROOT/bin:$PATH"

# Запуск через flatpak runtime, але в нашій ізоляції
exec flatpak run --filesystem="$WRAPPY_CONTAINER_ROOT" \
     --env=WRAPPY_ISOLATED=1 \
     org.mozilla.firefox "$@"
```

## 5. Розв'язання проблем залежностей

### Стратегія "Runtime Sharing"
```rust
pub struct DependencyManager {
    shared_runtimes: HashMap<String, PathBuf>,
}

impl DependencyManager {
    pub fn resolve_dependencies(&self, deps: &[Dependency]) -> Result<Vec<BindMount>> {
        let mut mounts = Vec::new();
        
        for dep in deps {
            if let Some(runtime_path) = self.shared_runtimes.get(&dep.name) {
                // Переиспользуем існуючий runtime
                mounts.push(BindMount {
                    source: runtime_path.clone(),
                    target: format!("/usr/lib/{}", dep.name),
                    readonly: true,
                });
            } else {
                // Скачуємо новий runtime
                self.download_runtime(dep)?;
            }
        }
        
        Ok(mounts)
    }
}
```

## Висновок та рекомендації

**Для MVP рекомендую:**

1. **Ізоляція**: Почати з `chroot + mount namespaces` - простота + ефективність
2. **Flathub**: Створити адаптер який просто перепакує flatpak у wrappy формат
3. **Залежності**: Спільні runtime через bind mounts

**Архітектура:**
```
wrappy
├── isolation/         # Namespace + chroot логіка  
├── adapters/         
│   ├── flathub/      # Flathub інтеграція
│   ├── appimage/     # AppImage підтримка
│   └── snap/         # Snap підтримка  
└── runtime/          # Shared runtimes cache
```

## Технічні деталі реалізації

### Namespace створення
```rust
use libc::{unshare, CLONE_NEWNS, CLONE_NEWPID, CLONE_NEWUSER};

pub fn create_isolated_environment() -> Result<()> {
    unsafe {
        // Створення mount namespace
        if unshare(CLONE_NEWNS) != 0 {
            return Err("Failed to create mount namespace");
        }
        
        // Створення PID namespace  
        if unshare(CLONE_NEWPID) != 0 {
            return Err("Failed to create PID namespace");
        }
        
        // Створення user namespace
        if unshare(CLONE_NEWUSER) != 0 {
            return Err("Failed to create user namespace");
        }
    }
    
    Ok(())
}
```

### Mount точки для ізоляції
```rust
pub fn setup_container_mounts(container_root: &Path) -> Result<()> {
    // Bind mount контейнера
    mount_bind(container_root, "/container")?;
    
    // Приватний /tmp
    mount_tmpfs("/container/tmp")?;
    
    // Readonly system directories
    mount_bind_ro("/usr", "/container/usr")?;
    mount_bind_ro("/lib", "/container/lib")?;
    
    // Chroot в контейнер
    chroot("/container")?;
    chdir("/")?;
    
    Ok(())
}
```

### Flathub API інтеграція
```rust
pub struct FlathubClient {
    client: reqwest::Client,
    base_url: String,
}

impl FlathubClient {
    pub async fn search_app(&self, query: &str) -> Result<Vec<AppInfo>> {
        let url = format!("{}/api/v1/apps?query={}", self.base_url, query);
        let response = self.client.get(&url).send().await?;
        let apps: Vec<AppInfo> = response.json().await?;
        Ok(apps)
    }
    
    pub async fn get_app_details(&self, app_id: &str) -> Result<AppDetails> {
        let url = format!("{}/api/v1/apps/{}", self.base_url, app_id);
        let response = self.client.get(&url).send().await?;
        let details: AppDetails = response.json().await?;
        Ok(details)
    }
    
    pub async fn download_flatpak(&self, app_id: &str, arch: &str) -> Result<Vec<u8>> {
        let url = format!("{}/repo/appstream/{}.flatpakref", self.base_url, app_id);
        let response = self.client.get(&url).send().await?;
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}
```