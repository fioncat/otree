# OTree - Object Tree TUI Viewer

![screenshot](assets/screenshot.png)

A command line tool to view objects (JSON/YAML/TOML) in TUI tree widget.

## Install

Download binary files from [release page](https://github.com/fioncat/otree/releases).

You can also build it from source (require `cargo` installed):

```bash
cargo install --git https://github.com/fioncat/otree
```

### Arch Linux (AUR)

You can install `otree` from the [AUR](https://aur.archlinux.org/packages/otree) with using an [AUR helper](https://wiki.archlinux.org/title/AUR_helpers).

```bash
paru -S otree
```

### macOS and Linux (Homebrew)

You can install [`otree`](https://formulae.brew.sh/formula/otree) using [Homebrew](https://brew.sh).

```bash
brew install otree
```

## Usage

Open a JSON/YAML/TOML file in TUI tree viewer:

```bash
otree /path/to/file.json
otree /path/to/file.yaml
otree /path/to/file.toml
```

For more command usage, please run `otree --help`.

You can configure TUI keys, colors, and more in `~/.config/otree.toml`, the default configuration is [here](config/default.toml).

For all available actions and their default key bindings, please refer to: [All Available Actions](docs/actions.md).

For how to configure TUI colors, please refer to: [Colors Document](docs/colors.md).

## Features

- [x] UI: Header (v0.1)
- [x] UI: Tree Overview (v0.1)
- [x] UI: Data Block (v0.1)
- [x] UI: Footer to show current root and identify (and other messages) (v0.2)
- [ ] UI: Filter Input
- [x] UI: Popup widget to show error or help messages (v0.2)
- [x] Action: Change current selected item as root (v0.1)
- [x] Action: Back to previous root (v0.1)
- [x] Action: Scale up/down tree widget (v0.1)
- [x] Action: Mouse click actions
- [x] Action: Mouse scroll actions
- [ ] Action: Mouse select actions
- [x] Action: Open current selected item in editor **ReadOnly** (v0.2)
- [x] Action: Switch between tree overview and data block (v0.1)
- [x] Action: Jump to parent item (v0.1)
- [x] Action: Jump to parent item and close (v0.1)
- [ ] Action: Expand selected item's children
- [ ] Action: Expand all items
- [x] Action: Close all opened items (v0.1)
- [ ] Action: Filter items, highlight searching keywords
- [ ] Action: Popup to show help messages
- [x] Action: Clipboard support, copy current item's content (might need to call external program like `wl-copy`, `pbcopy`) (v0.2)
- [x] Syntax highlighting in data block (v0.2)
- [x] Allow user to customize TUI colors and key bindings (and other things you can imagine) (v0.1)
- [ ] **Filter items! (Like [jnv](https://github.com/ynqa/jnv))**
- [x] With `--debug` flag, write some debug logs to a file (v0.4)

If you have any great ideas, please create an issue, thanks!

## Thanks

I created this tool to better view those super deep YAML files of Kubernetes while [jnv](https://github.com/ynqa/jnv) only supports JSON.

This is based on the amazing TUI framework [ratatui](https://github.com/ratatui-org/ratatui) and its tree widget [tui-tree-widget](https://github.com/EdJoPaTo/tui-rs-tree-widget.git).
