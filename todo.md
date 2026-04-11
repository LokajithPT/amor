# TODO - Make AMOR Feel Alive

## Main Goal

Build a `watchers + interrupt system` so AMOR stops being only reactive and starts acting like a real operator living on the Raspberry Pi.

The target behavior:
- AMOR watches important things in the background
- AMOR notices events without being asked
- AMOR interrupts the user when something matters
- every interruption includes context, urgency, and AMOR's personality

## Why This Is The Big Next Step

Right now AMOR is mostly:
- prompt-driven
- request/response only
- funny, but passive

After this, AMOR becomes:
- persistent
- event-driven
- useful even when the user says nothing

That is the jump from "cool assistant" to "digital creature on the Pi."

## Phase 0 - Preconditions

- [ ] Move secrets out of tracked files first
- [ ] Add a stable env-based config loader
- [ ] Make sure AMOR can restart safely without losing important state
- [ ] Add basic logs for tool use, errors, and outgoing alerts

Why first:
- watchers will multiply side effects
- if config and logs are weak, debugging becomes hell fast

## Phase 1 - Define The Event Model

Build one shared event structure in Rust.

Create a core concept like:
- event id
- source
- severity
- title
- body
- timestamp
- dedupe key
- optional action hints

Examples of event sources:
- reminders
- cpu_temp
- telegram
- instagram
- whatsapp
- script watcher
- system health

Output target:
- one event format that all watchers emit
- one interrupt pipeline that consumes those events

Files likely involved:
- `lemmelearn/src/main.rs`
- new module like `lemmelearn/src/event.rs`
- new module like `lemmelearn/src/watchers/mod.rs`

## Phase 2 - Build The Interrupt Pipeline

AMOR needs one clear path for "something happened."

Pipeline shape:
1. watcher emits event
2. event enters queue
3. dedupe/rate-limit check runs
4. AMOR formats the interruption
5. output goes to Telegram / TTS / WhatsApp / console
6. event is marked delivered

Requirements:
- [ ] event queue in memory
- [ ] small persistent log of delivered events
- [ ] dedupe by event key + time window
- [ ] severity levels like `info`, `warn`, `urgent`
- [ ] routing rules per source and severity

Important:
- do not let watchers send messages directly
- all alerts must go through one interrupt path

That keeps behavior sane.

## Phase 3 - Add Watcher Framework

Create a watcher trait or interface.

Each watcher should:
- have a name
- poll on an interval or wait on input
- emit zero or more events
- never talk to the user directly
- never contain output formatting logic

Basic watcher contract:
- `poll() -> Vec<Event>`

Runtime needs:
- [ ] watcher registry
- [ ] configurable polling intervals
- [ ] error isolation so one watcher dying does not kill all watchers
- [ ] per-watcher logging

Suggested first structure:
- `watchers/reminders.rs`
- `watchers/system.rs`
- `watchers/telegram.rs`
- `watchers/scripts.rs`

## Phase 4 - Ship The First 3 Watchers

Do not start with Instagram.
Start with reliable local wins.

### 1. Reminder Watcher

This should be the first clean event emitter.

Roadmap:
- [ ] stop having reminders fire ad hoc from scattered logic
- [ ] make reminders emit `Event`
- [ ] route reminders through interrupt pipeline
- [ ] include urgency and spoken variant

Success condition:
- reminder becomes one clean structured event

### 2. CPU / System Health Watcher

Make the Pi feel alive.

What to watch:
- CPU temp
- disk space
- maybe memory pressure
- maybe process death for AMOR sidecars

Rules:
- only interrupt on threshold crossing
- do not spam repeated "still hot" alerts

Example:
- temp crosses 75 C -> warn
- temp crosses 82 C -> urgent
- temp returns normal -> recovery event

### 3. Script Output Watcher

This is where the project gets powerful.

Let background scripts emit events into AMOR.

Simple first design:
- scripts write JSON lines into a watched file or pipe
- AMOR reads them and converts them to events

Example script payload:
- source
- severity
- title
- body

This lets you bolt in new abilities fast without rewriting core Rust every time.

## Phase 5 - Give Watchers Memory And Context

Interruptions should not feel random.

Add lightweight watcher state:
- last fired timestamp
- last severity
- last value
- cool-down timer

This makes alerts feel intelligent instead of noisy.

Example:
- not "CPU is 76 C" every cycle
- instead "CPU just crossed 75 C and has stayed hot for 3 minutes"

## Phase 6 - Make Output Actually Good

Interrupts need style rules.

Each event should produce:
- short machine summary
- user-facing line
- optional TTS version

Example formatting policy:
- `info` -> Telegram text only
- `warn` -> Telegram + maybe console
- `urgent` -> Telegram + TTS + repeat policy

AMOR voice here should be:
- brief
- contextual
- not too jokey on urgent alerts

## Phase 7 - Add Social Watchers

Only after the pipeline is stable.

### Telegram Watcher

Use it for:
- missed messages
- command triggers
- reply-needed classification later

### Instagram Watcher

Only once login/session is stable.

Use it for:
- DM received
- important username matched
- unread backlog detected

Do not start with auto-reply.
Start with:
- detect
- classify
- notify

Then move to reply drafting.

### WhatsApp Watcher

Same plan:
- detect
- normalize event
- route through interrupt system

## Phase 8 - Add Priority Brain

Once events exist, AMOR should decide what matters.

Add a small rule engine:
- important contacts
- urgent keywords
- quiet hours
- severity escalation

Examples:
- message from important person -> interrupt immediately
- random message -> batch later
- repeated high temp -> escalate

This can be plain Rust rules first.
Do not use the model for basic priority routing yet.

## Phase 9 - Add Event Inbox / Timeline

You will want visibility fast.

Build a persistent event log:
- append-only file
- or simple SQLite later

Track:
- event emitted
- event deduped
- event delivered
- event acknowledged
- event dismissed

This gives you:
- replay
- debugging
- later UI possibilities

## Phase 10 - Add Acknowledgement Commands

Interruptions become much better when user can control them.

Need commands like:
- acknowledge event
- snooze source
- mute watcher
- show pending alerts
- show recent events

This makes AMOR feel like an actual operations agent.

## Suggested Build Order

1. secret handling
2. event struct
3. interrupt queue
4. reminder watcher migration
5. CPU/system watcher
6. script-output watcher
7. event log
8. acknowledge/snooze commands
9. Telegram social watcher
10. Instagram watcher

## Concrete MVP For This Roadmap

If you want the smallest version that already feels sick:

- [ ] one `Event` struct
- [ ] one in-memory queue
- [ ] one dedupe map
- [ ] reminder watcher emits events
- [ ] CPU temp watcher emits events
- [ ] interrupt sender routes to Telegram
- [ ] urgent events also call TTS

If you finish just that, AMOR will already feel massively more alive.

## Anti-Goals

Do not do these too early:
- [ ] full autonomous Instagram chatting
- [ ] big memory/agent architecture rewrite
- [ ] fancy UI dashboard first
- [ ] model-based prioritization before rule-based routing works

Reason:
- the event spine is the real asset
- everything else should plug into that

## Definition Of Done

This feature is real when:
- AMOR can notice events without prompts
- AMOR can interrupt cleanly without spam
- multiple watchers can run without stepping on each other
- alerts feel intentional, not random
- logs explain what happened when something goes wrong

## Final Note

If you build this properly, AMOR stops being "chatbot on Raspberry Pi."

It becomes:
- a background operator
- a social relay
- a monitoring system
- a personality-driven machine brain

That is the version worth obsessing over.

## Appendix - How To Actually Set Up The Social Side

This is the practical setup path so the big roadmap does not stay abstract.

### WhatsApp Setup Plan

Best path for this project:
- keep the local bridge approach
- use `Baileys` as the transport
- wrap it later with a small MCP server you own

Reason:
- you already have [whatsapp_baileys.js](/home/skedaddle/code/amor/whatsapp_baileys.js)
- it is closer to a stable system than starting over with a random third-party MCP

Build steps:
- [ ] choose `whatsapp_baileys.js` as the one true WhatsApp transport
- [ ] kill the duplicate transport path unless you need it for testing
- [ ] keep auth files out of git
- [ ] standardize inbound event format from WhatsApp into one JSON structure
- [ ] standardize outbound command format into one JSON structure
- [ ] add reconnect handling and watchdog restart
- [ ] add delivery success/failure logging
- [ ] expose local functions like:
- `send_message`
- `list_chats`
- `get_recent_messages`
- `get_contact`
- [ ] then wrap those local functions in a tiny MCP server

First MCP tools to expose:
- `whatsapp_send_message`
- `whatsapp_list_chats`
- `whatsapp_get_recent_messages`
- `whatsapp_get_contact`

Do not make AMOR talk directly to Baileys internals.
Put a clean wrapper in the middle.

### Instagram Setup Plan

Best path for experimentation:
- session/cookie based wrapper first
- then MCP

Do not start from raw username/password login.
It is flaky and gets blocked too easily.

Build steps:
- [ ] get a working browser login first
- [ ] extract session/cookie auth safely
- [ ] store session values outside git
- [ ] build local wrapper functions:
- `instagram_list_threads`
- `instagram_read_thread`
- `instagram_send_dm`
- `instagram_get_user`
- [ ] make read-only flow reliable first
- [ ] only then add sending
- [ ] only then wrap with MCP

First MCP tools to expose:
- `instagram_list_threads`
- `instagram_read_thread`
- `instagram_get_recent_messages`
- `instagram_send_dm`

For long-term reliability:
- official Meta business messaging is stronger
- but for your current personal-account goal, session/cookie bridge is the realistic starting path

### Architecture Rule

All social platforms should follow the same shape:

1. bridge layer
2. normalized event layer
3. normalized action layer
4. MCP wrapper
5. AMOR consumes tools/events

If you obey that rule:
- replacing one platform is easy
- AMOR stays one brain
- transport bugs stay isolated

### Actual Build Order

1. Stabilize WhatsApp Baileys bridge
2. Turn it into a clean local API
3. Wrap it as MCP
4. Get Instagram browser/session auth working
5. Build Instagram local read wrapper
6. Add Instagram send wrapper
7. Wrap Instagram as MCP
8. Feed both into the watcher + interrupt system
9. Feed both into the social world-model

### Documents To Read Next

- [behemoth.md](/home/skedaddle/code/amor/behemoth.md)
- [social_mcp_research.md](/home/skedaddle/code/amor/social_mcp_research.md)
