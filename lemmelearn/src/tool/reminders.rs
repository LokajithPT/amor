use crate::tool::ToolResult;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub struct Reminders {
    path: PathBuf,
    last_check: Instant,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ReminderItem {
    id: u64,
    message: String,
    timestamp: u64,
    interval_secs: u64,
    triggered: bool,
}

impl Reminders {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Self {
            path: PathBuf::from(format!("{}/amorshi/reminders.md", home)),
            last_check: Instant::now(),
        }
    }

    pub fn execute(&self, cmd: &str) -> ToolResult {
        let cmd = cmd.trim().to_lowercase();

        // SET: reminder "message" | minutes
        if cmd.starts_with("set") || cmd.starts_with("add") || cmd.contains("remind") {
            if let Some(pos) = cmd.find(':') {
                let content = cmd[pos + 1..].trim().to_string();
                return self.add_reminder(&content);
            }
        }

        // LIST: show all reminders
        if cmd.contains("list") || cmd.contains("show") || cmd.contains("get") {
            return self.list_reminders();
        }

        // DELETE: remove reminder by id
        if cmd.starts_with("delete") || cmd.starts_with("remove") || cmd.starts_with("clear") {
            if let Some(id_str) = cmd.split_whitespace().last() {
                if let Ok(id) = id_str.parse::<u64>() {
                    return self.delete_reminder(id);
                }
            }
            return self.clear_all_reminders();
        }

        ToolResult::err("Use: set reminder \"message\" | minutes, list, or delete id")
    }

    fn add_reminder(&self, content: &str) -> ToolResult {
        let parts: Vec<&str> = content.split('|').collect();
        if parts.is_empty() {
            return ToolResult::err("Format: message | minutes");
        }

        let message = parts[0].trim().to_string();
        let minutes: u64 = parts
            .get(1)
            .and_then(|m| m.trim().parse().ok())
            .unwrap_or(60);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let reminder = ReminderItem {
            id: now,
            message: message.clone(),
            timestamp: now,
            interval_secs: minutes * 60,
            triggered: false,
        };

        let entry = format!("- [{}] {} ({} min)\n", now, message, minutes);

        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            Ok(mut f) => {
                f.write_all(entry.as_bytes()).ok();
                ToolResult::ok(format!(
                    "Reminder set: '{}' in {} minutes",
                    message, minutes
                ))
            }
            Err(e) => ToolResult::err(format!("Error: {}", e)),
        }
    }

    fn list_reminders(&self) -> ToolResult {
        match fs::read_to_string(&self.path) {
            Ok(contents) => {
                if contents.trim().is_empty() {
                    ToolResult::ok("No reminders set".to_string())
                } else {
                    ToolResult::ok(contents)
                }
            }
            Err(_) => ToolResult::ok("No reminders set".to_string()),
        }
    }

    fn delete_reminder(&self, id: u64) -> ToolResult {
        match fs::read_to_string(&self.path) {
            Ok(contents) => {
                let new_contents: String = contents
                    .lines()
                    .filter(|l| !l.contains(&format!("[{}]", id)))
                    .collect::<Vec<_>>()
                    .join("\n");

                fs::write(&self.path, new_contents).ok();
                ToolResult::ok(format!("Deleted reminder {}", id))
            }
            Err(e) => ToolResult::err(format!("Error: {}", e)),
        }
    }

    fn clear_all_reminders(&self) -> ToolResult {
        fs::write(&self.path, "").ok();
        ToolResult::ok("All reminders cleared".to_string())
    }

    pub fn check_due(&mut self) -> Vec<String> {
        if self.last_check.elapsed() < Duration::from_secs(30) {
            return Vec::new();
        }
        self.last_check = Instant::now();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut due_reminders = Vec::new();

        if let Ok(contents) = fs::read_to_string(&self.path) {
            let mut new_lines = Vec::new();

            for line in contents.lines() {
                if line.starts_with("- [") {
                    if let Some(id_start) = line.find("- [") {
                        if let Some(id_end) = line[id_start..].find("]") {
                            let id_str = &line[id_start + 3..id_start + id_end];
                            if let Ok(id) = id_str.parse::<u64>() {
                                let line_interval = if line.contains("min)") {
                                    line.split('(')
                                        .nth(1)
                                        .and_then(|s| s.split(' ').next())
                                        .and_then(|s| s.parse::<u64>().ok())
                                        .unwrap_or(60)
                                } else {
                                    60
                                };

                                if now >= id + line_interval {
                                    let msg = line
                                        .split(']')
                                        .nth(1)
                                        .map(|s| s.trim().split('(').next().unwrap_or(s).trim())
                                        .unwrap_or("")
                                        .to_string();

                                    if !msg.is_empty() {
                                        due_reminders.push(msg.clone());
                                    }

                                    continue;
                                }
                            }
                        }
                    }
                    new_lines.push(line.to_string());
                }
            }

            fs::write(&self.path, new_lines.join("\n")).ok();
        }

        due_reminders
    }
}

impl Default for Reminders {
    fn default() -> Self {
        Self::new()
    }
}
