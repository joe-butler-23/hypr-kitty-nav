# hypr-kitty-nav

Smart directional navigation between Kitty and Hyprland. Lets you use the same keybinds to navigate between windows inside a single instance of Kitty as you use to navigate between windows in Hyprland. 

- Reads the active window from the Hyprland socket
- If the active window is Kitty, focuses the neighboring Kitty window (sub-window? pane?)
- Otherwise, moves Hyprland focus in the same direction

This started life as a very clunky and laggy script that used different jq calls, which turned into a slightly better bash version. And then Codex (yes, the code is AI-generated) came up with this version in rust using IPC sockets which has basically zero latency.

## Binaries

This package provides two binaries:

- **hypr-kitty-nav** - Navigation between Kitty panes and Hyprland windows
- **hypr-tmux-nav** - Navigation between tmux panes and Hyprland windows

## Requirements

Obviously Hyprland and cargo to install if you want to build from source.

### For hypr-kitty-nav (Kitty)

Because this uses Kitty's remote control functionality, you need the following in your Kitty config:

```
allow_remote_control socket-only
listen_on unix:/run/user/1000/kitty
```

And then you set your Hyprland binds like this:

```ini
bind = SUPER, H, exec, hypr-kitty-nav left
bind = SUPER, J, exec, hypr-kitty-nav down
bind = SUPER, K, exec, hypr-kitty-nav up
bind = SUPER, L, exec, hypr-kitty-nav right
```

### For hypr-tmux-nav (tmux)

No special tmux configuration is required - the binary uses tmux's standard command interface.

Set your Hyprland binds like this:

```ini
bind = SUPER, H, exec, hypr-tmux-nav left
bind = SUPER, J, exec, hypr-tmux-nav down
bind = SUPER, K, exec, hypr-tmux-nav up
bind = SUPER, L, exec, hypr-tmux-nav right
```

**How it works:**
1. Checks if the active Hyprland window is running tmux
2. If so, attempts to navigate to the neighboring tmux pane
3. If navigation within tmux doesn't change panes (i.e., at the edge), falls back to Hyprland window navigation

This gives you seamless vim-style navigation that automatically "escapes" from tmux when you're at the edge of your pane layout.

**Terminal detection:** The tool uses the `$TERMINAL` environment variable to identify terminal windows. If not set, it falls back to a hardcoded list (currently just `kitty`). Non-terminal windows skip tmux detection entirely for faster response.

## Build and install

If you don't want to install anything, it would be pretty trivial to rewrite `src/main.rs` as a script that you could then call directly from your Hyprland.conf. Otherwise:

```bash
git clone https://github.com/joe-butler-23/hypr-kitty-nav
cd hypr-kitty-nav
cargo build --release
install -Dm755 target/release/hypr-kitty-nav ~/.local/bin/hypr-kitty-nav
install -Dm755 target/release/hypr-tmux-nav ~/.local/bin/hypr-tmux-nav
```

Or:

```bash
cargo install --git https://github.com/joe-butler-23/hypr-kitty-nav
```
