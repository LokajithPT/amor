use crate::tool::ToolResult;
use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Worker {
    active_workers: Mutex<HashMap<String, WorkerInfo>>,
}

#[derive(Debug, Clone)]
struct WorkerInfo {
    pid: u32,
    task: String,
    started: u64,
    status: String,
}

impl Worker {
    pub fn new() -> Self {
        Self {
            active_workers: Mutex::new(HashMap::new()),
        }
    }

    pub fn execute(&self, cmd: &str) -> ToolResult {
        let parts: Vec<&str> = cmd.splitn(2, ':').collect();
        if parts.is_empty() {
            return ToolResult::err("No command specified");
        }

        let action = parts[0].to_lowercase();

        if action == "spawn" || action == "start" {
            if parts.len() < 2 {
                return ToolResult::err("Usage: worker:spawn|<task_id>|<command>");
            }
            let task_parts: Vec<&str> = parts[1].splitn(2, '|').collect();
            if task_parts.len() < 2 {
                return ToolResult::err("Usage: worker:spawn|<task_id>|<command>");
            }
            let task_id = task_parts[0];
            let command = task_parts[1];
            self.spawn_worker(task_id, command)
        } else if action == "kill" || action == "stop" {
            if parts.len() < 2 {
                return ToolResult::err("Usage: worker:kill|<task_id>");
            }
            let task_id = parts[1];
            self.kill_worker(task_id)
        } else if action == "status" || action == "check" {
            if parts.len() < 2 {
                self.list_workers()
            } else {
                let task_id = parts[1];
                self.check_worker(task_id)
            }
        } else if action == "list" {
            self.list_workers()
        } else {
            ToolResult::err(format!(
                "Unknown worker command: {}. Use spawn, kill, status, or list",
                action
            ))
        }
    }

    fn spawn_worker(&self, task_id: &str, command: &str) -> ToolResult {
        let worker_script = format!(
            r#"#!/bin/bash
# Worker daemon for task: {}
# Reports status, completes when done

TASK_ID="{}"
LOG_FILE="/tmp/worker_{}.log"
PID_FILE="/tmp/worker_{}.pid"

echo "$$" > "$PID_FILE"
echo "Worker $TASK_ID started at $(date)" > "$LOG_FILE"

# Run the command
{} >> "$LOG_FILE" 2>&1

EXIT_CODE=$?
echo "Worker $TASK_ID completed with exit code $EXIT_CODE at $(date)" >> "$LOG_FILE"
rm -f "$PID_FILE"
exit $EXIT_CODE
"#,
            task_id, task_id, task_id, task_id, command
        );

        let script_path = format!("/tmp/worker_spawn_{}.sh", task_id);
        if let Err(e) = std::fs::write(&script_path, &worker_script) {
            return ToolResult::err(format!("Failed to write worker script: {}", e));
        }

        // Make executable
        if let Err(e) = Command::new("chmod").arg("+x").arg(&script_path).output() {
            return ToolResult::err(format!("Failed to chmod: {}", e));
        }

        // Spawn the worker in background
        match Command::new("nohup")
            .arg(&script_path)
            .arg(">/dev/null")
            .arg("2>&1")
            .arg("&")
            .spawn()
        {
            Ok(child) => {
                let pid = child.id();
                let started = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                self.active_workers.lock().unwrap().insert(
                    task_id.to_string(),
                    WorkerInfo {
                        pid,
                        task: command.to_string(),
                        started,
                        status: "running".to_string(),
                    },
                );

                ToolResult::ok(format!(
                    "Worker {} spawned with PID {} for task: {}",
                    task_id, pid, command
                ))
            }
            Err(e) => ToolResult::err(format!("Failed to spawn worker: {}", e)),
        }
    }

    fn kill_worker(&self, task_id: &str) -> ToolResult {
        let mut workers = self.active_workers.lock().unwrap();

        if let Some(info) = workers.get(task_id) {
            let result = Command::new("kill")
                .arg("-9")
                .arg(info.pid.to_string())
                .output();

            workers.remove(task_id);

            match result {
                Ok(_) => ToolResult::ok(format!("Worker {} killed", task_id)),
                Err(e) => ToolResult::err(format!("Failed to kill: {}", e)),
            }
        } else {
            // Also try by PID file
            let pid_file = format!("/tmp/worker_{}.pid", task_id);
            if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                    let _ = Command::new("kill").arg("-9").arg(pid.to_string()).output();
                }
                let _ = std::fs::remove_file(&pid_file);
            }
            ToolResult::ok(format!("Worker {} stopped (or not found)", task_id))
        }
    }

    fn check_worker(&self, task_id: &str) -> ToolResult {
        let workers = self.active_workers.lock().unwrap();

        if let Some(info) = workers.get(task_id) {
            // Check if process is still running
            let running = Command::new("ps")
                .arg("-p")
                .arg(info.pid.to_string())
                .arg("-o")
                .arg("pid=")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if running {
                ToolResult::ok(format!(
                    "Worker {} running (PID {}) - started {} seconds ago",
                    task_id, info.pid, info.started
                ))
            } else {
                ToolResult::ok(format!("Worker {} appears to have finished", task_id))
            }
        } else {
            // Check PID file
            let pid_file = format!("/tmp/worker_{}.pid", task_id);
            if std::path::Path::new(&pid_file).exists() {
                ToolResult::ok(format!("Worker {} is running (found pid file)", task_id))
            } else {
                ToolResult::ok(format!("Worker {} not found", task_id))
            }
        }
    }

    fn list_workers(&self) -> ToolResult {
        let workers = self.active_workers.lock().unwrap();

        if workers.is_empty() {
            return ToolResult::ok("No active workers");
        }

        let mut output = String::new();
        for (id, info) in workers.iter() {
            output.push_str(&format!("{}: PID {} - {}\n", id, info.pid, info.task));
        }

        ToolResult::ok(output.trim().to_string())
    }
}
