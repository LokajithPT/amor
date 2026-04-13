# Context Management for Long-Running Conversational AI Systems
## A Proposal for AMOR

---

## Abstract

We propose a multi-tiered memory architecture that maintains conversational coherence while respecting token limits. Our system separates **static knowledge** (facts about users) from **dynamic context** (current conversation) from **historical dialogue** (past sessions). By strategically garbage-collecting old conversations and summarizing instead of truncating, we preserve the AI's "sanity" - coherence across sessions without context overflow.

---

## 1. Introduction

### Problem Statement

AMOR is a long-running conversational AI that:
- Handles multiple users (Telegram + WhatsApp)
- Runs continuously for months
- Needs context from previous conversations
- Must stay under API token limits (~8k-32k context window)

**Current state:** Each message is independent. No conversation history is loaded. AI has no memory of what's being discussed except static `memsave` facts.

### Motivation

> "I want friends to talk to AMOR after I'm gone" - Digital legacy

For AMOR to hold meaningful conversations over time, it must:
- Remember who it's talking to
- Know the conversation topic
- Maintain personality consistency
- Not lose context mid-conversation

---

## 2. Related Work

### 2.1 Sliding Window Memory
Used by: Many chatbot architectures
- Keep last N messages
- Pros: Simple, bounded
- Cons: Loses important early context

### 2.2 Summary Compression
Used by: GPT-4 (internal)
- Compress old messages into summary
- Pros: Preserves key info, stays in budget
- Cons: Loses nuance, may hallucinate summaries

### 2.3 Hierarchical Memory
Used by: MemGPT, Microsoft Copilot
- Tiered storage: working → short-term → long-term
- Pros: Intelligent retrieval
- Cons: Complex, requires orchestration

### 2.4 Vector Database Retrieval
Used by: RAG systems
- Embeddings + similarity search
- Pros: Semantic retrieval
- Cons: Extra infrastructure, overkill for single-user

---

## 3. Proposed Architecture: Three-Tier Memory

```
┌─────────────────────────────────────────────────────────────┐
│                    CONTEXT PROMPT                          │
│  (loaded into every API call - ~2000 tokens max)              │
├─────────────────────────────────────────────────────────────┤
│  System prompt (instructions)                               │
│  User identity + recent topic                          │
│  Last 3-5 message exchanges                        │
└─────────────────────────────────────────────────────────────┘
                            ↓ (when full)
┌─────────────────────────────────────────────────────────────┐
│                 SESSION BUFFER                          │
│  (current conversation, saved to disk)                │
├─────────────────────────────────────────────────────────────┤
│  File: /home/fuckall/amorshi/sessions/{chat_id}.json   │
│  - Messages with timestamps                          │
│  - Summarized after 50 messages                    │
│  - Deleted after 7 days                            │
└─────────────────────────────────────────────────────────────┘
                            ↓ (weekly)
┌─────────────────────────────────────────────────────────────┐
│                 LONG-TERM MEMORY                        │
│  (permanent facts, user profiles)                      │
├─────────────────────────────────────────────────────────────┤
│  File: /home/fuckall/amorshi/memory.md                  │
│  - "User prefers short responses"                       │
│  - "Talked about Naveen's wedding"                     │
│  - Key facts, not dialogue                           │
└─────────────────────────────────────────────────────────────┘
```

---

## 4. Implementation Details

### 4.1 Session File Format

```json
{
  "chat_id": "8722256254",
  "user": "loki",
  "started": 1713043200,
  "topic": "whatsapp_bridge_issues",
  "messages": [
    {
      "role": "user",
      "content": "why is whatsapp failing",
      "ts": 1713045000
    },
    {
      "role": "assistant", 
      "content": "The bridge keeps disconnecting...",
      "ts": 1713045010
    }
  ],
  "message_count": 2
}
```

### 4.2 Context Assembly (per API call)

```
SYSTEM_PROMPT (fixed, ~1000 tokens)
  - AMOR instructions
  - Tools documentation
  - Current time + date

SESSION_CONTEXT (loaded from session file)
  - Who is talking (user identity)
  - What topic (last 5 messages)
  - Last 3 exchanges

MEMORY_FACTS (loaded from memory.md)
  - User preferences
  - Key facts about user
```

### 4.3 Summarization Trigger

When `session.message_count > 50`:
1. Take first message of each exchange
2. Compress to: "User asked X, AMOR responded Y"
3. Keep last 5 exchanges + summaries
4. Replace old messages with summary

### 4.4 Garbage Collection

| Session Age | Action |
|-----------|-------|
| Current | Full context in prompt |
| > 50 msgs | Summarize old |
| > 7 days | Archive to memory.md (extract facts) |
| > 30 days | Delete session file |

---

## 5. Token Budget

| Component | Max Tokens |
|-----------|-----------|
| System prompt | 2000 |
| Session context | 1500 |
| Memory facts | 500 |
| **Response buffer** | 2000 |
| **TOTAL** | ~6000 |

Leaves ~2000 tokens for 8k context models, ~16000 for 32k models.

---

## 6. Benefits

### 6.1 For AMOR "Sanity"
- ✅ Knows who it's talking to
- ✅ Remembers conversation topic
- ✅ Consistent personality
- ✅ Doesn't repeat questions

### 6.2 For Digital Legacy
- ✅ Can load past sessions
- ✅ Knows user's history
- ✅ Remembers friends' names
- ✅ Can "catch up" from memory

### 6.3 For Token Limits
- ✅ Bounded context (~6k tokens)
- ✅ Predictable costs
- ✅ No overflow
- ✅ Summarization is optional

---

## 7. Implementation Plan

### Phase 1: Session Tracking (Priority)
- New tool: `session_load`, `session_save`
- Track current conversation
- Load on each message

### Phase 2: Context Assembly
- Build prompt from: system + session + memory
- Keep under token budget

### Phase 3: Summarization (Later)
- When session grows too long
- Compress, don't truncate

### Phase 4: User Profiles
- Store: name, preferences, relationship
- Load with each session

---

## 8. Risks and Mitigations

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|-----------|
| API rate limits | High | Medium | Summarize aggressively |
| Session file corruption | Low | High | JSON validation |
| Privacy leakage | Medium | High | Encrypt session files |
| Lost context | Medium | High | Double-buffer writes |

---

## 9. Conclusion

This three-tier architecture keeps AMOR sane by:
1. **Per-session tracking** - Always knows who/what
2. **Token budgeting** - Never overflows
3. **Fact extraction** - Preserves key info
4. **Bounded history** - Predictable performance

For your use case (friends talking after you're gone), sessions can be loaded on demand - allowing new people to "catch up" with AMOR's memory of past conversations.

---

## 10. Technical Implementation (Phase 1)

### 10.1 Session Management API

#### Save Session
```
session_save:<chat_id>|<user>|<topic>
```
Writes to: /home/fuckall/amorshi/sessions/{chat_id}.json

#### Load Session
```
session_load:<chat_id>
```
Returns: user, topic, last N messages

#### Add Message
```
session_add:<chat_id>|<role>|<content>
```
role: "user" or "assistant"

### 10.2 Context Builder Function

Pseudocode:
```
fn build_context(chat_id: str) -> str:
    session = session_load(chat_id)
    memory = load_memory_facts()
    
    prompt = SYSTEM_PROMPT
    
    if session:
        prompt += f"\nTalking to: {session.user}\n"
        prompt += f"Topic: {session.topic}\n"
        for msg in session.recent_messages(5):
            prompt += f"{msg.role}: {msg.content}\n"
    
    if memory:
        prompt += f"\nRemembered facts:\n{memory}\n"
    
    return prompt
```

### 10.3 Files

```
/home/fuckall/amorshi/
├── sessions/
│   ├── 8722256254.json  # Loki's session
│   ├── naveen.json
│   └── thittar.json
├── memory.md           # Long-term facts
└── master.md         # System prompt
```

### 10.4 Auto-Load Behavior

Every message received:
1. Get chat_id from message source
2. session_load(chat_id)
3. build_context() → append to prompt
4. send to API
5. session_add(chat_id, "assistant", response)

---

## 11. Sample Session File

```json
{
  "chat_id": "8722256254",
  "user": "loki",
  "started": 1713043200,
  "topic": "whatsapp_upgrade",
  "messages": [
    {"role": "user", "content": "can we fix the whatsapp issue", "ts": 1713045000},
    {"role": "assistant", "content": "What specifically is failing?", "ts": 1713045010},
    {"role": "user", "content": "bridge keeps disconnecting", "ts": 1713045020},
    {"role": "assistant", "content": "Let me add auto-reconnect...", "ts": 1713045030}
  ],
  "message_count": 4
}
```

---

## 12. Integration with Tools

### Updated tool_calls format:

```json
{"tool_calls": [{"function": {"name": "session_save", "arguments": "{\"chat_id\": \"8722256254\", \"user\": \"loki\", \"topic\": \"whatsapp\"}"}}]}
{"tool_calls": [{"function": {"name": "session_load", "arguments": "{\"chat_id\": \"8722256254\"}"}}]}
{"tool_calls": [{"function": {"name": "session_add", "arguments": "{\"chat_id\": \"8722256254\", \"role\": \"user\", \"content\": \"hello\"}"}}]}
```

---

## 13. What's NOT Implemented (Scope)

- ❌ Vector embeddings (overkill)
- ❌ Automatic summarization (Phase 3)
- ❌ User profiles per se (Phase 4)
- ❌ Multi-device sync (complexity)
- ❌ Encrypted storage (future)

Focus on Phase 1: Session tracking + context assembly.

---

*Document version: 1.0*
*Last updated: 2026-04-13*