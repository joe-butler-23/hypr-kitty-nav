use std::process::{Command, Stdio};
use hypr_nav_lib::*;

fn main() {
    let hypr_socket = match find_hyprland_socket() {
        Some(path) => path,
        None => std::process::exit(1),
    };

    // Get active window info (class + PID) in single query
    if let Some((class, pid)) = get_active_window_info(&hypr_socket) {
        // Fast path: skip non-terminal windows entirely
        if is_terminal_class(&class) {
            // Combined detection: find TTY and check for tmux in one tree walk
            if let Some((tty, has_tmux)) = detect_tmux_and_tty(pid) {
                if has_tmux {
                    // Try to find session and close pane
                    if let Some(session) = find_tmux_session(&tty) {
                        if handle_tmux_close(&session, &hypr_socket) {
                            return;
                        }
                    }
                }
            }
        }
    }

    // Fall back to Hyprland close
    hypr_dispatch(&hypr_socket, "killactive");
}

fn handle_tmux_close(session: &str, _hypr_socket: &std::path::PathBuf) -> bool {
    if let Some(info) = get_tmux_session_info(session) {
        // 1. If named session -> Detach
        if info.is_named {
             let _ = Command::new("tmux")
                .args(&["detach-client", "-t", session])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .output();
             return true;
        }

        // 2. If last window AND last pane -> Detach & Kill Window (let Hyprland handle it or exit cleanly)
        // Actually, user said: "if its the last tmux window than super c detaches and kills the whole window rather than exiting tmux"
        // This likely means closing the terminal emulator window itself.
        // If we just close the pane, tmux exits, shell exits, kitty closes.
        // But maybe they want to FORCE close the window to be sure.
        // If it's the last pane of the last window, closing it typically closes tmux.
        
        if info.window_count == 1 && info.pane_count == 1 {
            // Fallthrough to Hyprland killactive to close the kitty window directly
            // effectively killing tmux (since it's attached).
            // Return false to trigger fallback.
            return false;
        }

        // 3. Otherwise -> Kill Pane
        return try_tmux_close_pane(session);
    }
    false
}

fn try_tmux_close_pane(session: &str) -> bool {
    // We target the session. tmux implicitly targets the active window/pane for that session.
    // If it succeeds, we return true (meaning we handled the close).
    // If it fails (e.g. no session?), we return false (fallback to Hyprland).
    let output = Command::new("tmux")
        .args(&[
            "kill-pane",
            "-t",
            session,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output();
        
    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}
