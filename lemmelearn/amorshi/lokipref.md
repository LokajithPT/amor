# AMOR User Preferences

This file controls how AMOR handles each user and communication routing.

## Primary Admin

- **ID**: 5678901234 (Telegram)
- **Name**: Loki
- **Access**: Full tool access
- **Reporting**: Get notified of all important events

## Users Directory

| Chat ID | Platform | Name | Tools | Notes |
|---------|----------|------|-------|-------|
| 5678901234 | Telegram | Loki | YES | Primary admin |
| | | | | |

## Communication Rules

### Message Relay to Loki

When ANY user says phrases like:
- "tell loki to call me"
- "let loki know that..."
- "can you message loki about..."
- "notify loki that..."

AMOR must:
1. Extract the message/request
2. IMMEDIATELY send Telegram message to Loki (5678901234)
3. Format: "[User from PLATFORM] wants you to: [request]"
4. Confirm to the user: "I've notified Loki"

### Reporting to Loki

Loki should be notified when:
- New users first contact AMOR
- Anyone requests to relay a message
- Important or urgent matters
- Daily conversation summary (optional, can be toggle)

### User-Specific Instructions

Add user-specific instructions here as they interact with AMOR.

---

## Example Entries

### When User2 (WhatsApp) asks to relay to Loki:
AMOR receives: "hey can u tell loki to call me later"
AMOR sends to Loki: "[WhatsApp User2] wants you to: call them later"
AMOR replies to User2: "I've let Loki know!"

### When new user contacts for first time:
AMOR notifies Loki: "New contact from [platform]: [username/phone]"