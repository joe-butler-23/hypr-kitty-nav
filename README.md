# hypr-kitty-nav

Smart directional navigation between Kitty panes and Hyprland windows.

## How it works

- Reads the active window from the Hyprland socket
- If the active window is Kitty, focuses the neighboring Kitty pane
- Otherwise, moves Hyprland focus in the same direction

## Requirements

- Hyprland (sets `HYPRLAND_INSTANCE_SIGNATURE`)
- Kitty with its IPC enabled (default)

## Build and install

```bash
cargo build --release
install -Dm755 target/release/hypr-kitty-nav ~/.local/bin/hypr-kitty-nav
```

## Usage

```bash
hypr-kitty-nav left
hypr-kitty-nav h
```

## Hyprland example

```ini
bind = SUPER, H, exec, hypr-kitty-nav left
bind = SUPER, J, exec, hypr-kitty-nav down
bind = SUPER, K, exec, hypr-kitty-nav up
bind = SUPER, L, exec, hypr-kitty-nav right
```
