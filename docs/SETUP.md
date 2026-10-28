# Настройка рабочей машины (Windows)

## 1. Инструменты

Всё ставится через `winget` из PowerShell.

```powershell
# Git
winget install --id Git.Git -e

# Компилятор и линкер MSVC + Windows SDK (нужны Rust для target x86_64-pc-windows-msvc)
winget install --id Microsoft.VisualStudio.2022.BuildTools -e --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

# Rust
winget install --id Rustlang.Rustup -e
```

После установки открыть **новый** терминал и проверить:

```powershell
rustc -vV      # host: x86_64-pc-windows-msvc
cargo --version
git --version
```

Версия Rust и компоненты (`rustfmt`, `clippy`) подтянутся автоматически из
`rust-toolchain.toml` при первом `cargo` в папке проекта.

### Желательно для отладки встраивания окон

- **Spy++** — смотреть дерево окон, стили, сообщения. Входит в Visual Studio
  Community с нагрузкой «Разработка классических приложений на C++»
  (в Build Tools его нет).
- Тестовые приложения для спайка: drawio desktop, Word, классический Блокнот.

## 2. Claude Code

```powershell
irm https://claude.ai/install.ps1 | iex
claude auth login        # войти тем же аккаунтом claude.ai, что и в облаке
```

Для работы Claude Code на Windows нужен Git for Windows (ставится выше).

## 3. Код

```powershell
git clone https://github.com/dimmik/wfoxd.git
cd wfoxd
```

Дальше один из двух вариантов.

**Продолжить облачную сессию со всей историей разговора** (подтянет ветку
сама, рабочая копия должна быть чистой):

```powershell
claude --teleport session_01NQ21Sdnwm2x8waEt497fPK
```

или просто `claude --teleport` и выбрать сессию из списка.

**Начать новую локальную сессию** — весь нужный контекст лежит в `CLAUDE.md`
и `docs/PLAN.md`:

```powershell
git checkout ccr-9493f92f-w0um25   # пока план не влит в main
claude
```

Например: «Сделай этап 0 из docs/PLAN.md».
