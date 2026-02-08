# hypr-nav-suite

A hackable suite of Rust tools for smart, context-aware interaction between Hyprland, Kitty, and Tmux.

This project enables **Seamless Context Switching**: use the same keybinds (like `Super+H/J/K/L` or `Super+C`) to interact with the active application's internal state (Tmux panes, Kitty windows) or fall back to the window manager (Hyprland) when appropriate.

It is designed to be **extensible**. The core logic is extracted into a shared library, making it easy to build your own "Smart Binds" that inspect the current window/process state before deciding on an action.

## Tools

### 1. hypr-tmux-nav
Seamless navigation between Tmux panes and Hyprland windows.

- **Smart Directional Awareness**: Checks if you are at the edge of a Tmux window.
- **Auto-Fallback**: If you try to move `UP` from the top Tmux pane, it automatically focuses the Hyprland window above your terminal.
- **Process Tree Detection**: Automatically detects if the active terminal is running Tmux (even via SSH or nested shells).

### 2. hypr-smart-close
Context-aware closing logic for `Super+C`.

- **Named Sessions**: If inside a named Tmux session, `Super+C` **detaches** the client instead of killing it.
- **Panes**: If inside a generic session with multiple panes, `Super+C` closes the active **pane**.
- **Windows**: If it's the last pane of the last window, or not in Tmux, `Super+C` closes the **Hyprland window**.

### 3. hypr-kitty-nav
Original tool for navigating between native Kitty windows (splits) and Hyprland.
*Note: Requires Kitty remote control enabled.*

## Architecture & Hacking

The core logic is modularized in `src/lib.rs`. You can easily create your own binary to handle other keys (e.g., `Super+Enter` to spawn a pane vs window).

**General Pattern:**
1. **Identify Context**: Get active Hyprland window info (class, PID).
2. **Deep Inspection**: Traverse process tree to find target apps (Tmux, Neovim, etc.).
3. **Query State**: Use app-specific IPC (Tmux CLI, Kitty socket) to check state (e.g., `is_pane_at_edge`).
4. **Action or Fallback**: Perform internal app action or dispatch Hyprland dispatcher.

### Example: Custom Action
Want a key that maximizes a Tmux pane if active, or toggles fullscreen in Hyprland otherwise?

```rust
// src/my-custom-bind.rs
use hypr_nav_lib::*;

fn main() {
    let socket = find_hyprland_socket().unwrap();
    if let Some((_, pid)) = get_active_window_info(&socket) {
        if let Some((tty, true)) = detect_tmux_and_tty(pid) {
             // Run tmux zoom
             return;
        }
    }
    // Fallback
    hypr_dispatch(&socket, "fullscreen 0");
}
```

## Installation

### From Source
```bash
git clone https://github.com/joe-butler-23/hypr-kitty-nav
cd hypr-kitty-nav
cargo build --release
cp target/release/hypr-tmux-nav ~/.local/bin/
cp target/release/hypr-smart-close ~/.local/bin/
```

### Configuration (Hyprland)

```ini
# Navigation
bind = SUPER, h, exec, hypr-tmux-nav left
bind = SUPER, j, exec, hypr-tmux-nav down
bind = SUPER, k, exec, hypr-tmux-nav up
bind = SUPER, l, exec, hypr-tmux-nav right

# Smart Close
bind = SUPER, c, exec, hypr-smart-close
```

## Requirements
- Rust (cargo)
- Hyprland
- Tmux (for tmux tools)
- Kitty (for kitty tools)
