## v0.4.0

### Features

- Command: Add new flag `--debug`, to write some debug logs to a file.

## v0.3.0

### Features

- Parser: Support [JSONL](https://jsonlines.org/)
- README: Add Homebrew instructions.

### Fixes

- Allow reading from stdin on macOS.

## v0.2.0

### Features

- UI: Add footer to show current root and identify.
- UI: Support popup widget to show error messages.
- UI: Syntax highlighting in data block widget (allow customization).
- New Action: Open current selected item's content in default editor (readonly).
- New Action: Copy current selected item's name or content to system clipboard.

### Fixes

- Fixed some typos.
- Allow building without git repository, gather and integrate git build information only if the `BUILD_OTREE_WITH_GIT_INFO` is set.

## v0.1.0

### Features

- Basic tree and data UI.
- Basic actions, see: [actions](./actions.md).
