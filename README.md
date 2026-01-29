# hypr-kitty-nav

Smart directional navigation between Kitty and Hyprland. Lets you use the same keybinds to navigate between windows inside a single instance of Kitty as you use to navigate between windows in Hyprland. 

- Reads the active window from the Hyprland socket
- If the active window is Kitty, focuses the neighboring Kitty window (sub-window? pane?)
- Otherwise, moves Hyprland focus in the same direction

This started life as a very clunky and laggy script that used different jq calls, which turned into a slightly better bash version. And then Codex (yes, the code is AI-generated) came up with this version in rust using IPC sockets which has basically zero latency.

## Requirements

Obviously Hyprland and Kitty, and cargo to install if you want to build from source. Because this uses Kitty's remote control functionality, you need the following in your Kitty config:

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

## Build and install

If you don't want to install anything, it would be pretty trivial to rewrite `src/main.rs` as a script that you could then call directly from your Hyprland.conf. Otherwise:

```bash
git clone https://github.com/joe-butler-23/hypr-kitty-nav
cd hypr-kitty-nav
cargo build --release
install -Dm755 target/release/hypr-kitty-nav ~/.local/bin/hypr-kitty-nav
```

Or:

```bash
cargo install --git https://github.com/joe-butler-23/hypr-kitty-nav
```
