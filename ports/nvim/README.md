# Acid for Neovim

Two flavours: **Acetic** (`#000000`), vibrant, and **Citric** (`#1c1b19`), muted.

Part of [Acid](https://github.com/acid-theme/acid), a very dark colourscheme in two
flavours. The main README lists the other ports.

## Install

With Neovim's built-in plugin manager:

```lua
vim.pack.add({ { src = "https://github.com/acid-theme/nvim" } })
vim.cmd.colorscheme("acid-acetic")
```

With lazy.nvim:

```lua
{ "acid-theme/nvim", name = "acid", lazy = false, priority = 1000 }
```

Or without a plugin manager, by dropping `colors/acid-acetic.lua` into
`~/.config/nvim/colors/`.

Covers the editor and chrome groups, legacy syntax, treesitter, markup,
diagnostics, LSP semantic tokens, diffs, spelling, `:terminal` and git signs.

## Files

- `colors/acid-acetic.lua`
- `colors/acid-citric.lua`

## Generated

Rendered by acidify from [`ports/nvim/acid.lua.tera`](https://github.com/acid-theme/acid/blob/main/ports/nvim/acid.lua.tera).
Edits to these files are overwritten on the next release. Report issues on
[acid-theme/acid](https://github.com/acid-theme/acid/issues).

## Licence

MIT.
