use crate::tool::ToolResult;
use std::process::Command;

pub struct Instagram;

impl Instagram {
    pub fn new() -> Self {
        Self
    }

    pub fn description(&self) -> &str {
        "# Tool: instagram\nUsage: instagram:send userid message\n       instagram:check"
    }

    pub fn execute(&self, cmd: &str) -> ToolResult {
        let cmd = cmd.trim();

        if cmd.starts_with("instagram:send") {
            if let Some(pos) = cmd.find("instagram:send") {
                let rest = cmd[pos + 15..].trim().to_string();
                let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                if parts.len() >= 2 {
                    let user_id = parts[0];
                    let message = parts[1];

                    let output = Command::new("python3")
                        .arg("/home/fuckall/code/amor/lemmelearn/amorshi/scripts/insta_listener.py")
                        .arg("send")
                        .arg(user_id)
                        .arg(message)
                        .output();

                    match output {
                        Ok(o) => {
                            let out = String::from_utf8_lossy(&o.stdout);
                            if out.contains("DM sent!") {
                                return ToolResult::ok("DM sent via Instagram");
                            } else {
                                return ToolResult::err(format!("DM failed: {}", out));
                            }
                        }
                        Err(e) => return ToolResult::err(format!("Error: {}", e)),
                    }
                }
            }
            return ToolResult::err("Format: instagram:send userid message");
        }

        if cmd.contains("instagram:check") {
            let output = Command::new("python3")
                .arg("/home/fuckall/code/amor/lemmelearn/amorshi/scripts/insta_listener.py")
                .arg("check")
                .output();

            match output {
                Ok(o) => {
                    let out = String::from_utf8_lossy(&o.stdout);
                    if out.starts_with('[') && out.len() > 2 {
                        return ToolResult::ok(out.to_string());
                    }
                    return ToolResult::ok("No new messages");
                }
                Err(e) => return ToolResult::err(format!("Error: {}", e)),
            }
        }

        ToolResult::err("Format: instagram:send userid message OR instagram:check")
    }
}

impl Default for Instagram {
    fn default() -> Self {
        Self::new()
    }
}
