# All Available Actions

| Action                    | Default Keys              | Description                                                  |
| ------------------------- | ------------------------- | ------------------------------------------------------------ |
| move_up                   | `k`, `<up>`               | Move cursor up                                               |
| move_down                 | `j`, `<down>`             | Move cursor down                                             |
| move_left                 | `h`, `<left>`             | Move cursor left<br />*In filter mode: move cursor back*     |
| move_right                | `l`, `<right>`            | Move cursor right<br />*In filter mode: move cursor forward* |
| select_focus              | `<enter>`                 | Toggle select current item<br />*In filter mode: confirm filtering* |
| select_parent             | `p`                       | Move cursor to the parent item                               |
| select_first              | `g`                       | Move cursor to the top<br />*In filter mode: move cursor to the head* |
| select_last               | `G`                       | Move cursor to the bottom<br />*In filter mode: move cursor to the end* |
| close_parent              | `<backspace>`             | Move cursor to the parent and close<br />*In filter mode: delete a character* |
| change_root               | `r`                       | Change current item as root<br/>Use `reset` action to recover |
| reset                     | `<esc>`                   | Reset cursor and items<br />*In filter mode: cancel filtering* |
| page_up                   | `<page-up>`, `<ctrl-y>`   | Scroll up                                                    |
| page_down                 | `<page-down>`, `<ctrl-e>` | Scroll down                                                  |
| change_layout             | `v`                       | Change current layout                                        |
| tree_scale_up             | `[`                       | Scale up tree widget                                         |
| tree_scale_down           | `]`                       | Scale down tree widget                                       |
| switch                    | `<tab>`                   | Switch focus widget                                          |
| edit                      | `e`                       | Open current item in editor<br />**(ReadOnly)**              |
| copy_name                 | `y`                       | Copy current selected item's name                            |
| copy_value                | `Y`                       | Copy current selected item's value                           |
| filter                    | `/`                       | Enter the filter mode (key and value)                        |
| filter_key                | `?`                       | Enter the filter mode (key)                                  |
| filter_value              | `*`                       | Enter the filter mode (value)                                |
| filter_switch_ignore_case | `I`                       | Change the filter's ignore case mode                         |
| quit                      | `<ctrl-c>`, `q`           | Quit program                                                 |

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
