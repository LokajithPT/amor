# AMOR - Project Documentation

## What is AMOR?

AI assistant running on Raspberry Pi 4B (Arch Linux). Telegram bot + console mode. Uses Groq API with moonshotai/kimi-k2-instruct-0905 model. Has tool calling capabilities.

---

## File Locations

### On RasPi (host: fuckall@fuckall)
- Binary: `/home/fuckall/amor` (runs 24/7)
- Source: `/home/fuckall/code/amor/lemmelearn/`
- TTS venv: `/home/fuckall/tars_voice/venv/` (kokoro-onnx)
- TTS models: `/home/fuckall/kokoro-v1.0.int8.onnx`, `/home/fuckall/voices-v1.0.bin`
- TTS script: `/home/fuckall/code/amor/lemmelearn/amorshi/scripts/tars_speak.py`

### On Local (host: skedaddle@localhost)
- Source: `/home/skedaddle/code/amor/lemmelearn/`
- Project root: `/home/skedaddle/code/amor/`

---

## Commands

```bash
/home/fuckall/amor         # Console mode
/home/fuckall/amor serve   # Telegram bot (24/7)
```

---

## Features Working

1. **Telegram Bot** - Receives messages via getUpdates API, responds via sendMessage
2. **Console Mode** - Interactive chat
3. **Auto-restart** - When console exits, restarts Telegram service
4. **API Key Rotation** - Has 2 Groq keys, auto-switches on quota
5. **Memory** - memsave/memrecall tools
6. **Web Search** - DuckDuckGo
7. **File Operations** - READ, WRITE, LS
8. **Reminders** - NOW WORKING (tool_calls parsing + fallback)
9. **TTS** - kokoro-onnx with am_michael voice, speed 1.0, only speaks for reminders/alerts

---

## How Reminders Work (IMPORTANT)

1. **Model should use tool_calls:**
   ```
   {"tool_calls": [{"function": {"name": "reminder", "arguments": "{\"message\": \"check oven\", \"minutes\": 0}"}}]}
   ```
2. **mod.rs parses this**: Extracts `message` and `minutes` from JSON arguments
3. **Fallback**: If model doesn't use tool_calls, mod.rs has fallback that catches "reminder", "timer", "seconds", "minutes" keywords and extracts time manually
4. **Reminder system**: Checks every 30 seconds, sends Telegram message + TTS when due

---

## TTS (Text-to-Speak)

- Script: `amorshi/scripts/tars_speak.py`
- Uses kokoro-onnx Python library
- Voice: am_michael (deep, TARS-like)
- Speed: 1.0
- Only triggers on: "remind", "alert", "notice", "deadline" keywords OR explicit request
- Called from `speak()` function in main.rs

---

## Source Files

```
lemmelearn/
├── Cargo.toml
├── src/
│   ├── main.rs         # Entry point, Telegram loop, speak(), config loading
│   ├── config.rs       # Loads amorshi files (shit.cfg, master.md, etc.)
│   ├── telegram.rs     # Telegram API calls (getUpdates, sendMessage) - via curl
│   ├── speak.rs        # Binary for CLI TTS (calls tars_speak.py)
│   └── tool/
│       ├── mod.rs      # ToolExecutor - parses tool_calls, fallback logic
│       ├── search.rs   # Web search
│       ├── files.rs    # READ, WRITE, LS
│       ├── reminders.rs # Reminder system
│       ├── memory.rs   # memsave/memrecall
│       ├── scripts.rs  # Script creation/execution
│       └── bash.rs     # execute_command
├── amorshi/
│   ├── master.md       # System prompt - TARS personality, reminder rules
│   ├── shit.cfg        # API keys, model config
│   ├── memory.md       # Stored memories
│   ├── reminders.md    # Active reminders
│   ├── tools.md        # Tool documentation
│   └── scripts/        # User scripts folder
│       └── tars_speak.py  # TTS script
└── target/release/
    └── lemmelearn      # Binary (copied to /home/fuckall/amor)
```

---

## Config (shit.cfg)

Contains:
- API keys (groq keys - multiple for fallback)
- Model: moonshotai/kimi-k2-instruct-0905
- Active account selection

---

## Master.md (System Prompt)

Key rules:
1. Must use reminder tool for ANY timer/reminder request
2. Format: {"tool_calls": [{"function": {"name": "reminder", "arguments": "{\"message\": \"X\", \"minutes\": Y}"}}]}
3. No code, no bash for timers
4. Personality: TARS-like, confident, direct
5. TTS triggers on reminder/alert keywords

---

## Known Issues

1. Model sometimes tries to write Python scripts instead of using reminder tool - fallback catches this
2. Web search returns raw-ish HTML (needs better parsing)
3. No conversation history limit (grows forever)
4. File read/write syntax is finicky

---

## To Do / Ideas

1. **Looper tool** - Background scripts that can interrupt AMOR when something happens
2. **Better TTS** - Already using kokoro-onnx (great quality)
3. **Conversation limit** - Truncate history to save tokens
4. **Improve websearch parsing** - Cleaner output

---

## Build Process

1. Edit code in `/home/skedaddle/code/amor/lemmelearn/`
2. Copy to RasPi: `scp src/tool/mod.rs fuckall@fuckall:/home/fuckall/code/amor/lemmelearn/src/tool/`
3. Build on RasPi: `cd /home/fuckall/code/amor/lemmelearn && cargo build --release`
4. Copy binary: `cp /home/fuckall/code/amor/lemmelearn/target/release/lemmelearn /home/fuckall/amor`
5. Restart: `/home/fuckall/amor serve &`

---

## Quick Reference

- SSH to RasPi: `ssh fuckall@fuckall`
- Check if running: `pgrep -f amor`
- Kill: `pkill -f amor`
- Logs: Check terminal output or `/tmp/amor.pid`
- Telegram chat_id: Get from user when they message bot