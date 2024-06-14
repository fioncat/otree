# All Available Actions

| Action          | Default Keys              | Description                                                  |
| --------------- | ------------------------- | ------------------------------------------------------------ |
| move_up         | `k`, `<up>`               | Move cursor up                                               |
| move_down       | `j`, `<down>`             | Move cursor down                                             |
| move_left       | `h`, `<left>`             | Move cursor left                                             |
| move_right      | `l`, `<right>`            | Move cursor right                                            |
| select_focus    | `<enter>`                 | Toggle select current item                                   |
| select_parent   | `p`                       | Move cursor to the parent item                               |
| select_first    | `g`                       | Move cursor to the top                                       |
| select_last     | `G`                       | Move cursor to the bottom                                    |
| close_parent    | `<backspace>`             | Move cursor to the parent and close                          |
| change_root     | `r`                       | Change current item as root<br/>Use `reset` action to recover |
| reset           | `<esc>`                   | Reset cursor and items                                       |
| page_up         | `<page-up>`, `<ctrl-y>`   | Scroll up                                                    |
| page_down       | `<page-down>`, `<ctrl-e>` | Scroll down                                                  |
| change_layout   | `v`                       | Change current layout                                        |
| tree_scale_up   | `[`                       | Scale up tree widget                                         |
| tree_scale_down | `]`                       | Scale down tree widget                                       |
| switch          | `<tab>`                   | Switch focus widget                                          |
| edit            | `e`                       | Open current item in editor<br />**Notice that edited changes won't be saved** |
| quit            | `<ctrl-c>`, `q`           | Quit program                                                 |

All available keys:

- `x`：Single key. Use a single character to map keyboard keys.
- Special keys in keyboard: `<up>`, `<down>`, `<left>`, `<right>`, `<backspace>`, `<enter>`, `<page-up>`, `<page-down>`, `<tab>`, `<esc>`.
- `<fn>`: Function keys, like `<f1>`, `<f2>`, the `n` should be less or equal than `12`.
- `<ctrl-x>`: Press control and another single key.
- `<alt-x>`: Press alt and another single key.

You can change the key bindings in config file, like:

```toml
[keys]
select_focus = [" ", "<enter>"]
```

This changes `select_focus` action's key binding to `space` and `<enter>` keys.
