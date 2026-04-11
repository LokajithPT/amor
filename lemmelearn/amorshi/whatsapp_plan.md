# WhatsApp Integration Plan

## Phase 1: Auto-start and Message Polling

### 1.1 WhatsApp Service on Boot
- Create systemd service for WhatsApp MCP
- Ensures it starts automatically on Pi boot

### 1.2 Message Polling Thread
- Separate thread checks for NEW personal messages every 5 seconds
- Ignores group messages
- When new message found:
  - Extract sender JID + message text
  - Write to queue file `/tmp/whatsapp_in.txt`
  - Format: `WHATSAPP|{jid}|{message}`

---

## Phase 2: User Identity System

### 2.1 User Map Structure (`amorshi/users/user_map.json`)
```json
{
  "users": {
    "917010381233@s.whatsapp.net": {
      "name": "Naveen",
      "platforms": ["whatsapp"],
      "memory_file": "naveen.md",
      "security_q": null,
      "security_a": null,
      "custom_response": null,
      "identified": false
    }
  }
}
```

### 2.2 Security Verification
- When user messages from new platform or suspicious activity
- Ask security question → verify answer
- If fail → ask 2-3 casual questions to build confidence

### 2.3 Casual Profiling Questions (non-strict)
- "Do you have any pets?"
- "What's your favorite food?"
- "What do you do for work/study?"
- Build personality over time

---

## Phase 3: Custom Response Rules

### 3.1 Per-User Instructions (`lokipref.md`)
```markdown
## Custom Response Rules

### Naveen (WhatsApp: 917010381233)
- If Naveen texts: Tell him I'm coding, ask if urgent
- If urgent: Send me Telegram immediately

### Mom (WhatsApp: 91xxxxxxxxxx)
- Always respond warmly, keep it short
- If anything urgent, notify Loki

### GF (WhatsApp: 91xxxxxxxxxx)
- Keep it loving, playful
- Priority: always respond
```

### 3.2 Default Behavior
- Unknown users: Friendly introduction, ask name
- Known users: Load their memory, respond naturally

---

## Phase 4: Cross-Platform Unification

### 4.1 Unified Identity
- User can be identified across Telegram, WhatsApp, Instagram
- Match by: security answer, casual conversation patterns, explicit linking

### 4.2 Platform-Aware Responses
- Know which platform user is messaging from
- Reference past conversations regardless of platform