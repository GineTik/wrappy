# Система прив'язок та інтеграції (BINDS)

## Варіанти інтеграції бінарників

### 1. Symlinks в /usr/bin (класичний підхід)

```bash
# При встановленні контейнера
ln -s /containers/nvim/bin/nvim /usr/bin/nvim
ln -s /containers/firefox/bin/firefox /usr/bin/firefox

# При видаленні
rm /usr/bin/nvim /usr/bin/firefox
```

**Плюси:**
- Звичний користувацький досвід
- Працює з будь-якими скриптами/програмами
- Автокомпліт в shell

**Мінуси:**
- Потрібні root права
- Конфлікти імен (два контейнери з nvim)
- Засмічення системної папки
- Складність очищення

### 2. PATH management (рекомендую!)

```bash
# ~/.wrappyrc або /etc/wrappy/config
export PATH="/containers/nvim/bin:/containers/firefox/bin:$PATH"

# Або динамічно через wrappy wrapper
wrappy activate nvim  # додає /containers/nvim/bin в PATH
wrappy deactivate nvim # видаляє з PATH
```

**Плюси:**
- Не потрібні root права
- Чиста системна папка
- Простість управління
- Можливість версіонування (nvim-0.9, nvim-0.10)

**Мінуси:**
- Потрібно налаштувати shell
- Може не працювати з деякими GUI додатками

### 3. Wrapper scripts (гібридний підхід)

```bash
# /usr/local/bin/nvim (або ~/.local/bin/nvim)
#!/bin/bash
exec /containers/nvim/bin/nvim "$@"
```

## Конфігурація та дані

### ~/.config лінкування

```rust
pub struct ConfigBinding {
    pub container_path: PathBuf,    // /containers/nvim/config
    pub host_path: PathBuf,         // ~/.config/nvim  
    pub binding_type: BindingType,
}

pub enum BindingType {
    Symlink,        // ln -s 
    BindMount,      // mount --bind
    Copy,           // cp -r (для ізоляції)
}
```

### Приклади прив'язок

```toml
# container/config/bindings.toml
[bindings]
# Бінарники
[[bindings.bin]]
source = "bin/nvim"
target = "~/.local/bin/nvim" 
type = "symlink"

[[bindings.bin]]  
source = "bin/hyprland"
target = "~/.local/bin/hyprland"
type = "wrapper_script"

# Конфігурація
[[bindings.config]]
source = "config/nvim"
target = "~/.config/nvim"
type = "symlink"

[[bindings.config]]
source = "config/hyprland" 
target = "~/.config/hypr"
type = "bind_mount"  # для ізоляції

# Дані
[[bindings.data]]
source = "data/nvim"
target = "~/.local/share/nvim"
type = "symlink"
```

## Рекомендована архітектура

```rust
// src/features/bindings/mod.rs
pub struct BindingManager {
    user_bin_dir: PathBuf,      // ~/.local/bin
    system_bin_dir: PathBuf,    // /usr/local/bin  
    config_dir: PathBuf,        // ~/.config
}

impl BindingManager {
    pub fn install_bindings(&self, container: &Container) -> Result<()> {
        let bindings = container.manifest.bindings.as_ref()
            .unwrap_or(&default_bindings());
            
        for binding in bindings {
            match binding.target_scope {
                TargetScope::User => {
                    self.create_user_binding(binding)?;
                }
                TargetScope::System => {
                    self.create_system_binding(binding)?; // потрібен sudo
                }
            }
        }
        
        // Оновлюємо PATH для поточної сесії
        self.update_path_for_session(container)?;
        
        Ok(())
    }
    
    fn create_user_binding(&self, binding: &Binding) -> Result<()> {
        let source = &binding.container_path;
        let target = self.resolve_target_path(&binding.target)?;
        
        match binding.binding_type {
            BindingType::Symlink => {
                std::fs::create_dir_all(target.parent().unwrap())?;
                std::os::unix::fs::symlink(source, target)?;
            }
            BindingType::WrapperScript => {
                self.create_wrapper_script(source, target)?;
            }
            BindingType::PathEntry => {
                self.add_to_user_path(source)?;
            }
        }
        
        Ok(())
    }
    
    fn create_wrapper_script(&self, source: &Path, target: &Path) -> Result<()> {
        let script_content = format!(
            "#!/bin/bash\nexec {} \"$@\"", 
            source.display()
        );
        
        std::fs::write(target, script_content)?;
        
        // Зробити виконуваним
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(target)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(target, perms)?;
        
        Ok(())
    }
}
```

## Маніфест з прив'язками

```json
{
  "name": "nvim-deluxe",
  "version": "0.10.0",
  "bindings": {
    "executables": [
      {
        "source": "bin/nvim",
        "target": "~/.local/bin/nvim",
        "type": "wrapper_script",
        "priority": 10
      }
    ],
    "configs": [
      {
        "source": "config/nvim", 
        "target": "~/.config/nvim",
        "type": "symlink",
        "backup_existing": true
      }
    ],
    "path_entries": [
      {
        "directory": "bin",
        "priority": 100
      },
      {
        "directory": "scripts", 
        "priority": 50
      }
    ]
  }
}
```

## PATH Management система

```bash
# ~/.wrappy/paths.d/nvim
export PATH="/containers/nvim/bin:$PATH"

# ~/.wrappy/paths.d/firefox  
export PATH="/containers/firefox/bin:$PATH"

# ~/.bashrc або ~/.zshrc
for path_file in ~/.wrappy/paths.d/*; do
    [ -f "$path_file" ] && source "$path_file"
done
```

## CLI команди

```bash
# Встановлення з прив'язками
wrappy install nvim-deluxe --bind-all
wrappy install nvim-deluxe --bind-bin --bind-config

# Управління прив'язками 
wrappy bind nvim-deluxe --enable bin,config
wrappy bind nvim-deluxe --disable config
wrappy unbind nvim-deluxe --all

# Перегляд прив'язок
wrappy list --bindings
wrappy show nvim-deluxe --bindings
```

## Конфлікт resolution

```rust
pub enum ConflictResolution {
    Skip,           // Пропустити якщо існує
    Backup,         // Зробити бекап існуючого  
    Overwrite,      // Перезаписати
    Prompt,         // Запитати користувача
    Priority,       // По пріоритету контейнера
}

impl BindingManager {
    fn resolve_conflict(&self, target: &Path, resolution: ConflictResolution) -> Result<()> {
        if !target.exists() {
            return Ok(());
        }
        
        match resolution {
            ConflictResolution::Backup => {
                let backup_path = format!("{}.wrappy-backup", target.display());
                std::fs::rename(target, backup_path)?;
            }
            ConflictResolution::Skip => {
                return Err("Target already exists".into());
            }
            // ...
        }
        
        Ok(())
    }
}
```

## Рекомендації

**Для MVP:**
1. **PATH management** - найпростіший та найбезпечніший
2. **~/.local/bin wrapper scripts** - для кращої інтеграції  
3. **Symlinks для config** - але з backup'ом

**Архітектура:**
```
~/.wrappy/
├── paths.d/          # PATH entries для кожного контейнера
├── bindings/         # Метадані прив'язок
└── backups/          # Бекапи перезаписаних файлів

~/.local/bin/         # Wrapper scripts
~/.config/            # Symlinks на конфіги контейнерів
```

## Технічні деталі

### Автоматичне виявлення бінарників

```rust
impl Container {
    pub fn discover_bindings(&self) -> Result<Vec<Binding>> {
        let mut bindings = Vec::new();
        
        // Сканування bin/ директорії
        let bin_dir = self.path.join("bin");
        if bin_dir.exists() {
            for entry in std::fs::read_dir(&bin_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    let binding = Binding {
                        source: entry.path(),
                        target: format!("~/.local/bin/{}", entry.file_name().to_string_lossy()),
                        binding_type: BindingType::WrapperScript,
                        priority: 50,
                    };
                    bindings.push(binding);
                }
            }
        }
        
        // Сканування config/ директорії
        let config_dir = self.path.join("config");
        if config_dir.exists() {
            for entry in std::fs::read_dir(&config_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    let binding = Binding {
                        source: entry.path(),
                        target: format!("~/.config/{}", entry.file_name().to_string_lossy()),
                        binding_type: BindingType::Symlink,
                        priority: 50,
                    };
                    bindings.push(binding);
                }
            }
        }
        
        Ok(bindings)
    }
}
```

### Shell інтеграція

```bash
# ~/.wrappy/shell-integration.sh
wrappy_activate() {
    local container_name="$1"
    local container_path="/containers/$container_name"
    
    if [ -d "$container_path/bin" ]; then
        export PATH="$container_path/bin:$PATH"
        echo "Activated container: $container_name"
    fi
}

wrappy_deactivate() {
    local container_name="$1" 
    local container_path="/containers/$container_name"
    
    # Видалити з PATH
    export PATH=$(echo "$PATH" | sed "s|$container_path/bin:||g")
    echo "Deactivated container: $container_name"
}

# Автокомпліт
_wrappy_containers() {
    local containers=$(ls /containers 2>/dev/null)
    COMPREPLY=($(compgen -W "$containers" -- "${COMP_WORDS[1]}"))
}

complete -F _wrappy_containers wrappy_activate
complete -F _wrappy_containers wrappy_deactivate
```

### Desktop файли

```rust
impl BindingManager {
    pub fn create_desktop_entries(&self, container: &Container) -> Result<()> {
        let desktop_dir = dirs::home_dir()
            .unwrap()
            .join(".local/share/applications");
        
        std::fs::create_dir_all(&desktop_dir)?;
        
        for app in &container.manifest.applications {
            let desktop_content = format!(
                "[Desktop Entry]\n\
                 Type=Application\n\
                 Name={}\n\
                 Exec=/containers/{}/bin/{}\n\
                 Icon=/containers/{}/icons/{}.png\n\
                 Categories={}\n\
                 Comment={}\n",
                app.name,
                container.name(),
                app.executable,
                container.name(),
                app.name.to_lowercase(),
                app.categories.join(";"),
                app.description
            );
            
            let desktop_file = desktop_dir.join(format!("{}.desktop", app.name.to_lowercase()));
            std::fs::write(desktop_file, desktop_content)?;
        }
        
        Ok(())
    }
}
```