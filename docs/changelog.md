## v0.6.0

### Features

- **Highlighting for filter keywords** (#94)
  - The default filter behavior has changed: instead of hiding unmatched items, all items are now displayed, with matches highlighted. This makes it easier to distinguish filtered results without losing context.
  - If you prefer the old behavior (hiding unmatched items), you can enable it by setting `filter.exclude_mode = true`.

- **Filter navigation actions** (#95). In filter mode, you can now quickly jump between matches using the new actions:
  - `filter_next_match` (default key: `n`)
  - `filter_prev_match` (default key: `N`)

- 🎉 All items in the [roadmap](https://github.com/fioncat/otree?tab=readme-ov-file#roadmap) have been completed!

## v0.5.2

### Features

Add new XML parser!

XML will be parsed in the same way as [yq](https://github.com/mikefarah/yq).

In short:

- If an XML element contains attributes, they will be represented in the tree as String items, with their keys prefixed by `"@"` for distinction.
- If an XML element has attributes, then even if its value is a string, it will be expanded into an object. The element's text value will be stored in a special field called `"#text"`.

For example:

```xml
<outer attr="value">
  <inner>1</inner>
  <inner attr="value">2</inner>
  <inner>3</inner>
</outer>
```

Will be parsed as:

```json
{
  "outer": {
    "inner": [
      "1",
      {
        "@property": "value",
        "#text": "2"
      },
      "3"
    ]
  },
  "@property": "value"
}
```

## v0.5.1

### Fixes

- Fix panic when using action `expand_children` for empty object. (#87)

## v0.5.0

### Features

- Add `filter` (filter by keys and values, default binding to `/`), `filter_key` (filter by key, default binding to `?`) and `filter_value` (filter by values, default binding to `*`) actions to filter items. (#82)
- Add `show_help` (default binding to `H`) action to show help message (all actions and their bindings) in popup widget. (#83)
- Add `expand_children` (default binding to `x`) and `expand_all` (default binding to `X`) actions. (#84)

## v0.4.1

### Features

- when copying values, use pure text. (#80)

## v0.4.0

### Features

- Command: Add new flag `--debug`, to write some debug logs to a file.
- UI: Add `--live-reload` option, to watch file changes and update tui (#63).
- Release: Add Windows prebuilt binary (**unstable**, require more testing).

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
