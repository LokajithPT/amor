use crate::tool::ToolResult;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub struct Memory {
    path: PathBuf,
}

impl Memory {
    pub fn new() -> Self {
        // Use /home/fuckall/amorshi/memory.md directly on RasPi, or fall back to HOME/amorshi
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let path = if home == "/home/fuckall" {
            PathBuf::from("/home/fuckall/amorshi/memory.md")
        } else {
            PathBuf::from(format!("{}/amorshi/memory.md", home))
        };

        Self { path }
    }

    pub fn execute(&self, cmd: &str) -> ToolResult {
        let cmd = cmd.trim().to_lowercase();
        eprintln!("MEMORY TOOL received: {}", cmd);

        // Save: memsave:content or save:content
        if cmd.starts_with("memsave") || cmd.starts_with("save") {
            let content = if let Some(pos) = cmd.find(':') {
                cmd[pos + 1..].trim().to_string()
            } else {
                cmd.to_string()
            };
            let content = content.replace('"', "");
            if content.is_empty() {
                return ToolResult::err("Empty");
            }

            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let entry = format!("\n[{}] {}\n", ts, content);

            return match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)
            {
                Ok(mut f) => match f.write_all(entry.as_bytes()) {
                    Ok(_) => ToolResult::ok(format!("Saved: {}", content)),
                    Err(e) => ToolResult::err(format!("Error: {}", e)),
                },
                Err(e) => ToolResult::err(format!("Error: {}", e)),
            };
        }

        // Recall: memrecall:query or recall:query
        if cmd.starts_with("memrecall") || cmd.starts_with("recall") || cmd.contains("remember") {
            let query = if let Some(pos) = cmd.find(':') {
                cmd[pos + 1..].trim().to_string()
            } else {
                cmd.replace("remember", "")
            };
            let query = query.replace('"', "");

            return match fs::read_to_string(&self.path) {
                Ok(contents) => {
                    let q = query.to_lowercase();
                    // Split query into words and match any of them
                    let words: Vec<&str> = q.split_whitespace().collect();
                    let matches: Vec<_> = contents
                        .lines()
                        .filter(|l| {
                            let line = l.to_lowercase();
                            words.iter().any(|w| line.contains(w))
                        })
                        .collect();
                    if matches.is_empty() {
                        ToolResult::ok(format!("No: {}", query))
                    } else {
                        ToolResult::ok(matches.join("\n"))
                    }
                }
                Err(_) => ToolResult::ok("No memory"),
            };
        }

        ToolResult::err("Use: memsave: or memrecall:")
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}
