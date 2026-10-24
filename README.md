# RustSpace

An [AeroSpace](https://github.com/nikitabobko/AeroSpace)-inspired tiling window manager daemon written in Rust.

> **Status:** Phase 1 — config loading, workspace model, and command dispatch.
> Window management via the macOS Accessibility API is coming in Phase 2.

---

## Requirements

- macOS 13 or later
- [Rust toolchain](https://rustup.rs/) (stable, 1.75+)

---

## Build

```sh
# Clone and enter the project
git clone <repo-url> RustSpace
cd RustSpace

# Debug build
cargo build

# Release build (recommended for daily use once Phase 2 lands)
cargo build --release
```

The compiled binary is `rustspaced`:

```
target/debug/rustspaced
target/release/rustspaced
```

---

## Test

```sh
cargo test
```

All tests should pass. Tests validate the four core workspace use cases
defined in [USE-CASES.md](USE-CASES.md):

| Test | What it checks |
|------|---------------|
| `default_config_has_workspace_switch_bindings` | `alt-1` / `alt-2` are bound in the default config |
| `default_config_has_move_node_bindings` | `alt-shift-1` / `alt-shift-2` are bound in the default config |
| `uc1_dispatch_workspace_1` | Dispatching `workspace 1` updates the active workspace |
| `uc2_dispatch_workspace_2` | Dispatching `workspace 2` updates the active workspace |
| `uc3_dispatch_move_node_to_workspace_1` | `move-node-to-workspace 1` does not change the active workspace |
| `uc4_dispatch_move_node_to_workspace_2` | `move-node-to-workspace 2` does not change the active workspace |
| `dispatch_unknown_workspace_is_noop` | Switching to a non-existent workspace is a no-op |
| `user_config_merge_overrides_single_binding` | A partial user config overlays bindings without wiping defaults |

---

## Run

```sh
# Run directly (Ctrl-C to stop)
cargo run

# Or run the compiled binary
./target/debug/rustspaced
```

On startup the daemon logs its resolved config to stderr and enters its event
loop. In Phase 1 no windows are managed yet — the loop is a placeholder for
the hotkey and IPC channels coming in later phases.

---

## Configuration

### Where to put your config

```
~/.config/rustspace/config.toml
```

The directory is created by you; the file is optional. If it does not exist
the daemon runs with the built-in defaults (see below).

On macOS the path resolves via `$XDG_CONFIG_HOME` if that variable is set,
otherwise `$HOME/.config`.

```sh
mkdir -p ~/.config/rustspace
$EDITOR ~/.config/rustspace/config.toml
```

### How config merging works

1. The **built-in defaults** (compiled into the binary from
   `resources/default-config.toml`) are loaded first.
2. Your **user config** is merged on top, field by field.
3. Within a binding mode, your keys are overlaid onto the defaults — you only
   need to write the bindings you want to change or add. Everything else is
   inherited.

### Annotated example config

```toml
# ~/.config/rustspace/config.toml
#
# Only include what you want to change.
# Omitted keys keep their built-in defaults.

config-version = 2

# Pixel gap between adjacent tiled windows
[gaps.inner]
horizontal = 8
vertical   = 8

# Pixel gap between windows and monitor edges
[gaps.outer]
left   = 4
right  = 4
top    = 4
bottom = 4

# Move the mouse cursor to the newly focused window
[focus-follows-mouse]
enabled = false

# Keyboard layout used to resolve key chord names
[key-mapping]
preset = "qwerty"   # qwerty | dvorak | colemak

# Workspaces that stay alive even when empty
persistent-workspaces = ["1", "2", "3", "4", "5"]

# Default layout for each workspace's root container
default-root-container-layout      = "tiles"      # tiles | accordion
default-root-container-orientation = "auto"        # horizontal | vertical | auto

# Override or extend the main mode bindings.
# Keys not listed here keep their built-in defaults.
[mode.main.binding]
# Example: open a terminal with alt-enter
# alt-enter = "exec-and-forget open -a iTerm"

# Example: remap workspace 1 key
# alt-1 = "workspace 1"
```

### Full default config

The complete set of built-in defaults lives in
[`resources/default-config.toml`](resources/default-config.toml).
That file is the authoritative reference for every supported key.

### Key chord syntax

| Part | Examples |
|------|---------|
| Modifiers | `cmd`, `alt`, `ctrl`, `shift` |
| Letters | `a` – `z` |
| Numbers | `0` – `9` |
| Arrows | `left`, `down`, `up`, `right` |
| Special | `minus`, `equal`, `space`, `enter`, `esc`, `backspace`, `tab`, `semicolon`, `comma`, `period`, `slash`, `backslash`, `quote`, `backtick` |

Combine with hyphens: `alt-h`, `alt-shift-1`, `ctrl-cmd-f`.

### Default key bindings

#### Main mode

| Key | Command |
|-----|---------|
| `alt-h/j/k/l` | Focus window left/down/up/right |
| `alt-shift-h/j/k/l` | Move window left/down/up/right |
| `alt-1` … `alt-9` | Switch to workspace 1–9 |
| `alt-shift-1` … `alt-shift-9` | Move focused window to workspace 1–9 |
| `alt-minus` / `alt-equal` | Resize window −50 / +50 px |
| `alt-slash` | Toggle tiles layout (horizontal/vertical) |
| `alt-comma` | Toggle accordion layout |
| `alt-tab` | Switch to previous workspace |
| `alt-shift-tab` | Move workspace to next monitor |
| `alt-shift-semicolon` | Enter service mode |

#### Service mode (`alt-shift-;` to enter, `esc` to exit)

| Key | Command |
|-----|---------|
| `esc` | Reload config, return to main mode |
| `r` | Flatten workspace tree, return to main mode |
| `f` | Toggle floating/tiling for focused window |
| `backspace` | Close all windows except the focused one |
| `alt-shift-h/j/k/l` | Join focused window with neighbour |

---

## Project layout

```
RustSpace/
├── Cargo.toml                  — package metadata and dependencies
├── README.md                   — this file
├── USE-CASES.md                — workspace use cases driving the test suite
├── RUST.md                     — implementation plan and architecture notes
├── resources/
│   └── default-config.toml    — built-in defaults (compiled into binary)
└── src/
    ├── config.rs               — config structs, TOML parsing, merge logic
    └── main.rs                 — daemon entry point, dispatch stub, tests
```

Upcoming modules (see `RUST.md` for the full build plan):

| File | Phase | Purpose |
|------|-------|---------|
| `src/ax.rs` | 2 | macOS Accessibility API wrappers |
| `src/layout.rs` | 3 | Tiling tree data structure and algorithms |
| `src/workspace.rs` | 5 | Virtual workspace switching |
| `src/hotkeys.rs` | 4 | CGEventTap global hotkey listener |
| `src/ipc.rs` | 1b | Unix-socket CLI→daemon communication |

---

## Accessibility permission

When window management is enabled (Phase 2+) the daemon needs the macOS
Accessibility permission:

**System Settings → Privacy & Security → Accessibility → enable `rustspaced`**

During development you can grant the permission to your terminal emulator
instead, which covers any binary launched from it.
