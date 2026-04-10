use crate::tool::ToolResult;
use std::process::Command;

const TOOL_DESC: &str = r#"# Tool: bash
Usage:
- bash:"command" - run shell command
"#;

pub struct Bash;

impl Bash {
    pub fn new() -> Self {
        Self
    }

    pub fn description(&self) -> &str {
        TOOL_DESC
    }

    pub fn execute(&self, cmd: &str) -> ToolResult {
        let cmd = cmd.trim();

        if let Some(after) = cmd.strip_prefix("bash:\"") {
            if let Some(end) = after.rfind('"') {
                let cmd = &after[..end];
                return match Command::new("sh").arg("-c").arg(cmd).output() {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if stdout.is_empty() && !stderr.is_empty() {
                            ToolResult::ok(stderr.to_string())
                        } else {
                            ToolResult::ok(stdout.to_string())
                        }
                    }
                    Err(e) => ToolResult::err(format!("Error: {}", e)),
                };
            }
        }

        ToolResult::err("Format: bash:\"command\"".to_string())
    }
}

impl Default for Bash {
    fn default() -> Self {
        Self::new()
    }
}
