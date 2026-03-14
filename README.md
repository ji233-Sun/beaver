# be

Turn complex shell commands into simple tags.

`be` lets you save frequently used shell commands as short tags, then run them with `be <tag>`. No more scrolling through shell history or maintaining scattered scripts.

## Install

```bash
brew install ji233-Sun/tap/be
```

Or build from source:

```bash
git clone https://github.com/ji233-Sun/beaver.git
cd beaver
cargo build --release
cp target/release/be /usr/local/bin/
```

## Quick Start

```bash
# Initialize in your project
be init

# Add commands
be add dev -c "npm run dev" -d "Start dev server"
be add deploy -c "docker build -t app . && docker push registry/app:latest" -d "Build and push"

# Run by tag
be dev
be deploy

# List all commands
be list
```

## Usage

```
be <tag>                              # Run a saved command
be init                               # Initialize .be directory
be list                               # List all commands
be add <tag> -c <cmd> [-d <desc>]     # Add a command
be remove <tag>                       # Remove a command
be edit <tag> [-c <cmd>] [-d <desc>]  # Edit a command
be manager                            # Open TUI manager
```

## TUI Manager

Run `be manager` to open an interactive terminal UI for managing commands.

Keybindings:
- `↑↓` / `jk` — Navigate
- `Enter` — Open action menu
- `a` — Add command
- `d` — Delete command
- `q` — Quit

## Config

Commands are stored in `.be/config.toml` (similar to `.git`, `be` searches upward from the current directory):

```toml
[commands.dev]
description = "Start dev server"
command = "npm run dev"

[commands.deploy]
description = "Build and push"
command = "docker build -t app . && docker push registry/app:latest"
```

## License

[MIT](LICENSE)
