pub mod search;
pub mod files;
pub mod reminders;
pub mod memory;
pub mod scripts;
pub mod bash;
pub mod worker;

pub use search::WebSearch;
pub use files::FileOps;
pub use reminders::Reminders;
pub use memory::Memory;
pub use scripts::Scripts;
pub use bash::Bash;
pub use worker::Worker;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
}

impl ToolResult {
    pub fn ok(output: impl Into<String>) -> Self {
        Self { success: true, output: output.into() }
    }
    pub fn err(output: impl Into<String>) -> Self {
        Self { success: false, output: output.into() }
    }
}

pub struct ToolExecutor {
    pub web_search: WebSearch,
    pub file_ops: FileOps,
    pub reminders: Reminders,
    pub memory: Memory,
    pub scripts: Scripts,
    pub bash: Bash,
    pub worker: Worker,
}

impl ToolExecutor {
    pub fn new() -> Self {
        Self {
            web_search: WebSearch::new(),
            file_ops: FileOps::new(),
            reminders: Reminders::new(),
            memory: Memory::new(),
            scripts: Scripts::new(),
            bash: Bash::new(),
            worker: Worker::new(),
        }
    }

    pub async fn execute(&self, response: &str) -> (Option<String>, Vec<String>) {
        let mut outputs = Vec::new();
        let text = response.to_lowercase().replace("speach", "speech");
        let orig = response.to_string();
        
        eprintln!("TOOL EXECUTE CALLED with: {}", &response[..response.len().min(200)]);
        
        if orig.contains("tool_calls") {
            if let Some(start) = orig.find("{\"tool_calls\":") {
                let slice = &orig[start..];
                
                let mut func_name = String::new();
                if let Some(n) = slice.find("\"name\":") {
                    let rest = &slice[n+7..];
                    for quote in &["\"", "\u{201C}", "\u{201D}"] {
                        if let Some(q1) = rest.find(quote) {
                            if let Some(q2) = rest[q1+1..].find(quote) {
                                func_name = rest[q1+1..q1+1+q2].to_string();
                                break;
                            }
                        }
                    }
                }
                
                let mut value = String::new();
                let mut path = String::new();
                let mut line = String::new();
                let mut command = String::new();
                if let Some(a) = slice.find("\"arguments\":") {
                    let args = &slice[a+12..];
                    let args = args.replace("\\\"", "\"").replace("\\\\", "\\");
                    
                    // content: memsave, file_write
                    if let Some(c) = args.find("\"content\":") {
                        let cv = &args[c+10..];
                        for quote in &["\"", "\u{201C}", "\u{201D}"] {
                            if let Some(q1) = cv.find(quote) {
                                if let Some(q2) = cv[q1+1..].find(quote) {
                                    value = cv[q1+1..q1+1+q2].to_string();
                                    break;
                                }
                            }
                        }
                    }
                    // query: websearch, memrecall
                    if value.is_empty() {
                        if let Some(q) = args.find("\"query\":") {
                            let qv = &args[q+8..];
                            for quote in &["\"", "\u{201C}", "\u{201D}"] {
                                if let Some(q1) = qv.find(quote) {
                                    if let Some(q2) = qv[q1+1..].find(quote) {
                                        value = qv[q1+1..q1+1+q2].to_string();
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    // command: execute_command
                    if value.is_empty() && command.is_empty() {
                        if let Some(c) = args.find("\"command\":") {
                            let cv = &args[c+10..];
                            for quote in &["\"", "\u{201C}", "\u{201D}"] {
                                if let Some(q1) = cv.find(quote) {
                                    if let Some(q2) = cv[q1+1..].find(quote) {
                                        command = cv[q1+1..q1+1+q2].to_string();
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    // path: file ops - handle both regular and curly quotes
                    if value.is_empty() || func_name == "file_edit" || func_name == "file_write" || func_name == "file_read" || func_name == "ls" {
                        for quote in &["\"", "\u{201C}", "\u{201D}"] {
                            if let Some(p) = args.find(&format!("\"path\": {}", quote)) {
                                let pv = &args[p+8..];
                                if let Some(q1) = pv.find(quote) {
                                    if let Some(q2) = pv[q1+1..].find(quote) {
                                        path = pv[q1+1..q1+1+q2].to_string();
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    // line: file_edit
                    if let Some(l) = args.find("\"line\":") {
                        let lv = &args[l+7..];
                        let le = lv.find(',').unwrap_or(lv.find('}').unwrap_or(lv.len()));
                        line = lv[..le].trim().to_string();
                    }
                    
                    // message: reminder - extract from arguments
                    let mut reminder_message = String::new();
                    let mut reminder_minutes: u64 = 60;
                    if func_name == "reminder" {
                        // Extract message
                        if let Some(m) = args.find("\"message\":") {
                            let mv = &args[m+10..];
                            for quote in &["\"", "\u{201C}", "\u{201D}"] {
                                if let Some(q1) = mv.find(quote) {
                                    if let Some(q2) = mv[q1+1..].find(quote) {
                                        reminder_message = mv[q1+1..q1+1+q2].to_string();
                                        break;
                                    }
                                }
                            }
                        }
                        // Extract minutes
                        if let Some(min_pos) = args.find("\"minutes\":") {
                            let min_str = &args[min_pos+10..];
                            let min_end = min_str.find(',').unwrap_or(min_str.find('}').unwrap_or(min_str.len()));
                            if let Ok(m) = min_str[..min_end].trim().parse::<u64>() {
                                reminder_minutes = m;
                            }
                        }
                        eprintln!("EXTRACTED: reminder message={} minutes={}", reminder_message, reminder_minutes);
                        let result = self.reminders.execute(&format!("set {} | {}", reminder_message, reminder_minutes));
                        outputs.push(format!("[{}] → {}", func_name, result.output));
                    }
                }
                
                if !value.is_empty() && func_name == "memsave" {
                    eprintln!("EXTRACTED: memsave={}", value);
                    let result = self.memory.execute(&format!("memsave:{}", value));
                    outputs.push(format!("[{}] → {}", func_name, result.output));
                } else if !value.is_empty() && func_name == "memrecall" {
                    eprintln!("EXTRACTED: memrecall={}", value);
                    let result = self.memory.execute(&format!("memrecall:{}", value));
                    outputs.push(format!("[{}] → {}", func_name, result.output));
                } else if !value.is_empty() && func_name == "websearch" {
                    eprintln!("EXTRACTED: websearch={}", value);
                    let result = self.web_search.execute(&value).await;
                    outputs.push(format!("[{}] → {}", func_name, result.output));
                } else if func_name == "file_edit" && !path.is_empty() {
                    if let Ok(line_num) = line.parse::<usize>() {
                        let content = value.clone();
                        eprintln!("EXTRACTED: file_edit path={} line={} content={}", path, line_num, content);
                        let result = self.file_ops.execute(&format!("EDIT: {}:{}|{}", path, line_num, content));
                        outputs.push(format!("[{}] → {}", func_name, result.output));
                    }
                } else if func_name == "file_write" && !path.is_empty() {
                    eprintln!("EXTRACTED: file_write path={} content={}", path, value);
                    let result = self.file_ops.execute(&format!("WRITE: {}|{}", path, value));
                    outputs.push(format!("[{}] → {}", func_name, result.output));
                } else if func_name == "file_read" && !path.is_empty() {
                    eprintln!("EXTRACTED: file_read={}", path);
                    let result = self.file_ops.execute(&format!("READ: {}", path));
                    outputs.push(format!("[{}] → {}", func_name, result.output));
                } else if func_name == "ls" && !path.is_empty() {
                    eprintln!("EXTRACTED: ls={}", path);
                    let result = self.file_ops.execute(&format!("LS: {}", path));
                    outputs.push(format!("[{}] → {}", func_name, result.output));
                } else if !value.is_empty() && func_name == "bash" {
                    eprintln!("EXTRACTED: bash={}", value);
                    let result = self.bash.execute(&format!("bash:\"{}\"", value));
                    outputs.push(format!("[{}] → {}", func_name, result.output));
                } else if func_name == "execute_command" && !command.is_empty() {
                    // execute_command - run a bash command
                    // (Timers should use reminder tool, but this runs any other command)
                    let cmd = format!("bash:\"{}\"", command);
                    eprintln!("EXTRACTED: execute_command={}", cmd);
                    let result = self.bash.execute(&cmd);
                    outputs.push(format!("[execute] → {}", result.output));
                } else if func_name == "script_run" && !path.is_empty() {
                    // script_run - run a script
                    eprintln!("EXTRACTED: script_run={}", path);
                    let result = self.scripts.execute(&format!("run:{}", path));
                    outputs.push(format!("[{}] → {}", func_name, result.output));
                } else if func_name == "worker" {
                    // worker - spawn, kill, or check background workers
                    let worker_cmd = if !command.is_empty() { command.clone() } else { value.clone() };
                    eprintln!("EXTRACTED: worker={}", worker_cmd);
                    let result = self.worker.execute(&worker_cmd);
                    outputs.push(format!("[{}] → {}", func_name, result.output));
                } else if func_name == "check_quota" {
                    // check_quota doesn't need value - it uses stored API key
                    eprintln!("EXTRACTED: check_quota");
                    outputs.push(format!("[{}] → Use the check_quota command directly in terminal to verify API key status", func_name));
                } else {
                    eprintln!("DEBUG: unhandled func_name={} value={} path={}", func_name, value, path);
                }
            }
        }
        
        if text.contains("memsave:") {
            if let Some(pos) = text.find("memsave:") {
                let content = text[pos + 8..].trim().to_string();
                if content.len() > 1 && content.len() < 100 {
                    let result = self.memory.execute(&format!("memsave:{}", content));
                    outputs.push(format!("[memsave] → {}", result.output));
                }
            }
        }
        
        // Handle reminder: format (fallback when model doesn't use tool_calls properly)
        // More aggressive - if ANYTHING mentions "reminder" + time, use it
        if text.contains("reminder") || text.contains("remind me") || text.contains("set a reminder") || text.contains("timer for") || text.contains(" secs") || text.contains(" seconds") || text.contains(" mins") || text.contains(" minutes") || text.contains(" hours") || text.contains(" hour") {
            // Try to extract message and time from the text
            let mut message = String::new();
            let mut minutes: u64 = 60;
            
            // Handle various time units
            let has_seconds = text.contains("second") || text.contains(" secs") || text.contains(" sec");
            let has_hours = text.contains("hour") || text.contains(" hrs") || text.contains(" hr");
            
            // Scan for number followed by time unit
            let words: Vec<&str> = text.split_whitespace().collect();
            for i in 0..words.len() {
                let word = words[i].trim_end_matches('.');
                if word.chars().all(|c| c.is_ascii_digit()) {
                    if let Ok(n) = word.parse::<u64>() {
                        if n < 10000 {
                            // Check next word for time unit
                            if i + 1 < words.len() {
                                let next = words[i + 1].to_lowercase();
                                if next.contains("sec") {
                                    minutes = (n as f64 / 60.0).ceil() as u64;
                                    if minutes == 0 { minutes = 1; }
                                } else if next.contains("hour") || next.contains("hr") {
                                    minutes = n * 60;
                                } else if next.contains("min") || next.contains("sec") {
                                    minutes = n;
                                }
                            }
                            break;
                        }
                    }
                }
            }
            
            // Also try regex-like approach - find digit followed by time word
            if minutes == 60 {
                let words: Vec<&str> = text.split_whitespace().collect();
                for i in 0..words.len() {
                    let word = words[i];
                    if word.chars().all(|c| c.is_ascii_digit()) {
                        if let Ok(n) = word.parse::<u64>() {
                            if n < 1000 {
                                // Check next word for time unit
                                if i + 1 < words.len() {
                                    let next = words[i + 1].to_lowercase();
                                    if next.contains("sec") {
                                        minutes = (n as f64 / 60.0).ceil() as u64;
                                        if minutes == 0 { minutes = 1; }
                                    } else if next.contains("hour") || next.contains("hr") {
                                        minutes = n * 60;
                                    } else if next.contains("min") || next.contains("sec") {
                                        minutes = n;
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
            
            // Try to find what the reminder is about - look for key phrases
            let orig_lower = text.to_lowercase();
            if let Some(start) = orig_lower.find("reminder") {
                let after = &orig_lower[start..];
                // Skip to "to" or "for" or "that" or "in" - common patterns
                if let Some(msg_start) = after.find("to ").map(|p| p + 3)
                    .or_else(|| after.find("for ").map(|p| p + 4))
                    .or_else(|| after.find("that ").map(|p| p + 5))
                    .or_else(|| after.find("in ").map(|p| p + 3))
                {
                    let msg_part = &after[msg_start..];
                    // Take first 50 chars or until "in" or "will" or "have" or "seconds" or "minutes"
                    message = msg_part.split(|c| c == ' ' || c == '.').take(8).collect::<Vec<_>>().join(" ");
                    message = message.replace("i ", "").replace("you ", "").replace("the ", "").replace("to ", "").replace("for ", "").trim().to_string();
                }
            }
            
            if message.is_empty() {
                message = "Reminder".to_string();
            }
            
            // Clean up message
            message = message.replace("minute", "").replace("min", "").replace("hour", "").replace("second", "").trim().to_string();
            
            if !message.is_empty() && message.len() < 100 {
                eprintln!("REMINDER FALLBACK: message={} minutes={}", message, minutes);
                let result = self.reminders.execute(&format!("set {} | {}", message, minutes));
                outputs.push(format!("[reminder] → {}", result.output));
            }
        }
        
        // Speech mode - simple toggle - fallback when model doesn't use tool_calls
        // Also catch common typos like "speach"
        let lower = text.to_lowercase().replace("speach", "speech");
        eprintln!("SPEECH CHECK: lower={}", lower);
        
        if lower.contains("speech") && (lower.contains(" on") || lower.contains("on ") || lower.contains("mode")) || lower.contains("voice on") || lower.contains("talk on") || lower.contains("voice mode") || lower.contains("talk mode") {
            eprintln!("FOUND: speech on pattern!");
            std::fs::write("/tmp/amor_speech_mode", "1").ok();
            outputs.push("__SPEECH_ON__".to_string());
        } else if lower.contains("speech") && (lower.contains(" off") || lower.contains("off ") || lower.contains("mode off")) || lower.contains("voice off") || lower.contains("talk off") || (lower.contains("exit") && std::path::Path::new("/tmp/amor_speech_mode").exists()) {
            eprintln!("FOUND: speech off pattern!");
            std::fs::remove_file("/tmp/amor_speech_mode").ok();
            outputs.push("__SPEECH_OFF__".to_string());
        }
        
        // Also check bash commands for speech mode
        if lower.contains("echo") && lower.contains("speech") {
            eprintln!("SPEECH via bash detected");
            std::fs::write("/tmp/amor_speech_mode", "1").ok();
            outputs.push("__SPEECH_ON__".to_string());
        }
        
        // Filter out tool results
        let clean: Vec<_> = orig.lines()
            .filter(|l| !l.contains("tool_call") && !l.contains("function") && 
                     !l.contains("arguments") && !l.contains("memsave") &&
                     !l.contains("memrecall") && !l.contains("websearch"))
            .filter(|l| !l.trim().is_empty())
            .collect();
        
        (Some(clean.join("\n")), outputs)
    }
    
    pub async fn check_api_quota(&self, api_key: &str) -> ToolResult {
        let client = reqwest::Client::new();
        
        let test_request = serde_json::json!({
            "model": "llama-3.1-8b-instant",
            "messages": [{"role": "user", "content": "hi"}],
            "max_tokens": 1
        });
        
        match client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&test_request)
            .send()
            .await
        {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    ToolResult::ok("API key is valid and working ✓")
                } else if status.as_u16() == 429 {
                    ToolResult::err("⚠️ RATE LIMIT reached - API key has hit its limit")
                } else if status.as_u16() == 401 {
                    ToolResult::err("🔑 AUTH ERROR - API key is invalid or expired")
                } else {
                    ToolResult::err(format!("API error: {}", status))
                }
            }
            Err(e) => ToolResult::err(format!("Connection error: {}", e)),
        }
    }
}

impl Default for ToolExecutor {
    fn default() -> Self { Self::new() }
}
