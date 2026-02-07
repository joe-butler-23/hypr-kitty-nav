use std::env;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: hypr-tmux-nav <h|j|k|l|left|right|up|down>");
        std::process::exit(1);
    }

    let (move_dir, tmux_dir) = match args[1].as_str() {
        "h" | "left" => ("l", "L"),
        "l" | "right" | "r" => ("r", "R"),
        "k" | "up" | "u" => ("u", "U"),
        "j" | "down" | "d" => ("d", "D"),
        _ => std::process::exit(2),
    };

    let xdg = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let sig = env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap_or_default();
    if sig.is_empty() {
        std::process::exit(1);
    }

    let hypr_socket = PathBuf::from(&xdg)
        .join("hypr")
        .join(&sig)
        .join(".socket.sock");

    // Check if active window is running tmux
    if let Some(pid) = get_active_window_pid(&hypr_socket) {
        if is_running_tmux(pid) {
            // Try to find the tmux session for this terminal
            if let Some(session) = find_tmux_session_for_pid(pid) {
                if try_tmux_navigate_session(&session, tmux_dir) {
                    return;
                }
            }
        }
    }

    // Fall back to Hyprland window navigation
    hypr_dispatch(&hypr_socket, move_dir);
}

fn get_active_window_pid(socket_path: &PathBuf) -> Option<u32> {
    let mut stream = UnixStream::connect(socket_path).ok()?;
    stream.write_all(b"activewindow").ok()?;
    stream.shutdown(std::net::Shutdown::Write).ok()?;

    let mut response = String::new();
    stream.read_to_string(&mut response).ok()?;

    response
        .lines()
        .find(|l| l.trim().starts_with("pid: "))
        .and_then(|l| l.trim().strip_prefix("pid: "))
        .and_then(|s| s.trim().parse::<u32>().ok())
}

fn is_running_tmux(pid: u32) -> bool {
    let mut pids_to_check = vec![pid];
    let mut checked = std::collections::HashSet::new();

    while let Some(current_pid) = pids_to_check.pop() {
        if checked.contains(&current_pid) {
            continue;
        }
        checked.insert(current_pid);

        if let Ok(cmdline) = fs::read_to_string(format!("/proc/{}/cmdline", current_pid)) {
            let cmd = cmdline.replace('\0', " ");
            if cmd.contains("tmux") && !cmd.contains("hypr-tmux-nav") {
                return true;
            }
        }

        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if !metadata.is_dir() {
                        continue;
                    }
                }

                if let Some(name) = entry.file_name().to_str() {
                    if let Ok(child_pid) = name.parse::<u32>() {
                        if let Ok(status) =
                            fs::read_to_string(format!("/proc/{}/status", child_pid))
                        {
                            for line in status.lines() {
                                if line.starts_with("PPid:") {
                                    if let Some(ppid_str) = line.strip_prefix("PPid:") {
                                        if let Ok(ppid) = ppid_str.trim().parse::<u32>() {
                                            if ppid == current_pid {
                                                pids_to_check.push(child_pid);
                                            }
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

fn find_tmux_session_for_pid(pid: u32) -> Option<String> {
    // Get the TTY of the active window's process
    let tty = fs::read_link(format!("/proc/{}/fd/0", pid)).ok()?;
    let tty_str = tty.to_str()?;

    // Get all tmux clients and find one using this TTY
    let output = Command::new("tmux")
        .args(&["list-clients", "-F", "#{client_session} #{client_tty}"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let clients = String::from_utf8_lossy(&output.stdout);
    for line in clients.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let session = parts[0];
            let client_tty = parts[1];
            if tty_str.ends_with(client_tty) || client_tty.ends_with(tty_str) {
                return Some(session.to_string());
            }
        }
    }

    // Fallback: return most recently attached session
    let output = Command::new("tmux")
        .args(&[
            "list-sessions",
            "-F",
            "#{session_name} #{session_last_attached}",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if output.status.success() {
        let mut sessions: Vec<(String, u64)> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 1 {
                    let name = parts[0].to_string();
                    let time = parts
                        .get(1)
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0);
                    Some((name, time))
                } else {
                    None
                }
            })
            .collect();

        sessions.sort_by(|a, b| b.1.cmp(&a.1));
        return sessions.first().map(|(name, _)| name.clone());
    }

    None
}

fn try_tmux_navigate_session(session: &str, direction: &str) -> bool {
    let before = get_tmux_pane(session);

    let status = Command::new("tmux")
        .args(&["select-pane", "-t", session, &format!("-{}", direction)])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if let Ok(s) = status {
        if s.success() {
            let after = get_tmux_pane(session);
            if before != after {
                return true;
            }
        }
    }

    false
}

fn get_tmux_pane(session: &str) -> Option<String> {
    let output = Command::new("tmux")
        .args(&["display-message", "-t", session, "-p", "#{pane_id}"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn hypr_dispatch(socket_path: &PathBuf, dir: &str) {
    if let Ok(mut stream) = UnixStream::connect(socket_path) {
        let cmd = format!("dispatch movefocus {}", dir);
        let _ = stream.write_all(cmd.as_bytes());
    }
}
