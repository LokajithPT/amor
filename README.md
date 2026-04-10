# AMOR

**Autonomous Multi-Operator Response system** running on Raspberry Pi.

AMOR is a personal AI assistant with:
- Telegram chat mode
- Local console mode
- Tool calling (web search, reminders, file ops, memory, scripts, bash)
- Optional speech output
- Multi-key Groq fallback support

## Why This Project Exists

AMOR is built to act like a practical daily operator, not a toy chatbot:
- remembers important things
- runs commands and scripts when needed
- manages reminders and alerts
- can be extended for social automation workflows

## Core Stack

- Rust (`tokio`, `reqwest`, `serde`, `teloxide`)
- Groq Chat Completions API
- DuckDuckGo HTML endpoint for search
- Telegram Bot API
- Optional Python speech pipeline (`kokoro-onnx`)

## Main Commands

```bash
# Console mode
/home/fuckall/amor

# Telegram bot service (24/7 mode)
/home/fuckall/amor serve

# Kill service
/home/fuckall/amor kill
```

## Repo Layout

```text
.
├── lemmelearn/                # Main Rust project
│   ├── src/
│   │   ├── main.rs            # Core runtime + Telegram loop
│   │   ├── config.rs          # Config + prompt/material loading
│   │   ├── speak.rs           # CLI speech helper
│   │   └── tool/              # Tool executor + tool modules
│   └── amorshi/               # Runtime prompt/config/memory files
├── forme.md                   # Project operating notes
├── MVP.md                     # Product direction notes
└── README.md                  # This file
```

## Tooling Built In

- `websearch` - web lookups
- `reminder` - add/list/delete reminders
- `memsave` / `memrecall` - lightweight persistent memory
- `file_read` / `file_write` / `file_edit` / `ls` - file operations
- `execute_command` / `bash` - shell execution
- `script_run` - run saved scripts

## Notes

- Config and secrets are currently file-based (`amorshi/shit.cfg`).
- AMOR has both an active code tree and legacy mirrored paths in this repo.
- This setup is optimized for a personal/private environment first.

## Next Up

- Stronger credential handling
- Cleaner search result parsing
- Better conversation memory lifecycle
- Instagram account workflow integration

