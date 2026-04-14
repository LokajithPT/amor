# AMOR - Ask for Advice

Give this to an AI (like ChatGPT, Claude, etc), then ask your question.

---

## Project Summary

**AMOR** = AI assistant on Raspberry Pi 4B
- Built in Rust (source at /home/fuckall/code/amor/lemmelearn/)
- Runs at /home/fuckall/amor (binary)
- Uses Groq API (model: moonshotai/kimi-k2-instruct)
- Handles Telegram + WhatsApp messages

## Current Status

**Working:**
- Telegram bot responds to messages
- Tools: bash (run commands), memsave (remember), memrecall (remember), websearch
- Memory.md loads ONCE at startup (recent fix)
- No more rate limit issues from loading files every message

**Broken:**
- WhatsApp bridge disconnects randomly
- No session tracking (AMOR forgets previous conversation)
- Model sometimes outputs `<use_mcp_tool>` XML instead of `bash:`

## Key Locations

| What | Where |
|------|-------|
| Source code | /home/fuckall/code/amor/lemmelearn/ |
| Binary | /home/fuckall/amor |
| Config | /home/fuckall/amorshi/ |
| Master prompt | /home/fuckall/amorshi/master.md |
| Memory | /home/fuckall/amorshi/memory.md |
| Tools | /home/fuckall/amorshi/tools.md |
| User memory | /home/fuckall/amorshi/users/{chat_id}.md |

## Build & Run

```bash
# On Pi (not laptop - different architecture)
cd /home/fuckall/code/amor/lemmelearn
cargo build --release
cp target/release/lemmelearn /home/fuckall/amor
/home/fuckall/amor
```

## Files to Read for Context

Start here:
- allofit.md = everything
- behemoth.md = context management research

---

## Ask Your Question Below:

[Your question goes here]