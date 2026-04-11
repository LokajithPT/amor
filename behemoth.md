# The Behemoth - Social World Model

## The Real Ambition

Build AMOR into a persistent social operator that maintains a live world model of:
- people
- conversations
- platforms
- relationship state
- message timing
- priorities
- unfinished threads
- next best action

This is not just memory.
This is a living graph of your social world.

If it works, AMOR stops being:
- a chatbot
- a bag of notes
- a reply generator

And becomes:
- a relationship operating system
- a social strategist
- a continuity engine
- a machine personality that actually knows the state of your world

## What Makes This Different

Normal assistant memory:
- stores facts
- maybe retrieves a few notes

This system:
- knows that `person A` on Telegram and `person A` on Instagram are probably the same human
- knows who matters
- knows how each person talks
- knows what happened last
- knows whether a thread is hot, cold, dead, tense, playful, unresolved, or needs action
- knows whether AMOR should interrupt, suggest, wait, or ignore

## Core Data Model

You need 5 layers.

### 1. Identity Layer

One entity per human.

Fields:
- `person_id`
- real name
- aliases
- usernames by platform
- phone numbers
- confidence score for identity merge
- tags like `important`, `friend`, `lead`, `romantic`, `avoid`

### 2. Channel Layer

One entity per account/thread endpoint.

Examples:
- Telegram chat id
- WhatsApp jid
- Instagram thread id

Fields:
- `channel_id`
- `person_id`
- platform
- account identifier
- inbox state
- sync cursor

### 3. Conversation Layer

One entity per thread.

Fields:
- `conversation_id`
- `person_id`
- platform
- current status
- last inbound
- last outbound
- unread count
- last topic summary
- next action deadline

### 4. Style Layer

A practical behavior profile per person.

Fields:
- average reply delay
- message length preference
- emoji tolerance
- directness preference
- humor tolerance
- formality
- flirt energy
- reliability score

This should start rule-based.
Do not overcomplicate it with embeddings on day one.

### 5. Strategy Layer

What AMOR thinks should happen next.

Fields:
- urgency
- emotional tone
- momentum score
- risk score
- recommended action
- draft candidate
- follow-up timer

## Minimum Architecture

### Storage

Use SQLite first.

Tables:
- `people`
- `aliases`
- `channels`
- `conversations`
- `messages`
- `events`
- `style_profiles`
- `pending_actions`

Why SQLite:
- simple
- local
- inspectable
- good enough for a Pi

### Services

Split into services, not one blob.

Suggested modules:
- `identity`
- `ingest`
- `conversation_state`
- `style_profiler`
- `priority_engine`
- `drafting`
- `watchers`
- `interrupts`

## Build Order

### Phase 1 - Unified Ingestion

Goal:
- all platforms feed one normalized message format

Normalized message fields:
- message id
- platform
- sender id
- thread id
- timestamp
- text
- media flag
- direction in/out
- raw payload pointer

Do this before any "intelligence."

### Phase 2 - Identity Merge

Goal:
- merge cross-platform identities carefully

Start with hard matches:
- exact phone number
- exact linked usernames
- manual user-confirmed merges

Then add soft matches:
- same display name
- same writing patterns
- same linked metadata

Important:
- keep merge confidence
- do not auto-merge aggressively

### Phase 3 - Conversation State Machine

Each conversation should have a status.

Example statuses:
- `new`
- `active`
- `waiting_on_them`
- `waiting_on_us`
- `stalled`
- `needs_followup`
- `muted`
- `high_priority`

This lets AMOR reason in operational terms instead of loose vibes.

### Phase 4 - Style Profiling

Goal:
- learn how each person talks and how they respond best

Start with simple measurable signals:
- mean reply latency
- average length
- punctuation style
- common slang
- emoji density
- initiation frequency

Then infer tags:
- low-effort
- warm
- cold
- practical
- chaotic
- formal
- playful

### Phase 5 - Priority Engine

Goal:
- decide who matters now

Base signals:
- importance tag
- unread age
- inbound sentiment
- action request detection
- long silence after your last message
- recurrence of contact

Output:
- priority score
- recommended action time

### Phase 6 - Follow-up Engine

This is where AMOR starts to feel alive.

For each conversation:
- detect whether a reply is owed
- detect whether waiting is smarter
- detect whether thread is dying
- detect whether user should re-engage

Actions:
- suggest reply
- remind later
- interrupt now
- do nothing

### Phase 7 - Drafting Engine

Only after the state model is good.

Inputs:
- person
- style profile
- thread summary
- recent messages
- objective

Outputs:
- 1 short draft
- 1 bolder draft
- 1 safer draft

Important:
- AMOR should not send automatically by default
- default should be draft-first, approve-then-send

### Phase 8 - Self-Updating Relationship Memory

Every important interaction updates:
- style
- trust
- tension
- momentum
- next action

This is the heart of the system.

## The "No One Does This Well" Part

Most systems fail because they:
- only store messages
- do not model thread state
- do not merge identities
- do not track relationship momentum
- do not decide whether to speak now or later

AMOR should do all of that.

## What Makes It Actually Doable

Keep it modular.

Do not try to build full social AGI.
Build a ruthless operator stack:

1. normalize
2. merge
3. classify
4. prioritize
5. draft
6. interrupt

That is enough.

## Practical First MVP

If you want a brutal but achievable first version:

- [ ] ingest Telegram + WhatsApp + Instagram into one `messages` table
- [ ] create `people` and `conversations` tables
- [ ] add manual merge command for identities
- [ ] track last inbound/outbound timestamps
- [ ] tag conversations as `waiting_on_them` or `waiting_on_us`
- [ ] compute simple priority score
- [ ] show "top 10 threads that need action"
- [ ] let AMOR generate 3 reply options for one selected thread

That alone would already be wild.

## What To Avoid

Do not do these first:
- vector database obsession
- emotional AI nonsense
- fully autonomous sending
- giant multi-agent architecture
- trying to infer too much from too little data

You need a hard spine first.

## Success Condition

This behemoth is real when AMOR can answer:
- who matters right now
- who is waiting on me
- which thread is cooling off
- how should I reply to this person
- should I ping them now or later
- have I already said something similar elsewhere
- is this the same person across platforms

That is the line between "assistant" and "operator."
