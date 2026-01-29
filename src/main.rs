use std::env;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: hypr-kitty-nav <h|j|k|l|left|right|up|down>");
        std::process::exit(1);
    }

    let (move_dir, kitty_dir) = match args[1].as_str() {
        "h" | "left" => ("l", "left"),
        "l" | "right" | "r" => ("r", "right"),
        "k" | "up" | "u" => ("u", "top"),
        "j" | "down" | "d" => ("d", "bottom"),
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

    if is_kitty_active(&hypr_socket) {
        let kitty_sock = PathBuf::from(&xdg).join("kitty");
        if kitty_sock.exists() {
            let status = Command::new("kitty")
                .args(&[
                    "@",
                    "--to",
                    &format!("unix:{}", kitty_sock.display()),
                    "focus-window",
                    "--match",
                    &format!("neighbor:{}", kitty_dir),
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();

            if let Ok(s) = status {
                if s.success() {
                    return;
                }
            }
        }
    }

    hypr_dispatch(&hypr_socket, move_dir);
}

fn is_kitty_active(socket_path: &PathBuf) -> bool {
    if let Ok(mut stream) = UnixStream::connect(socket_path) {
        if stream.write_all(b"activewindow").is_ok() {
            let mut response = String::new();
            if stream.read_to_string(&mut response).is_ok() {
                return response.contains("class: kitty");
            }
        }
    }

    false
}

fn hypr_dispatch(socket_path: &PathBuf, dir: &str) {
    if let Ok(mut stream) = UnixStream::connect(socket_path) {
        let cmd = format!("dispatch movefocus {}", dir);
        let _ = stream.write_all(cmd.as_bytes());
    }
}
