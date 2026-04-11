# AMOR - Project Documentation

## What is AMOR?

AI assistant running on Raspberry Pi 4B (Arch Linux). Telegram bot + console mode + WhatsApp integration. Uses Groq API. Has tool calling capabilities.

---

## File Locations

### On RasPi (host: fuckall@fuckall)
- Binary: `/home/fuckall/amor` (runs 24/7)
- Source: `/home/fuckall/code/amor/lemmelearn/`
- TTS venv: `/home/fuckall/tars_voice/venv/` (kokoro-onnx)
- TTS models: `/home/fuckall/kokoro-v1.0.int8.onnx`, `/home/fuckall/voices-v1.0.bin`
- TTS script: `/home/fuckall/code/amor/lemmelearn/amorshi/scripts/tars_speak.py`
- WhatsApp: `/home/fuckall/whatsapp-bridge/`, `/home/fuckall/whatsapp-mcp-server/`

### On Local (host: skedaddle@localhost)
- Source: `/home/skedaddle/code/amor/lemmelearn/`
- Project root: `/home/skedaddle/code/amor/`

---

## Commands

```bash
# WhatsApp (start first)
/home/fuckall/whatsapp-mcp-initd.sh start

# AMOR Telegram bot
/home/fuckall/amor serve

# Console mode
/home/fuckall/amor
```

---

## WhatsApp Integration (Key Info)

### How It Works
1. **WhatsApp Bridge** (Go + whatsmeow) - connects to WhatsApp Web, stores messages in SQLite
2. **MCP Server** - provides API endpoints for sending/receiving
3. **Message Poller** - checks for new messages every 5s, writes to queue
4. **AMOR Main Loop** - reads queue, processes via AI, sends responses

### WhatsApp Start Script
```bash
/home/fuckall/whatsapp-mcp-initd.sh start
# or
/home/fuckall/start_mcp_v3.sh
```

### WhatsApp Files
- Wrapper: `/home/fuckall/whatsapp_wrapper.py` - send/receive/search messages
- Poller: `/home/fuckall/simple_poller.py` - checks for new messages
- Queue in: `/tmp/whatsapp_in.txt` (format: WHATSAPP|JID|NAME|MESSAGE)
- Queue out: `/tmp/whatsapp_out.txt`

### Send WhatsApp Message
```bash
# Via wrapper
python3 /home/fuckall/whatsapp_wrapper.py send <phone_number> "message"

# Via AMOR (on Telegram)
whatsapp: naveen|hey whats up
# or natural language
tell my girl i miss her
```

---

## User Map System

File: `/home/fuckall/code/amor/lemmelearn/amorshi/users/user_map.json`

```json
{
  "users": {
    "loki": {
      "telegram": "5678901234",
      "has_tools": true,
      "notes": "Primary admin"
    },
    "naveen": {
      "whatsapp": "917010381233@s.whatsapp.net",
      "name": "Naveen",
      "platforms": ["whatsapp"],
      "custom_rules": "Keep it casual, bro style"
    },
    "thittar": {
      "whatsapp": "158802238242897@lid",
      "name": "Thittar (GF)",
      "platforms": ["whatsapp"],
      "custom_rules": "Keep loving, playful, cute, use emojis"
    }
  }
}
```

### User JIDs
- **Naveen**: 917010381233@s.whatsapp.net
- **Thittar (GF)**: 158802238242897@lid (IMPORTANT - uses @lid not @s.whatsapp.net)

---

## WhatsApp Wingman Mode

When Loki says natural things like:
- "tell my girl i miss her"
- "text my girlfriend that im thinking of her"
- "hey amor message thittar saying call me"
- "send a message to naveen saying sup"

AMOR extracts who + what, looks up in user_map.json, and sends with appropriate tone.

Command format understood internally: `whatsapp: name|message`

---

## Features Working

1. **Telegram Bot** - Receives messages via getUpdates API, responds via sendMessage
2. **Console Mode** - Interactive chat
3. **Auto-restart** - When console exits, restarts Telegram service
4. **API Key Rotation** - Has 2 Groq keys, auto-switches on quota
5. **Memory** - memsave/memrecall tools
6. **Web Search** - DuckDuckGo
7. **File Operations** - READ, WRITE, LS
8. **Reminders** - WORKS (tool_calls parsing + fallback)
9. **TTS** - kokoro-onnx with am_michael voice
10. **WhatsApp** - Send/receive messages via MCP
11. **Wingman Mode** - Natural language triggers for messaging GF

---

## How Reminders Work (IMPORTANT)

1. **Model should use tool_calls:**
   ```
   {"tool_calls": [{"function": {"name": "reminder", "arguments": "{\"message\": \"check oven\", \"minutes\": 0}"}}]}
   ```
2. **mod.rs parses this**: Extracts `message` and `minutes` from JSON arguments
3. **Fallback**: If model doesn't use tool_calls, mod.rs has fallback that catches keywords
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
│   ├── main.rs         # Entry point, Telegram loop, WhatsApp handling, speak()
│   ├── config.rs       # Loads amorshi files
│   ├── telegram.rs     # Telegram API calls - via curl
│   ├── speak.rs        # Binary for CLI TTS
│   └── tool/
│       ├── mod.rs      # ToolExecutor, WhatsApp command handling
│       ├── search.rs   # Web search
│       ├── files.rs    # File ops
│       ├── reminders.rs # Reminders
│       ├── memory.rs   # Memory tools
│       ├── scripts.rs  # Script execution
│       └── bash.rs     # Bash execution
├── amorshi/
│   ├── master.md       # System prompt - personality, wingman rules
│   ├── shit.cfg        # API keys, model
│   ├── memory.md       # Stored memories
│   ├── lokipref.md    # User preferences
│   ├── users/
│   │   └── user_map.json  # User identities
│   └── scripts/
│       └── tars_speak.py  # TTS
└── target/release/
    └── lemmelearn      # Binary
```

---

## Git & Deploy

```bash
# Pull and build on Pi
ssh fuckall@fuckall
cd /home/fuckall/code/amor
git pull origin master
cd lemmelearn && cargo build --release
cp target/release/lemmelearn /home/fuckall/amor

# Restart
pkill -f amor
/home/fuckall/amor serve &
```

---

## Test Commands

```bash
# Test WhatsApp send
python3 /home/fuckall/whatsapp_wrapper.py send 917010381233 "test"

# Check WhatsApp queue
cat /tmp/whatsapp_in.txt

# Check logs
tail /home/fuckall/whatsapp_bridge.log
tail /home/fuckall/amor.log

# Check AMOR process
pgrep -f amor
```

---

## Known Issues

1. WhatsApp session expires ~20 days - may need re-login from different device
2. Instagram still blocked - Meta API issues
3. Some users may have @lid JIDs vs @s.whatsapp.net - need correct one
4. thittar uses @lid format (158802238242897@lid)

---

## Quick Reference

- SSH to RasPi: `ssh fuckall@fuckall`
- Check if running: `pgrep -f amor`
- Kill: `pkill -f amor`
- WhatsApp startup: `/home/fuckall/whatsapp-mcp-initd.sh start`

---

## History

- 2026-04-11: WhatsApp MCP integrated, wingman mode added
- thittar (gf) added with loving/cute custom rules (JID: 158802238242897@lid)
- Naveen added as casual WhatsApp contact (JID: 917010381233@s.whatsapp.net)
- Multi-user system working (Telegram + WhatsApp)