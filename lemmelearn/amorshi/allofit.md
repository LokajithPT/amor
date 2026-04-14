# AMOR - Everything You Need to Know

**FOR FUTURE AI: If you need to understand or modify AMOR, start here.**

## TL;DR (For AI)
- AMOR is written in **Rust** (not Python)
- Runs on **Raspberry Pi** (not cloud)
- Uses **Groq API** for LLM (not OpenAI)
- Telegram messages are the main interface
- If code doesn't build, run: `cd /home/fuckall/code/amor/lemmelearn && cargo build --release`

## What is AMOR?

AMOR is an AI assistant running on a Raspberry Pi 4B. It responds to Telegram messages and WhatsApp messages. It's designed to be a "wingman" - when Loki asks it to message his girlfriend, it does. Eventually, friends should be able to talk to AMOR after Loki is gone (digital legacy).

## The Goal

1. **Instant help** - Answer questions, run commands, send WhatsApp messages
2. **Wingman** - Message Loki's girlfriend when asked naturally
3. **Digital legacy** - Friends can talk to AMOR after Loki passes away

---

## Tech Stack

### Hardware
- **Raspberry Pi 4B** (4GB RAM) - runs everything
- **Server**: runs 24/7 via systemd or manual

### Software

| Component | What it does | Where it runs |
|-----------|------------|-------------|
| **AMOR (Rust)** | Main AI - handles messages, runs tools | `/home/fuckall/amor` |
| **Telegram Bot** | Receives/sends TG messages | via getUpdates API |
| **WhatsApp Bridge (Go)** | Connects to WhatsApp | `/home/fuckall/whatsapp-bridge/` |
| **WhatsApp MCP (Python)** | MCP server for WA | `/home/fuckall/whatsapp-mcp-server/` |
| **WhatsApp Poller** | Checks for new WA messages | `/home/fuckall/whatsapp_poller.py` |

### APIs Used
- **Groq** - LLM API (moonshotai/kimi-k2-instruct model)
- **Telegram getUpdates** - receives messages
- **WhatsApp MCP** - sends/receives WhatsApp

### Files

```
/home/fuckall/
├── amor                    # Main binary (Rust compiled)
├── amorshi/              # Config files
│   ├── master.md         # System prompt (instructions)
│   ├── memory.md         # Global memory (loaded ONCE at startup)
│   ├── tools.md          # Tool documentation
│   ├── user_map.json    # WhatsApp JID mappings
│   └── users/           # Per-user memory files (NOT loaded every message)
├── whatsapp-bridge/      # Go WhatsApp bridge
├── whatsapp-mcp-server/   # Python MCP server
└── whatsapp_poller.py    # Message poller
```

---

## How AMOR Works (Simplified)

```
1. User sends message → Telegram getUpdates API
2. AMOR receives message (chat_id)
3. AMOR builds prompt:
   - System instructions (master.md) - loaded ONCE at startup
   - Global memory (memory.md) - loaded ONCE at startup  
   - User's message
4. Send to Groq API → Get response
5. Parse response for tools (bash:, memsave:, whatsapp:, etc)
6. Run tools → Get results
7. Add results to prompt → Send to Groq again
8. Return final response to user
```

## Message Flow

```
Telegram → getUpdates → AMOR → Groq → Parse tools → Run → Groq → Telegram
WhatsApp → Poller → out queue → AMOR → Groq → WhatsApp MCP → WhatsApp
```

---

## Key Files

### `/home/fuckall/amor` (Binary)
- Built from `/home/fuckall/code/amor/lemmelearn/`
- Run with `./amor` or `./amor serve` for Telegram bot

### Config Files (`/home/fuckall/amorshi/`)

| File | Purpose | Loaded |
|------|---------|--------|
| `master.md` | System instructions | Once at startup |
| `memory.md` | Global facts | Once at startup |
| `tools.md` | Tool docs | Reference only |
| `user_map.json` | WhatsApp JIDs | When needed |
| `shit.cfg` | API keys, model config | At startup |

### Source Code (`/home/fuckall/code/amor/lemmelearn/`)

```
src/
├── main.rs        # Entry point, Telegram handling
├── config.rs    # File loading
├── telegram.rs  # Telegram API
└── tool/
    ├── mod.rs      # Tool executor
    ├── memory.rs   # memsave/memrecall
    ├── bash.rs     # Execute commands
    ├── search.rs   # Web search
    ├── reminders.rs# Reminders
    ├── files.rs    # File ops
    ├── scripts.rs # Run scripts
    └── worker.rs  # Background workers (new)
```

---

## Important Code Patterns

### Tool Format (in model response)
```bash
bash: python3 read_cpu_temp.py
websearch: what's the weather
memsave: my name is loki
memrecall: what's my name
whatsapp: naveen|hey bro what's up
```

### JSON Tool Calls (also works)
```json
{"tool_calls": [{"function": {"name": "bash", "arguments": "{\"command\": \"ls\"}"}}]}
```

### Memory
- **memsave** - saves to `/home/fuckall/amorshi/memory.md`
- **memrecall** - searches memory.md

### User Memory
- Stored in `/home/fuckall/amorshi/users/{chat_id}.md`
- **Currently NOT loaded every message** (fixed)
- Only via memrecall when asked

---

## What's Been Fixed

### 1. Rate Limit Issues
**Problem**: Loading too much every message caused rate limits
**Fix**: 
- `master.md` loaded once at startup (not every message)
- `memory.md` loaded once at startup (not every message)
- User memory files (`users/{chat_id}.md`) NOT loaded every message

### 2. Tool Execution
**Problem**: Model ran command but didn't show output until user said "huh"
**Fix**: After tool runs, loop continues and calls API again with results

### 3. MCP Tool Format
**Problem**: Model using `<use_mcp_tool>` XML tags instead of `bash:`
**Fix**: Added parsing for XML-style tool tags

---

## Current State

### Working ✅
- Telegram bot responds
- Tools work (bash, memsave, memrecall, websearch)
- Memory loaded once at startup (fixed)
- Rate limits should be much better now

### Not Working ❌
- WhatsApp integration (bridge disconnects)
- Session tracking per user (Phase 1 not done)
- User memory not loading

---

## What's Left to Do

### Priority 1: WhatsApp Fix
The WhatsApp bridge keeps disconnecting. Need to:
1. Check bridge logs: `journalctl -u whatsapp-bridge`
2. Look at auto-reconnect logic in poller
3. Maybe restart bridge more aggressively

### Priority 2: Session Tracking (Phase 1)
Track conversation per user:
1. Save session to `/home/fuckall/amorshi/sessions/{chat_id}.json`
2. Load on first message
3. Add to context (last 5 messages)
4. Summarize after 50 messages

### Priority 3: User Memory
Decide: Should user memory load?
- Option A: Only via memrecall (current)
- Option B: Load at startup like global memory

### Priority 4: Voice/TTS
Add speech output for responses

---

## Common Tasks

### Restart AMOR
```bash
pkill -f amor
cp /home/fuckall/code/amor/lemmelearn/target/release/lemmelearn /home/fuckall/amor
./amor serve
```

### Check Logs
```bash
systemctl status amor
journalctl -u amor -n 20
```

### Check Running Services
```bash
systemctl status whatsapp-bridge whatsapp-mcp whatsapp-poller amor
```

### Rebuild from Source
```bash
cd /home/fuckall/code/amor/lemmelearn
git pull origin master
cargo build --release
cp target/release/lemmelearn /home/fuckall/amor
```

### Add Memory
```
> remember my name is loki
```
Saved to memory.md

### Recall Memory
```
> what is my name
```
Searches memory.md

---

## Code Examples (How to Add Features)

### Example 1: Add a New Tool

1. Create tool file in `src/tool/mytool.rs`:
```rust
use crate::tool::ToolResult;

pub struct MyTool { }

impl MyTool {
    pub fn new() -> Self { Self { } }
    
    pub fn execute(&self, cmd: &str) -> ToolResult {
        // Parse cmd, do stuff, return result
        ToolResult::ok("Result here")
    }
}
```

2. Add to `src/tool/mod.rs`:
```rust
pub mod mytool;
pub use mytool::MyTool;
```

3. Add to ToolExecutor struct:
```rust
pub struct ToolExecutor {
    pub mytool: MyTool,
    // ... other tools
}

impl ToolExecutor {
    pub fn new() -> Self {
        Self {
            mytool: MyTool::new(),
            // ...
        }
    }
}
```

4. Add execution in mod.rs execute() function:
```rust
} else if func_name == "mytool" {
    let result = self.mytool.execute(&value);
    outputs.push(format!("[{}] → {}", func_name, result.output));
}
```

### Example 2: Load Something Once at Startup

In `src/config.rs`, add field:
```rust
pub struct AmorshiFiles {
    pub config: Config,
    pub master: String,
    pub memory: String,  // NEW - loaded once
    // ...
}
```

In config loading:
```rust
let memory = fs::read_to_string(&base_path.join("memory.md"))
    .unwrap_or_default();
```

Add to AppState in `src/main.rs`:
```rust
struct AppState {
    config: config::Config,
    model: String,
    master: String,
    memory: String,  // NEW
}
```

Use in message handler:
```rust
// Instead of reading file every time:
let full_system = state.master.clone();

// Add memory once:
// (memory already in state, use it)
```

### Example 3: Add Session Tracking (Phase 1)

Create session file:
```json
// /home/fuckall/amorshi/sessions/8722256254.json
{
  "chat_id": "8722256254",
  "user": "loki",
  "started": 1713043200,
  "topic": "whatsapp_issues",
  "messages": [
    {"role": "user", "content": "hi", "ts": 1713045000},
    {"role": "assistant", "content": "hey", "ts": 1713045010}
  ]
}
```

Load at start of conversation:
```rust
fn load_session(chat_id: i64) -> Session {
    let path = format!("{}/sessions/{}.json", base_path, chat_id);
    // read JSON, parse, return
}
```

Add to context:
```rust
let session = load_session(chat_id);
prompt.push_str(&format!("Topic: {}\n", session.topic));
for msg in session.messages.iter().rev().take(5) {
    prompt.push_str(&format!("{}: {}\n", msg.role, msg.content));
}
```

### Example 4: Fix Rate Limits

The fix was simple - don't load files every message:

**BEFORE (bad)** - every message:
```rust
let user_memory = fs::read_to_string(&user_memory_path).unwrap_or_default();
full_system.push_str(&user_memory);  // LOADED EVERY TIME!
```

**AFTER (good)** - once at startup:
```rust
// In config.rs - load once
let memory = fs::read_to_string(&base_path.join("memory.md"))
    .unwrap_or_default();

// In AppState - store once
struct AppState { memory: String }

// In message handler - use stored
let memory_section = format!("\n## MEMORY:\n{}\n", state.memory);
```

---

## Architecture Diagram

```
                    Telegram Bot
                         ↓
                    getUpdates API
                         ↓
                   ┌────┴────┐
                   │  AMOR   │
                   └────┬────┘
                        ↓
                   ┌────┴────┐
                   │ Groq   │ ←── models: moonshotai/kimi-k2-instruct
                   └────┬────┘
                        ↓
              ┌──────────┼──────────┐
              ↓          ↓          ↓
           bash    memsave   whatsapp
              ↓          ↓          ↓
         /tmp/out   memory.md  MCP bridge
                                 ↓
                           WhatsApp
```

---

## API Keys (in shit.cfg)

```
GROQ_API_KEY_1=...
GROQ_API_KEY_2=...
TELEGRAM_BOT_TOKEN=...
```

---

## Testing

### Test Console Mode
```bash
./amor
> hello
> what's my cpu temp
> remember my name is loki
> what's my name
```

### Test Telegram
Send message to bot on Telegram

---

## The Research Document

See `behemoth.md` for detailed research on context management and Phase 1-4 planning.

---

## Credits

Built by Loki with help from opencode. Running on Raspberry Pi 4B.

Last updated: 2026-04-13