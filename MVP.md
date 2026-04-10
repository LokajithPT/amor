# AMOR - Agentic Model on Raspi

## Overview
A lightweight AI assistant written in Rust. Runs locally on laptop, connects to LLM models on remote Raspi (10.0.0.2) via HTTP API.

## What is AMOR?
**AMOR** = Agentic Model on Raspi

A self-improving AI agent that learns from conversations, manages its own tools, and runs locally on your infrastructure.

## Architecture
```
[Laptop: AMOR] --HTTP--> [Raspi: llama-server/groq-api] 
       |                          |
       |                          +-- Port 11435 (gemma-3-270m)
       |                          +-- Port 11438 (qwen2.5-0.5b)
       |                          +-- Groq API (cloud)
       |
       +-- Memory (local text files)
       +-- Tools (self-written)
       +-- Reminders
       +-- Web UI (future)
       +-- Telegram Bot (future)
       +-- WhatsApp/Insta (future)
```

## Connection Details
- **Raspi IP**: 10.0.0.2
- **Local Models**: 
  - gemma-3-270m: http://10.0.0.2:11435/v1/chat/completions
  - qwen2.5-0.5b: http://10.0.0.2:11438/v1/chat/completions
- **Groq API**: https://api.groq.com/openai/v1/chat/completions

## Features (MVP)

### 1. Chat Interface
- CLI chat mode
- HTTP POST to LLM endpoint
- Parse JSON response

### 2. Self Memory
- Stores conversation in memory.txt
- Reads on startup to continue context
- Can delete memory anytime (delete memory.txt)

### 3. Self Tools
- Tools stored as .md files in tools/ folder
- On startup, reads all tools/*.md to know capabilities
- Can add new tools - AMOR reads them on next boot

### 4. Reminders
- Simple: "remind me in X minutes"
- Store in reminders.txt (plain text)
- Background task checks and notifies

### 5. Web Search
- DuckDuckGo API (free, no key)
- Return text results to user

### 6. File Operations
- Read file: say "READ: /path/to/file"
- Write file: say "WRITE: /path | content"
- List dir: say "LS: /path"

### 7. Web UI (Future)
- Axum web server
- Simple HTML chat interface

### 8. Telegram Control (Future)
- Bot API: https://core.telegram.org/bots/api
- Set webhook or long polling

### 9. WhatsApp/Instagram (Future)
- WhatsApp Business API
- Instagram Graph API

## NOT Included (MVP)
- Code generation
- Complex agents
- MCP support

## Tech Stack
- **Language**: Rust
- **HTTP Client**: reqwest
- **Web Server**: axum
- **Storage**: Plain text files (memory.txt, tools/, reminders.txt)
- **Async**: tokio
- **Serialization**: serde, serde_json

## Files Structure
```
amor/
├── src/
│   ├── main.rs
│   ├── chat.rs      # LLM calls
│   ├── search.rs    # Web search
│   ├── files.rs     # File ops
│   ├── reminders.rs # Reminder system
│   └── memory.rs    # Self memory system
├── memory/
│   └── memory.txt   # conversation history
├── tools/
│   ├── web_search.md
│   ├── file_ops.md
│   └── reminders.md
├── reminders/
│   └── reminders.txt
├── config.json      # api keys, settings
├── master.txt       # system prompt
├── Cargo.toml
└── README.md
```

## Memory System
On startup:
1. Read master.txt - main system prompt
2. Read all tools/*.md - knows available tools
3. Read memory.txt - past conversation
4. Read reminders.txt - pending reminders

All stored as plain text - easy to edit, delete, understand

## How AMOR Works

### Tool Parsing (Prompt-based)
AMOR uses simple text commands in responses:
- "SEARCH: what to search"
- "READ: /path/to/file"
- "WRITE: /path | content"
- "REMIND: message | minutes"

Rust code parses these and executes tools.

### Adding New Tools
User tells AMOR: "add a tool to read sensor from raspi"
AMOR writes new tools/sensor.md
Next boot - AMOR reads it and knows about sensor!

### Multiple API Accounts
```json
// config.json
{
    "accounts": {
        "groq1": "gsk_...",
        "groq2": "gsk_..."
    },
    "active_account": "groq1"
}
```

## Groq Free Tier
- 60 requests/minute
- 1,000 requests/day  
- 500K tokens/day

Plenty for personal use!

## Learning Roadmap

### Phase 1: Rust Basics
- [x] Ownership & Borrowing
- [x] Structs & Enums
- [x] Result & Option
- [x] Crates (dependencies)

### Phase 2: Async & Networking
- [x] tokio basics
- [x] reqwest for HTTP
- [ ] JSON parsing

### Phase 3: Core Features
- [ ] Chat with LLM (reqwest → Raspi/Groq)
- [ ] Memory system (read/write text files)
- [ ] Tool parsing (simple string matching)
- [ ] Web search (DuckDuckGo)
- [ ] File read/write

### Phase 4: Messaging
- [ ] Telegram bot
- [ ] WhatsApp API

### Phase 5: Web UI
- [ ] Axum setup
- [ ] Simple HTML frontend

## Key Code Patterns

**HTTP to LLM (reqwest)**:
```rust
let client = reqwest::Client::new();
let response = client.post("https://api.groq.com/openai/v1/chat/completions")
    .header("Authorization", format!("Bearer {}", api_key))
    .json(&payload)
    .send()
    .await?;
```

**Simple Chat Loop**:
```rust
loop {
    let input = read_line();
    if input == "exit" { break; }
    let response = call_llm(input).await;
    // parse tools from response
    // execute tools
    // save to memory
    println!("{}", response);
}
```

**Memory (plain text)**:
```rust
// read
let memory = fs::read_to_string("memory.txt").unwrap_or_default();

// append
let mut file = OpenOptions::new().append(true).open("memory.txt")?;
writeln!(file, "User: {}", input)?;
writeln!(file, "Bot: {}", response)?;

// delete
fs::remove_file("memory.txt")?;
```

## Environment Variables
```
GROQ_API_KEY=gsk_...
RASPI_URL=http://10.0.0.2
TELEGRAM_BOT_TOKEN=...
WHATSAPP_TOKEN=...
```

## Notes
- Start simple, add features one by one
- Test each component before moving on
- Use `cargo run` to test
- `cargo build --release` for final binary
- Memory is local - u own it, can delete anytime
- Tools are self-written - AMOR can learn new capabilities