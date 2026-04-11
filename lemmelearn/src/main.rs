use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::process;
use std::sync::Arc;

mod config;
mod tool;

use tool::ToolExecutor;
use teloxide::prelude::*;
use teloxide::Bot;

const PID_FILE: &str = "/tmp/amor.pid";
const TTS_ENABLED: bool = true;

fn extract_json_arg(args: &str, key: &str) -> Option<String> {
    let search = format!("\"{}\":", key);
    if let Some(pos) = args.find(&search) {
        let rest = &args[pos + search.len()..];
        for quote in &["\"", "\u{201C}", "\u{201D}"] {
            if let Some(q1) = rest.find(quote) {
                if let Some(q2) = rest[q1+1..].find(quote) {
                    return Some(rest[q1+1..q1+1+q2].to_string());
                }
            }
        }
    }
    None
}

fn is_command_tool_name(name: &str) -> bool {
    matches!(name, "execute_command" | "bash")
}

fn clean_output(text: &str) -> String {
    let mut result = text.to_string();
    result = result.replace("**", "");
    // Preserve speech mode markers - don't remove underscores around these
    if result.contains("SPEECH_ON") || result.contains("SPEECH_OFF") {
        return result;
    }
    result = result.replace("__", "");
    result = result.replace("*", "");
    result = result.replace("_", "");
    result = result.replace("# ", "");
    result = result.replace("## ", "");
    result = result.replace("### ", "");
    result = result.replace("`", "");
    result = result.replace("~~", "");
    // Strip thinking tags like <think> and 
    while let Some(start) = result.find("<think>") {
        if let Some(end) = result[start..].find("") {
            result = format!("{}{}", &result[..start], &result[start+end+9..]);
        } else {
            break;
        }
    }
    // Strip markdown code blocks
    while let Some(start) = result.find("```") {
        if let Some(end) = result[start+3..].find("```") {
            result = format!("{}{}", &result[..start], &result[start+3+end+3..]);
        } else {
            break;
        }
    }
    while let Some(start) = result.find('[') {
        if let Some(end) = result[start..].find(")(") {
            if let Some(paren_end) = result[start + end..].find(')') {
                result = format!("{}{}", &result[..start], &result[start + end + paren_end + 1..]);
            } else {
                break;
            }
        } else {
            break;
        }
    }
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }
    result.trim().to_string()
}

const RED: &str = "\x1b[91m";
const GREEN: &str = "\x1b[92m";
const YELLOW: &str = "\x1b[93m";
const BLUE: &str = "\x1b[94m";
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";

fn speak(text: &str) {
    // Only speak if speech mode is enabled (file exists)
    if !std::path::Path::new("/tmp/amor_speech_mode").exists() {
        return;
    }
    
    eprintln!("SPEAK CALLED: {}", text);
    
    // Use nohup to run in background with timeout
    let cmd = format!(
        "nohup /home/fuckall/tars_voice/venv/bin/python /home/fuckall/code/amor/lemmelearn/amorshi/scripts/tars_speak.py '{}' > /dev/null 2>&1 &",
        text.replace("'", "\\'")
    );
    
    std::process::Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .spawn()
        .ok();
}

fn set_speech_mode(enabled: bool) {
    let path = "/tmp/amor_speech_mode";
    if enabled {
        std::fs::write(path, "1").ok();
    } else {
        std::fs::remove_file(path).ok();
    }
}

fn send_whatsapp_message(to: &str, message: &str) {
    // Write to out queue - whatsapp bot reads this and sends
    let out_queue = "/tmp/whatsapp_out.txt";
    let line = format!("{}|{}\n", to, message);
    std::fs::write(out_queue, line).ok();
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChatMsg { role: String, content: String }

#[derive(Deserialize, Debug)]
struct ChatResponse { choices: Vec<Choice> }

#[derive(Deserialize, Debug)]
struct Choice { message: ChatMsg, #[serde(default)] tool_calls: Vec<ToolCall> }

#[derive(Deserialize, Debug, Clone)]
struct ToolCall { id: Option<String>, #[serde(rename = "type")] call_type: Option<String>, function: ToolFunction }

#[derive(Deserialize, Debug, Clone)]
struct ToolFunction { name: String, arguments: String }

#[derive(Clone)]
struct AppState {
    config: config::Config,
    model: String,
    master: String,
}

fn write_pid(pid: u32) {
    if let Ok(mut f) = std::fs::File::create(PID_FILE) {
        writeln!(f, "{}", pid).ok();
    }
}

fn read_pid() -> Option<u32> {
    std::fs::read_to_string(PID_FILE).ok()?.trim().parse().ok()
}

fn kill_service_for_console() {
    if let Some(pid) = read_pid() {
        println!("Found running service (PID: {}), killing it...", pid);
        if process::Command::new("kill").arg(pid.to_string()).output().is_ok() {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        std::fs::remove_file(PID_FILE).ok();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("chat");

    match command {
        "serve" | "start" => {
            run_telegram_service();
        }
        "kill" => {
            if let Some(pid) = read_pid() {
                process::Command::new("kill").arg(pid.to_string()).output().ok();
                std::fs::remove_file(PID_FILE).ok();
            }
            println!("Service killed.");
        }
        _ => {
            run_console_mode();
        }
    }
}

fn run_telegram_service() {
    let amorshi = match config::AmorshiFiles::load() {
        Ok(f) => f, Err(e) => { eprintln!("Error loading amorshi: {}", e); std::process::exit(1); }
    };

    let config = amorshi.config.clone();
    let model = amorshi.config.model.clone();
    let master = amorshi.master.clone();

    println!("{}=== AMOR Service ==={}", BOLD, RESET);
    println!("{}Model: {}{}", BLUE, model, RESET);
    println!("{}API: {}{} ({} keys)", BLUE, config.active_account, RESET, config.accounts.len());

    let tg_token = config.telegram_bot_token.clone();

    if tg_token.is_none() {
        eprintln!("❌ No Telegram token configured!");
        std::process::exit(1);
    }

    let state = AppState { config, model, master };
    let state = Arc::new(state);

    write_pid(process::id());

    println!("{}🤖 Telegram bot running (24/7)...{}", GREEN, RESET);
    println!("{}PID: {} saved to {}{}", BLUE, process::id(), PID_FILE, RESET);

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        run_telegram_loop(state).await;
    });
}

async fn run_telegram_loop(state: Arc<AppState>) {
    let tg_token = state.config.telegram_bot_token.clone().unwrap();
    let mut last_update_id = 0i64;

    loop {
        // Check WhatsApp queue
        let whatsapp_queue = "/tmp/whatsapp_in.txt";
        if std::path::Path::new(whatsapp_queue).exists() {
            if let Ok(content) = std::fs::read_to_string(whatsapp_queue) {
                let lines: Vec<&str> = content.lines().collect();
                for line in lines {
                    if let Some((source, rest)) = line.split_once('|') {
                        if source == "WHATSAPP" {
                            if let Some((from, msg)) = rest.split_once('|') {
                                println!("{}📱 WA:{} {}", YELLOW, RESET, msg);
                                
                                let response = clean_output(&process_telegram_message(&state, msg).await);
                                
                                // Send back via WhatsApp
                                send_whatsapp_message(&from, &response);
                                
                                println!("{}📤 WA Sent response{}", GREEN, RESET);
                                if std::path::Path::new("/tmp/amor_speech_mode").exists() {
                                    speak(&response);
                                }
                            }
                        }
                    }
                }
                std::fs::remove_file(whatsapp_queue).ok();
            }
        }
        
        // Check reminders
        let mut reminder_tool = tool::Reminders::new();
        let due = reminder_tool.check_due();
        for msg in due {
            eprintln!("⏰ REMINDER DUE: {}", msg);
            speak(&msg);
            
            let chat_id = state.config.telegram_chat_id.as_deref().unwrap_or("8722256254");
            let send_url = format!("https://api.telegram.org/bot{}/sendMessage", tg_token);
            let send_data = format!("chat_id={}&text={}", chat_id, format!("⏰ REMINDER: {}", msg).replace(" ", "%20"));
            std::process::Command::new("curl")
                .arg("-s")
                .arg("-X")
                .arg("POST")
                .arg("-d")
                .arg(&send_data)
                .arg(&send_url)
                .output()
                .ok();
        }

        // Get updates using curl for reliability
        let mut url = format!("https://api.telegram.org/bot{}/getUpdates?timeout=1", tg_token);
        if last_update_id > 0 {
            url = format!("https://api.telegram.org/bot{}/getUpdates?timeout=1&offset={}", tg_token, last_update_id + 1);
        }

        let output = std::process::Command::new("curl")
            .arg("-s")
            .arg(&url)
            .output();

        if let Ok(output) = output {
            if let Ok(update_resp) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(results) = update_resp.get("result").and_then(|r| r.as_array()) {
                    for update in results {
                        if let Some(msg) = update.get("message") {
                            // Check for voice message first
                            let mut input_text = None;
                            
                            // Check for voice/audio
                            if let Some(voice) = msg.get("voice") {
                                if let Some(file_id) = voice.get("file_id").and_then(|f| f.as_str()) {
                                    // Download voice file
                                    let file_url = format!("https://api.telegram.org/bot{}/getFile?file_id={}", tg_token, file_id);
                                    if let Ok(file_resp) = std::process::Command::new("curl").arg("-s").arg(&file_url).output() {
                                        if let Ok(file_json) = serde_json::from_str::<serde_json::Value>(&String::from_utf8_lossy(&file_resp.stdout)) {
                                            if let Some(file_path) = file_json.get("result").and_then(|r| r.get("file_path")).and_then(|p| p.as_str()) {
                                                let download_url = format!("https://api.telegram.org/file/bot{}/{}", tg_token, file_path);
                                                let ogg_path = "/tmp/amor_voice.ogg";
                                                std::process::Command::new("curl")
                                                    .arg("-s")
                                                    .arg("-o")
                                                    .arg(ogg_path)
                                                    .arg(&download_url)
                                                    .output()
                                                    .ok();
                                                
                                                // Convert to text using whisper
                                                if let Ok(whisper_out) = std::process::Command::new("whisper")
                                                    .arg("--model")
                                                    .arg("base")
                                                    .arg("--language")
                                                    .arg("english")
                                                    .arg("--output-txt")
                                                    .arg("--no-gpu")
                                                    .arg(ogg_path)
                                                    .output()
                                                {
                                                    if let Ok(text) = String::from_utf8(whisper_out.stdout) {
                                                        let txt_file = "/tmp/amor_voice.txt";
                                                        input_text = std::fs::read_to_string(txt_file).ok();
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // If no voice, check for text
                            if input_text.is_none() {
                                if let Some(text) = msg.get("text").and_then(|t| t.as_str()) {
                                    input_text = Some(text.to_string());
                                }
                            }
                            
                            if let Some(text) = input_text {
                                let chat_id = msg.get("chat").and_then(|c| c.get("id")).and_then(|i| i.as_i64()).unwrap_or(0);
                                
                                println!("{}📱 TG:{} {}", YELLOW, RESET, text);
                                
                                let response = clean_output(&process_telegram_message(&state, &text, chat_id).await);
                                
                                // Send response using curl
                                let send_url = format!("https://api.telegram.org/bot{}/sendMessage", tg_token);
                                let send_data = format!("chat_id={}&text={}", chat_id, response.replace(" ", "%20"));
                                std::process::Command::new("curl")
                                    .arg("-s")
                                    .arg("-X")
                                    .arg("POST")
                                    .arg("-d")
                                    .arg(&send_data)
                                    .arg(&send_url)
                                    .output()
                                    .ok();
                                
                                println!("{}📤 Sent response{}", GREEN, RESET);
                                if std::path::Path::new("/tmp/amor_speech_mode").exists() {
                                    speak(&response);
                                }
                            }
                        }
                        if let Some(update_id) = update.get("update_id").and_then(|u| u.as_i64()) {
                            last_update_id = update_id;
                        }
                    }
                }
            }
        }
        
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
}

fn run_console_mode() {
    kill_service_for_console();

    let amorshi = match config::AmorshiFiles::load() {
        Ok(f) => f, Err(e) => { eprintln!("Error loading amorshi: {}", e); std::process::exit(1); }
    };

    let mut config = amorshi.config.clone();
    let model = amorshi.config.model.clone();
    let master = amorshi.master.clone();

    println!("{}=== AMOR Console ==={}", BOLD, RESET);
    println!("{}Model: {}{}", BLUE, model, RESET);
    println!("{}API: {}{} ({} keys)\n", BLUE, config.active_account, RESET, config.accounts.len());

    tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(async {
        run_console_chat(&mut config, model, master).await.ok();
    });

    println!("\n🔄 Restarting service...");
    restart_service();
}

fn restart_service() {
    let binary = std::env::current_exe().unwrap_or_default();
    if binary.to_string_lossy().contains("amor") {
        std::process::Command::new(&binary)
            .arg("serve")
            .spawn()
            .ok();
        
        if let Some(pid) = std::process::Command::new("pgrep")
            .arg("-f")
            .arg("amor serve")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.trim().parse().ok())
        {
            write_pid(pid);
            println!("✅ Service restarted (PID: {})", pid);
        }
    }
}

async fn process_telegram_message(state: &AppState, input: &str, chat_id: i64) -> String {
    let mut chat_hist: Vec<ChatMsg> = Vec::new();
    let tool_executor = ToolExecutor::new();
    
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let base_path = std::path::PathBuf::from(format!("{}/amorshi", if home == "/home/fuckall" { "/home/fuckall" } else { &home }));
    
    // Read global memory.md
    let global_memory_path = base_path.join("memory.md");
    let global_memory = std::fs::read_to_string(&global_memory_path).unwrap_or_default();
    
    // Read user-specific memory
    let user_memory_path = base_path.join("users").join(format!("{}.md", chat_id));
    let user_memory = std::fs::read_to_string(&user_memory_path).unwrap_or_default();
    
    // Check if user has tool access (for now, only chat_id 5678901234 has tools)
    let has_tools = chat_id == 5678901234;
    
    // Build memory section
    let mut memory_section = String::new();
    if !global_memory.trim().is_empty() {
        memory_section.push_str(&format!("\n## GLOBAL MEMORY:\n{}\n", global_memory.trim()));
    }
    if !user_memory.trim().is_empty() {
        memory_section.push_str(&format!("\n## THIS USER'S MEMORY:\n{}\n", user_memory.trim()));
    }
    
    // Build system prompt based on user
    let mut full_system = state.master.clone();
    full_system.push_str(&memory_section);
    
    // Add user identity section
    let user_section = format!("\n## CURRENT USER\nChat ID: {}\nTool Access: {}\n", chat_id, if has_tools { "FULL" } else { "NONE - conversational only" });
    full_system.push_str(&user_section);
    
    chat_hist.push(ChatMsg { role: "system".to_string(), content: full_system });
    chat_hist.push(ChatMsg { role: "user".to_string(), content: input.to_string() });
    
    let mut config = state.config.clone();
    let model = state.model.clone();
    let mut loop_count = 0;
    
    while loop_count < 3 {
        loop_count += 1;
        if chat_hist.len() >= 10 {
            let sys = chat_hist.first().cloned();
            let rec: Vec<_> = chat_hist.iter().rev().take(6).cloned().collect();
            chat_hist.clear();
            if let Some(s) = sys { chat_hist.push(s); }
            chat_hist.extend(rec.into_iter().rev());
        }
        
        let api_key = match config.get_active_key() {
            Some(k) => k.clone(),
            None => return "No API key available".to_string(),
        };
        
        // Build JSON payload - escape all special chars properly
        let messages_json = chat_hist.iter().map(|m| {
            let escaped = m.content
                .replace("\\", "\\\\")
                .replace("\"", "\\\"")
                .replace("\n", "\\n")
                .replace("\r", "\\r")
                .replace("\t", "\\t");
            format!("{{\"role\":\"{}\",\"content\":\"{}\"}}", m.role, escaped)
        }).collect::<Vec<_>>().join(",");
        let payload = format!("{{\"model\":\"{}\",\"messages\":[{}],\"temperature\":0.7}}", model, messages_json);
        
        // Use curl instead of reqwest
        let output = std::process::Command::new("curl")
            .arg("-s")
            .arg("--max-time")
            .arg("25")
            .arg("-X")
            .arg("POST")
            .arg("https://api.groq.com/openai/v1/chat/completions")
            .arg("-H")
            .arg(format!("Authorization: Bearer {}", api_key))
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-d")
            .arg(&payload)
            .output();
        
        let body_str = match output {
            Ok(o) => {
                if !o.status.success() {
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    return format!("Error: {}", stderr);
                }
                String::from_utf8_lossy(&o.stdout).to_string()
            }
            Err(e) => return format!("Error: {}", e),
        };
        
        let body: ChatResponse = match serde_json::from_str(&body_str) { 
            Ok(r) => r, 
            Err(e) => {
                // Log first 300 chars of response for debugging
                eprintln!("GROQ Parse error: {} - Response: {}", e, &body_str[..body_str.len().min(300)]);
                return format!("Parse error: {}", e);
            }
        };
        
        let Some(choice) = body.choices.get(0) else { break };
        let content = choice.message.content.clone();
        
        // Parse simple tool call format: "websearch: query", "bash: command", etc
        let mut manual_calls: Vec<(String, String)> = Vec::new();
        let lines: Vec<&str> = content.split('\n').collect();
        eprintln!("DEBUG: Checking lines for tool calls, total lines: {}", lines.len());
        for line in lines {
            let line = line.trim();
            eprintln!("DEBUG: Checking line: {}", line);
            if line.starts_with("websearch:") {
                let query = line[10..].trim();
                eprintln!("DEBUG: Found websearch with query: {}", query);
                if !query.is_empty() {
                    manual_calls.push(("websearch".to_string(), format!(r#"{{"query":"{}"}}"#, query)));
                }
            } else if line.starts_with("bash:") {
                let cmd = line[5..].trim();
                eprintln!("DEBUG: Found bash with command: {}", cmd);
                if !cmd.is_empty() {
                    manual_calls.push(("bash".to_string(), format!(r#"{{"command":"{}"}}"#, cmd)));
                }
            } else if line.starts_with("memsave:") {
                let content = line[8..].trim();
                eprintln!("DEBUG: Found memsave with content: {}", content);
                if !content.is_empty() {
                    manual_calls.push(("memsave".to_string(), format!(r#"{{"content":"{}"}}"#, content)));
                }
            } else if line.starts_with("memrecall:") {
                let query = line[9..].trim();
                eprintln!("DEBUG: Found memrecall with query: {}", query);
                if !query.is_empty() {
                    manual_calls.push(("memrecall".to_string(), format!(r#"{{"query":"{}"}}"#, query)));
                }
            }
        }
        eprintln!("DEBUG: manual_calls parsed: {:?}", manual_calls);
        
        let only_structured_commands =
            !choice.tool_calls.is_empty()
                && choice
                    .tool_calls
                    .iter()
                    .all(|tc| is_command_tool_name(tc.function.name.as_str()));
        let only_manual_commands =
            !manual_calls.is_empty() && manual_calls.iter().all(|(name, _)| name == "bash");
        
        // Check if user has tool access
        let user_has_tools = chat_id == 5678901234;
        let tool_calls_requested = !choice.tool_calls.is_empty() || !manual_calls.is_empty();
        
        // If tools requested but user doesn't have access, skip tool execution
        let has_tools = if tool_calls_requested && !user_has_tools {
            false
        } else {
            tool_calls_requested
        };
        
        let mut outs = if has_tools { 
            let mut o = Vec::new(); 
            for tc in &choice.tool_calls { 
                let func_name = tc.function.name.as_str();
                let args = tc.function.arguments.as_str();
                
                if func_name == "memsave" || func_name == "save" {
                    if let Some(v) = extract_json_arg(args, "content").or_else(|| extract_json_arg(args, "value")) {
                        o.push(tool_executor.memory.execute(&format!("memsave:{}", v)).output); 
                    }
                } else if func_name == "memrecall" || func_name == "recall" {
                    if let Some(v) = extract_json_arg(args, "query").or_else(|| extract_json_arg(args, "content")) {
                        o.push(tool_executor.memory.execute(&format!("memrecall:{}", v)).output); 
                    }
                } else if func_name == "execute_command" || func_name == "bash" {
                    if let Some(cmd) = extract_json_arg(args, "command") {
                        o.push(tool_executor.bash.execute(&format!("bash:\"{}\"", cmd)).output); 
                    }
                } else if func_name == "websearch" || func_name == "search" {
                    if let Some(q) = extract_json_arg(args, "query") {
                        o.push(tokio::runtime::Runtime::new().unwrap().block_on(tool_executor.web_search.execute(&q)).output); 
                    }
                } else if func_name == "reminder" || func_name == "remind" {
                    let action = extract_json_arg(args, "action").unwrap_or_default();
                    if action == "list" {
                        o.push(tool_executor.reminders.execute("list").output);
                    } else if action == "delete" {
                        if let Some(id) = extract_json_arg(args, "id") {
                            o.push(tool_executor.reminders.execute(&format!("delete {}", id)).output);
                        }
                    } else {
                        let msg = extract_json_arg(args, "message").unwrap_or_default();
                        let mins = extract_json_arg(args, "minutes").or_else(|| extract_json_arg(args, "time")).unwrap_or_else(|| "60".to_string());
                        o.push(tool_executor.reminders.execute(&format!("set {} | {}", msg, mins)).output);
                    }
                }
            } 
            eprintln!("DEBUG: Processing {} manual calls (console)", manual_calls.len());
            // Process manual tool calls from text
            for (name, args) in manual_calls {
                let fname = name.as_str();
                let a = args.as_str();
                if fname == "websearch" || fname == "search" {
                    if let Some(q) = extract_json_arg(a, "query") {
                        o.push(tokio::runtime::Runtime::new().unwrap().block_on(tool_executor.web_search.execute(&q)).output);
                    }
                } else if fname == "bash" {
                    if let Some(c) = extract_json_arg(a, "command") {
                        o.push(tool_executor.bash.execute(&format!("bash:\"{}\"", c)).output);
                    }
                }
            }
            o 
        } else { 
            tool_executor.execute(&content).await.1 
        };
        
        // Check for speech mode in original response too (before tool results)
        let lower = content.to_lowercase().replace("speach", "speech");
        if lower.contains("speech") && (lower.contains(" on") || lower.contains("on ") || lower.contains("mode")) {
            eprintln!("FOUND: speech on in raw content!");
            std::fs::write("/tmp/amor_speech_mode", "1").ok();
            outs.push("__SPEECH_ON__".to_string());
        } else if lower.contains("speech") && (lower.contains(" off") || lower.contains("off ")) {
            eprintln!("FOUND: speech off in raw content!");
            std::fs::remove_file("/tmp/amor_speech_mode").ok();
            outs.push("__SPEECH_OFF__".to_string());
        }
        
        // Check for speech mode toggle
        eprintln!("DEBUG: outs = {:?}", outs);
        let speech_on = outs.iter().any(|o| o.contains("__SPEECH_ON__"));
        let speech_off = outs.iter().any(|o| o.contains("__SPEECH_OFF__"));
        
        // Telegram loop will handle speaking when speech mode is active
        
        // Also always speak if speech mode file exists (for regular responses)
        let speech_mode_active = std::path::Path::new("/tmp/amor_speech_mode").exists();
        if speech_mode_active && !speech_on && !speech_off && !content.trim().is_empty() {
            // Don't speak every response, only on explicit ask
        }
        
        if outs.is_empty() { 
            chat_hist.push(ChatMsg { role: "assistant".to_string(), content: content.clone() }); 
            
            // Update user memory file with this conversation
            let user_memory_path = base_path.join("users").join(format!("{}.md", chat_id));
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let user_memory_entry = format!("### {}\n- User: {}\n- AMOR: {}\n\n", 
                timestamp, input, content.trim());
            
            let current_user_mem = std::fs::read_to_string(&user_memory_path).unwrap_or_default();
            let updated_user_mem = format!("{}{}", current_user_mem, user_memory_entry);
            std::fs::write(&user_memory_path, &updated_user_mem).ok();
            
            return content; 
        }

        if only_structured_commands || only_manual_commands {
            return outs.join("\n");
        }
        
        chat_hist.push(ChatMsg { role: "user".to_string(), content: format!("Result: {}", outs.join("\n")) });
    }
    
    "No response".to_string()
}

async fn run_console_chat(config: &mut config::Config, model: String, master: String) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut chat_hist: Vec<ChatMsg> = Vec::new();
    let tool_executor = ToolExecutor::new();

    // Read memory.md and include in system prompt
    let home = std::env::var("HOME").unwrap_or_default();
    let memory_path = if home == "/home/fuckall" {
        std::path::PathBuf::from("/home/fuckall/amorshi/memory.md")
    } else {
        std::path::Path::new(&home).join("amorshi/memory.md")
    };
    let memory_content = std::fs::read_to_string(&memory_path).unwrap_or_default();
    
    let memory_section = if !memory_content.trim().is_empty() {
        format!("\n\n## YOUR MEMORY (from past conversations):\n{}\n", memory_content.trim())
    } else {
        String::new()
    };
    
    let full_system = format!("{}{}", master, memory_section);
    
    chat_hist.push(ChatMsg { role: "system".to_string(), content: full_system });

    println!("{}=== AMOR Running ==={}\n", BOLD, RESET);

    loop {
        print!("{}> {}", BLUE, RESET); io::stdout().flush()?; 
        let mut inp = String::new(); 
        io::stdin().read_line(&mut inp)?; 
        let inp = inp.trim();
        
        if inp.is_empty() || inp == "exit" { println!("{}Bye!{}", GREEN, RESET); break; }
        
        chat_hist.push(ChatMsg { role: "user".to_string(), content: inp.to_string() });
        let response = process_console_message(&client, config, &mut chat_hist, &tool_executor, &model).await;
        println!("\n{}{}{}\n", GREEN, response, RESET);
    }
    Ok(())
}

async fn process_console_message(_client: &reqwest::Client, config: &mut config::Config, chat_hist: &mut Vec<ChatMsg>, tool_executor: &ToolExecutor, model: &str) -> String {
    let mut loop_count = 0;
    while loop_count < 3 {
        loop_count += 1;
        if chat_hist.len() >= 10 {
            let sys = chat_hist.first().cloned();
            let rec: Vec<_> = chat_hist.iter().rev().take(6).cloned().collect();
            chat_hist.clear();
            if let Some(s) = sys { chat_hist.push(s); }
            chat_hist.extend(rec.into_iter().rev());
        }
        let api_key = config.get_active_key().expect("No key").clone();
        
        // Build JSON payload - escape all special chars properly
        let messages_json = chat_hist.iter().map(|m| {
            let escaped = m.content
                .replace("\\", "\\\\")
                .replace("\"", "\\\"")
                .replace("\n", "\\n")
                .replace("\r", "\\r")
                .replace("\t", "\\t");
            format!("{{\"role\":\"{}\",\"content\":\"{}\"}}", m.role, escaped)
        }).collect::<Vec<_>>().join(",");
        let payload = format!("{{\"model\":\"{}\",\"messages\":[{}],\"temperature\":0.7}}", model, messages_json);
        
        let output = std::process::Command::new("curl")
            .arg("-s")
            .arg("--max-time")
            .arg("25")
            .arg("-X")
            .arg("POST")
            .arg("https://api.groq.com/openai/v1/chat/completions")
            .arg("-H")
            .arg(format!("Authorization: Bearer {}", api_key))
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-d")
            .arg(&payload)
            .output();
        
        let body_str = match output {
            Ok(o) => {
                if !o.status.success() {
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    return format!("Error: {}", stderr);
                }
                String::from_utf8_lossy(&o.stdout).to_string()
            }
            Err(e) => return format!("Error: {}", e),
        };
        
        let body: ChatResponse = match serde_json::from_str(&body_str) { Ok(r) => r, Err(e) => return format!("Parse: {}", e) };
        let Some(choice) = body.choices.get(0) else { break };
        let content = choice.message.content.clone();
        
        // Parse simple tool call format: "websearch: query", "bash: command", etc
        let mut manual_calls: Vec<(String, String)> = Vec::new();
        let lines: Vec<&str> = content.split('\n').collect();
        for line in lines {
            let line = line.trim();
            if line.starts_with("websearch:") {
                let query = line[10..].trim();
                if !query.is_empty() {
                    manual_calls.push(("websearch".to_string(), format!(r#"{{"query":"{}"}}"#, query)));
                }
            } else if line.starts_with("bash:") {
                let cmd = line[5..].trim();
                if !cmd.is_empty() {
                    manual_calls.push(("bash".to_string(), format!(r#"{{"command":"{}"}}"#, cmd)));
                }
            } else if line.starts_with("memsave:") {
                let c = line[8..].trim();
                if !c.is_empty() {
                    manual_calls.push(("memsave".to_string(), format!(r#"{{"content":"{}"}}"#, c)));
                }
            } else if line.starts_with("memrecall:") {
                let q = line[9..].trim();
                if !q.is_empty() {
                    manual_calls.push(("memrecall".to_string(), format!(r#"{{"query":"{}"}}"#, q)));
                }
            }
        }
        
        let only_structured_commands =
            !choice.tool_calls.is_empty()
                && choice
                    .tool_calls
                    .iter()
                    .all(|tc| is_command_tool_name(tc.function.name.as_str()));
        let only_manual_commands =
            !manual_calls.is_empty() && manual_calls.iter().all(|(name, _)| name == "bash");
        let has_tools = !choice.tool_calls.is_empty() || !manual_calls.is_empty();
        let outs = if has_tools { 
            let mut o = Vec::new(); 
            for tc in &choice.tool_calls { 
                let func_name = tc.function.name.as_str();
                let args = tc.function.arguments.as_str();
                
                if func_name == "memsave" || func_name == "save" {
                    if let Some(v) = extract_json_arg(args, "content").or_else(|| extract_json_arg(args, "value")) {
                        o.push(tool_executor.memory.execute(&format!("memsave:{}", v)).output); 
                    }
                } else if func_name == "memrecall" || func_name == "recall" {
                    if let Some(v) = extract_json_arg(args, "query").or_else(|| extract_json_arg(args, "content")) {
                        o.push(tool_executor.memory.execute(&format!("memrecall:{}", v)).output); 
                    }
                } else if func_name == "execute_command" || func_name == "bash" {
                    if let Some(cmd) = extract_json_arg(args, "command") {
                        o.push(tool_executor.bash.execute(&format!("bash:\"{}\"", cmd)).output); 
                    }
                } else if func_name == "websearch" || func_name == "search" {
                    if let Some(q) = extract_json_arg(args, "query") {
                        o.push(tokio::runtime::Runtime::new().unwrap().block_on(tool_executor.web_search.execute(&q)).output); 
                    }
                } else if func_name == "reminder" || func_name == "remind" {
                    let action = extract_json_arg(args, "action").unwrap_or_default();
                    if action == "list" {
                        o.push(tool_executor.reminders.execute("list").output);
                    } else if action == "delete" {
                        if let Some(id) = extract_json_arg(args, "id") {
                            o.push(tool_executor.reminders.execute(&format!("delete {}", id)).output);
                        }
                    } else {
                        let msg = extract_json_arg(args, "message").unwrap_or_default();
                        let mins = extract_json_arg(args, "minutes").or_else(|| extract_json_arg(args, "time")).unwrap_or_else(|| "60".to_string());
                        o.push(tool_executor.reminders.execute(&format!("set {} | {}", msg, mins)).output);
                    }
                }
            } 
            eprintln!("DEBUG: Processing {} manual calls (console)", manual_calls.len());
            // Process manual tool calls from text
            for (name, args) in manual_calls {
                let fname = name.as_str();
                let a = args.as_str();
                if fname == "websearch" || fname == "search" {
                    if let Some(q) = extract_json_arg(a, "query") {
                        o.push(tokio::runtime::Runtime::new().unwrap().block_on(tool_executor.web_search.execute(&q)).output);
                    }
                } else if fname == "bash" {
                    if let Some(c) = extract_json_arg(a, "command") {
                        o.push(tool_executor.bash.execute(&format!("bash:\"{}\"", c)).output);
                    }
                }
            }
            o 
        } else { tool_executor.execute(&content).await.1 };
        if outs.is_empty() { chat_hist.push(ChatMsg { role: "assistant".to_string(), content: content.clone() }); return content; }
        if only_structured_commands || only_manual_commands {
            return outs.join("\n");
        }
        chat_hist.push(ChatMsg { role: "user".to_string(), content: format!("Result: {}", outs.join("\n")) });
    }
    "No response".to_string()
}
