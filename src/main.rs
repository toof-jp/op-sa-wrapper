use std::process::Command;

fn load_op_service_account_token() -> String {
    let home = std::env::var("HOME").expect("HOME no set");
    let token_path = format!("{}/.secret/op_service_account_token.gpg", home);
    let output = Command::new("gpg")
        .args(["--decrypt", &token_path])
        .output()
        .expect("failed to decrypt token");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn main() {
    let op_service_account_token = load_op_service_account_token();

    let mut args = std::env::args().skip(1);

    let op_path = "/usr/bin/op";
    let mut cmd = Command::new(op_path);

    if let Some(command) = args.next() {
        let args: Vec<String> = args.collect();

        if command == "run" {
            if let Some(pos) = args.iter().position(|s| s == "--") {
                let flags = args[..pos].to_vec();
                let commands = args[pos + 1..].to_vec();

                fn shell_escape(arg: &str) -> String {
                    if arg.is_empty() {
                        return "''".to_string();
                    }
                    let needs_quotes = arg.bytes().any(|b| matches!(b,
                        b' ' | b'\t' | b'\n' | b'\r' | b'"' | b'\'' | b'`' | b'\\' | b'$' | b'&' |
                        b'|' | b'>' | b'<' | b';' | b'(' | b')' | b'[' | b']' | b'{' | b'}' |
                        b'*' | b'?' | b'!' | b'~'));

                    if !needs_quotes {
                        return arg.to_string();
                    }

                    let mut out = String::with_capacity(arg.len() + 2);
                    out.push('\'');
                    for ch in arg.chars() {
                        if ch == '\'' {
                            out.push_str("'\\''");
                        } else {
                            out.push(ch);
                        }
                    }
                    out.push('\'');
                    out
                }

                let escaped = commands
                    .iter()
                    .map(|s| shell_escape(s))
                    .collect::<Vec<_>>()
                    .join(" ");

                cmd.arg("run");
                cmd.args(flags);
                cmd.args([
                    "--",
                    "sh",
                    "-c",
                    &format!("unset OP_SERVICE_ACCOUNT_TOKEN && {}", escaped),
                ]);
            }
        } else {
            cmd.args(args);
        }
    }

    let status = cmd
        .env("OP_SERVICE_ACCOUNT_TOKEN", op_service_account_token)
        .status()
        .expect("op failed");

    if let Some(code) = status.code() {
        std::process::exit(code);
    }
}
