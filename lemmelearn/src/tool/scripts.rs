use crate::tool::ToolResult;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const TOOL_DESC: &str = r#"# Tool: scripts
Usage:
- script_create:"name|content" - create script
- script_run:"name" - run a script
- script_delete:"name" - delete script
- ls_scripts - list scripts
"#;

pub struct Scripts {
    scripts_dir: PathBuf,
}

impl Scripts {
    pub fn new() -> Self {
        Self {
            scripts_dir: PathBuf::from("amorshi/scripts"),
        }
    }

    pub fn description(&self) -> &str {
        TOOL_DESC
    }

    pub fn execute(&self, cmd: &str) -> ToolResult {
        let cmd = cmd.trim();

        if cmd.starts_with("script_create:\"") {
            let after = &cmd[16..];
            if let Some(pipe) = after.find('|') {
                let name = after[..pipe].trim_end_matches('"');
                let content = &after[pipe + 1..];
                let path = self.scripts_dir.join(name);
                return match fs::write(&path, content) {
                    Ok(_) => ToolResult::ok(format!("Created: {}", name)),
                    Err(e) => ToolResult::err(format!("Error: {}", e)),
                };
            }
            return ToolResult::err("Format: script_create:\"name|content\"".to_string());
        }

        if cmd.starts_with("script_run:\"") {
            let after = &cmd[12..];
            if let Some(end) = after.find('"') {
                let name = &after[..end];
                let path = self.scripts_dir.join(name);
                if !path.exists() {
                    return ToolResult::err(format!("Not found: {}", name));
                }
                return match Command::new("python3").arg(&path).output() {
                    Ok(output) => {
                        let out = String::from_utf8_lossy(&output.stdout);
                        let err = String::from_utf8_lossy(&output.stderr);
                        ToolResult::ok(if out.is_empty() {
                            err.to_string()
                        } else {
                            out.to_string()
                        })
                    }
                    Err(e) => ToolResult::err(format!("Error: {}", e)),
                };
            }
        }

        if cmd.starts_with("script_delete:\"") {
            let after = &cmd[15..];
            if let Some(end) = after.find('"') {
                let name = &after[..end];
                let path = self.scripts_dir.join(name);
                return match fs::remove_file(&path) {
                    Ok(_) => ToolResult::ok(format!("Deleted: {}", name)),
                    Err(e) => ToolResult::err(format!("Error: {}", e)),
                };
            }
        }

        if cmd == "ls_scripts" {
            return match fs::read_dir(&self.scripts_dir) {
                Ok(entries) => {
                    let list: Vec<_> = entries
                        .flatten()
                        .filter_map(|e| {
                            e.path()
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                        })
                        .collect();
                    if list.is_empty() {
                        return ToolResult::ok("No scripts".to_string());
                    }
                    ToolResult::ok(list.join("\n"))
                }
                Err(e) => ToolResult::err(format!("Error: {}", e)),
            };
        }

        ToolResult::err("Usage: script_create/run/delete:\"name\" or ls_scripts".to_string())
    }
}

impl Default for Scripts {
    fn default() -> Self {
        Self::new()
    }
}
