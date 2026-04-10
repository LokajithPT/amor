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

fn clean_output(text: &str) -> String {
    let mut result = text.to_string();
    result = result.replace("**", "");
    result = result.replace("__", "");
    result = result.replace("*", "");
    result = result.replace("_", "");
    result = result.replace("# ", "");
    result = result.replace("## ", "");
    result = result.replace("### ", "");
    result = result.replace("`", "");
    result = result.replace("~~", "");
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
    let lower = text.to_lowercase();
    if !lower.contains("remind") && !lower.contains("alert") && !lower.contains("notice") && !lower.contains("deadline") {
        return;
    }
    
    let text = text.replace("AMOR", "A M O R").replace('"', "'");
    std::process::Command::new("bash")
        .arg("-c")
        .arg(format!(
            "export LD_LIBRARY_PATH=/home/fuckall/tools/piper:$LD_LIBRARY_PATH; echo '{}' | /home/fuckall/tools/piper/piper --model /home/fuckall/.local/share/piper/voices/en_US-ryan-high.onnx --length_scale 1.1 --noise_scale 0.35 --output-file /tmp/amor_tts.wav && aplay /tmp/amor_tts.wav 2>/dev/null",
            text
        ))
        .spawn()
        .ok();
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
        "test" => {
            let query = args.get(2).map(|s| s.as_str()).unwrap_or("");
            if query.is_empty() {
                println!("Usage: ./amor test \"your query here\"");
                return;
            }
            run_test_mode(query);
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

fn run_test_mode(query: &str) {
    kill_service_for_console();
    
    let amorshi = match config::AmorshiFiles::load() {
        Ok(f) => f, Err(e) => { eprintln!("Error loading amorshi: {}", e); std::process::exit(1); }
    };
    
    let mut config = amorshi.config.clone();
    let model = amorshi.config.model.clone();
    let master = amorshi.master.clone();
    
    println!("=== AMOR Test ===");
    println!("Query: {}\n", query);
    
    // Just run ONE request, don't use interactive console
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let client = reqwest::Client::new();
        let mut chat_hist: Vec<ChatMsg> = Vec::new();
        let tool_executor = tool::ToolExecutor::new();
        
        chat_hist.push(ChatMsg { role: "system".to_string(), content: master });
        chat_hist.push(ChatMsg { role: "user".to_string(), content: query.to_string() });
        
        // Single API call - use process_telegram_message style
        let api_key = match config.get_active_key() {
            Some(k) => k.clone(),
            None => { eprintln!("No API key!"); std::process::exit(1); }
        };
        
        let messages_json = chat_hist.iter().map(|m| {
            let escaped = m.content.replace("\\", "\\\\").replace("\"", "\\\"").replace("\n", "\\n");
            format!("{{\"role\":\"{}\",\"content\":\"{}\"}}", m.role, escaped)
        }).collect::<Vec<_>>().join(",");
        
        let payload = format!("{{\"model\":\"{}\",\"messages\":[{}],\"temperature\":0.7}}", model, messages_json);
        
        let output = std::process::Command::new("curl")
            .arg("-s")
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
            Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
            Err(e) => { eprintln!("Curl error: {}", e); std::process::exit(1); }
        };
        
        let body: serde_json::Value = match serde_json::from_str(&body_str) {
            Ok(b) => b,
            Err(e) => { eprintln!("Parse error: {} - {}", e, &body_str[..body_str.len().min(200)]); std::process::exit(1); }
        };
        
        let content = body["choices"][0]["message"]["content"].as_str().unwrap_or("");
        
        // Run tools if needed
        let mut outputs = Vec::new();
        let text_lower = content.to_lowercase();
        
        if text_lower.contains("bash:") {
            if let Some(pos) = text_lower.find("bash:") {
                let cmd = content[pos+5..].trim().to_string();
                if !cmd.is_empty() {
                    let result = tool_executor.bash.execute(&format!("bash:\"{}\"", cmd));
                    outputs.push(format!("[bash] → {}", result.output));
                }
            }
        }
        
        if text_lower.contains("websearch:") {
            if let Some(pos) = text_lower.find("websearch:") {
                let query = content[pos+11..].trim().to_string();
                if !query.is_empty() {
                    let rt2 = tokio::runtime::Runtime::new().unwrap();
                    let result = rt2.block_on(tool_executor.web_search.execute(&query));
                    outputs.push(format!("[websearch] → {}", result.output));
                }
            }
        }
        
        if outputs.is_empty() {
            content.to_string()
        } else {
            format!("{}\n\nTool Results:\n{}", content, outputs.join("\n"))
        }
    });
    
    println!("\n=== RESPONSE ===\n{}", result);
    
    // Restart service
    println!("\nRestarting service...");
    let binary = std::env::current_exe().unwrap_or_default();
    if binary.to_string_lossy().contains("amor") {
        std::process::Command::new(&binary).arg("serve").spawn().ok();
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
                            if let Some(text) = msg.get("text").and_then(|t| t.as_str()) {
                                let chat_id = msg.get("chat").and_then(|c| c.get("id")).and_then(|i| i.as_i64()).unwrap_or(0);
                                
                                println!("{}📱 TG:{} {}", YELLOW, RESET, text);
                                
                                let response = clean_output(&process_telegram_message(&state, text).await);
                                
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
                                speak(&response);
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

async fn process_telegram_message(state: &AppState, input: &str) -> String {
    let mut chat_hist: Vec<ChatMsg> = Vec::new();
    let tool_executor = ToolExecutor::new();
    
    chat_hist.push(ChatMsg { role: "system".to_string(), content: state.master.clone() });
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
        
        let outs = if !choice.tool_calls.is_empty() { 
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
            o 
        } else { 
            tool_executor.execute(&content).await.1 
        };
        
        eprintln!("DEBUG: tool outputs: {:?}", outs);
        
        if outs.is_empty() { 
            chat_hist.push(ChatMsg { role: "assistant".to_string(), content: content.clone() }); 
            return content; 
        }
        
        let tool_result = outs.join("\n");
        eprintln!("DEBUG: feeding back tool result: {}", &tool_result[..tool_result.len().min(200)]);
        
        chat_hist.push(ChatMsg { role: "user".to_string(), content: format!("Result: {}", tool_result) });
    }
    
    "No response".to_string()
}

async fn run_console_chat(config: &mut config::Config, model: String, master: String) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut chat_hist: Vec<ChatMsg> = Vec::new();
    let tool_executor = ToolExecutor::new();

    chat_hist.push(ChatMsg { role: "system".to_string(), content: master });

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
        let outs = if !choice.tool_calls.is_empty() { 
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
            o 
        } else { tool_executor.execute(&content).await.1 };
        if outs.is_empty() { chat_hist.push(ChatMsg { role: "assistant".to_string(), content: content.clone() }); return content; }
        chat_hist.push(ChatMsg { role: "user".to_string(), content: format!("Result: {}", outs.join("\n")) });
    }
    "No response".to_string()
}
