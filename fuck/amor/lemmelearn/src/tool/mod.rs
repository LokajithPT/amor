pub mod search;
pub mod files;
pub mod reminders;
pub mod memory;
pub mod scripts;
pub mod bash;
pub mod instagram;

pub use search::WebSearch;
pub use files::FileOps;
pub use reminders::Reminders;
pub use memory::Memory;
pub use scripts::Scripts;
pub use bash::Bash;
pub use instagram::Instagram;

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
    pub instagram: Instagram,
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
            instagram: Instagram::new(),
        }
    }

    pub async fn execute(&self, response: &str) -> (Option<String>, Vec<String>) {
        let mut outputs = Vec::new();
        let text = response.to_lowercase();
        let orig = response.to_string();
        
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
                    let cmd = format!("bash:\"{}\"", command);
                    eprintln!("EXTRACTED: execute_command={}", cmd);
                    let result = self.bash.execute(&cmd);
                    outputs.push(format!("[execute] → {}", result.output));
                } else if func_name == "script_run" && !path.is_empty() {
                    // script_run - run a script
                    eprintln!("EXTRACTED: script_run={}", path);
                    let result = self.scripts.execute(&format!("run:{}", path));
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
        
        if text.contains("websearch:") {
            let pos = text.find("websearch:").unwrap();
            let query = text[pos + 10..].trim().to_string();
            if !query.is_empty() {
                let result = self.web_search.execute(&query).await;
                outputs.push(format!("[websearch] → {}", result.output));
            }
        }
        
        if text.contains("bash:") || text.contains("execute:") {
            let pos = text.find("bash:").unwrap_or(text.find("execute:").unwrap_or(1000));
            let cmd = text[pos + 5..].trim().to_string();
            if !cmd.is_empty() {
                let result = self.bash.execute(&format!("bash:\"{}\"", cmd));
                outputs.push(format!("[bash] → {}", result.output));
            }
        }
        
        // Instagram tool
        if text.contains("instagram:") {
            let cmd = text[text.find("instagram:").unwrap() + 11..].trim().to_string();
            if !cmd.is_empty() {
                let result = self.instagram.execute(&cmd);
                outputs.push(format!("[instagram] → {}", result.output));
            }
        }
        
        if text.contains("file_read:") {
            if let Some(pos) = orig.find("file_read:\"") {
                let path = &orig[pos + 11..];
                let pe = path.find('"').unwrap_or(path.len());
                let p = path[..pe].trim();
                if !p.is_empty() {
                    let result = self.file_ops.execute(&format!("READ: {}", p));
                    outputs.push(format!("[file] → {}", result.output));
                }
            }
        }
        
        if text.contains("file_edit:") || text.contains("edit:\"") {
            if let Some(pos) = orig.find("file_edit:\"").or_else(|| orig.find("edit:\"")) {
                let rest = &orig[pos + 10..];
                if let Some(end) = rest.find('"') {
                    let args = &rest[..end];
                    if let Some((path_line, new_content)) = args.split_once('|') {
                        if let Some((path, line)) = path_line.split_once(':') {
                            if let Ok(line_num) = line.parse::<usize>() {
                                let result = self.file_ops.execute(&format!("EDIT: {}:{}|{}", path, line_num, new_content));
                                outputs.push(format!("[file] → {}", result.output));
                            }
                        }
                    }
                }
            }
        }
        
        if text.contains("ls:") {
            if let Some(pos) = orig.find("ls:\"") {
                let path = &orig[pos + 4..];
                let pe = path.find('"').unwrap_or(path.len());
                let p = path[..pe].trim();
                if !p.is_empty() {
                    let result = self.file_ops.execute(&format!("LS: {}", p));
                    outputs.push(format!("[ls] → {}", result.output));
                }
            }
        }
        
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
