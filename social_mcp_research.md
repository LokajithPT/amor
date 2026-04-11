# Social MCP Research

## Goal

Find the most reliable way to connect AMOR to:
- WhatsApp
- Instagram

with a path that is actually maintainable, not just a demo.

## Bottom Line

### WhatsApp

Best DIY path:
- keep using a local bridge built on `Baileys`
- wrap that bridge with your own small MCP server if you want MCP semantics

Best enterprise / official-ish MCP path:
- use a provider-backed MCP like Infobip if you are okay with business onboarding and provider constraints

### Instagram

For personal account DMs:
- most practical path is session/cookie based integration
- not the official API
- good for experimentation, fragile for long-term production

For real long-term reliability:
- use the official Meta business messaging route
- that means business/professional account constraints, app review, and less freedom

## Recommendation Matrix

### WhatsApp Recommendation

Use:
- `Baileys` as the transport layer
- your current queue model as the first bridge
- a thin MCP wrapper later

Why:
- you already have `whatsapp_baileys.js`
- it already handles inbound and outbound queue flow
- it keeps auth local
- it avoids provider lock-in

Do not switch to a random WhatsApp MCP first.
Wrap the working bridge you control.

### Instagram Recommendation

Split the decision:

If you want personal-account experimentation now:
- use a session/cookie based MCP or local bridge
- expect breakage
- expect re-auth pain
- keep it draft-first, not fully autonomous

If you want real long-term durability:
- move to official Meta business messaging
- accept the product/account restrictions

There is no magic reliable personal-account Instagram DM automation path.

## Concrete Setup Plan

### WhatsApp - Best Path For This Project

You already have:
- [whatsapp_baileys.js](/home/skedaddle/code/amor/whatsapp_baileys.js)
- [whatsapp_bot.js](/home/skedaddle/code/amor/whatsapp_bot.js)

The right move is:

#### Stage 1 - Stabilize The Existing Baileys Bridge

- [ ] standardize one auth directory
- [ ] standardize one inbound event format
- [ ] standardize one outbound command format
- [ ] add delivery success/failure logging
- [ ] add reconnect handling
- [ ] add watchdog restart if socket dies

#### Stage 2 - Convert Bridge To Structured Local API

Instead of text files only, expose:
- `send_message`
- `list_recent_chats`
- `get_chat_messages`
- `mark_seen`

You can still keep the queue files for AMOR compatibility at first.

#### Stage 3 - Wrap That API In MCP

Make a local MCP server with tools like:
- `whatsapp_send_message`
- `whatsapp_list_chats`
- `whatsapp_get_recent_messages`
- `whatsapp_get_contact`

Why this is better:
- transport stays stable
- AMOR gets clean tools
- you do not rewrite your working bridge

### Instagram - Best Path For This Project

#### Stage 1 - Do Not Start With Full DM Automation

Start with:
- session validation
- inbox read
- thread list
- message read

Only after that:
- draft reply
- send reply

#### Stage 2 - Use Session/Cookie Auth For Experiments

Use a local session/cookie flow because:
- raw username/password login is flaky
- session reuse is more stable
- this matches how several Instagram MCP tools are built

Store:
- session id
- csrf token
- ds_user_id

Do not commit these.

#### Stage 3 - Build Local Wrapper First

Before MCP:
- `instagram_list_threads`
- `instagram_read_thread`
- `instagram_send_dm`
- `instagram_get_user`

#### Stage 4 - Wrap In MCP

Then expose those as MCP tools.

AMOR should use them for:
- notify on new DM
- summarize thread
- draft reply
- send approved reply

## Reliability Ranking

### WhatsApp

1. Provider/official business route with MCP wrapper
2. Local `Baileys` bridge with your own MCP wrapper
3. Browser automation / web client hacks

For you, number 2 is the best balance.

### Instagram

1. Official Meta business messaging route
2. Session/cookie local wrapper with MCP
3. Raw password login automation

For you, number 2 is the best experiment path.
For true long-term reliability, number 1 wins.

## How To Make This Actually Work In AMOR

Do not wire social platforms directly into the model.

Do this:
- platform bridge
- normalized event layer
- MCP or tool layer
- AMOR prompt/tool consumer

That means:
- transport bugs stay isolated
- AMOR remains one brain over many channels
- replacing one platform backend does not wreck the whole system

## Recommended Final Architecture

- `bridges/whatsapp` -> Baileys runtime
- `bridges/instagram` -> session/cookie runtime
- `events` -> normalized inbound messages
- `actions` -> normalized outbound commands
- `mcp` -> optional tool facade
- `amor` -> decision and style engine

## Sources Used

- Baileys GitHub
- Infobip MCP docs
- taskmaster-ai `insta-mcp`
- Meta / official Instagram API ecosystem constraints

Use these as anchor points, not gospel.
The main design decision is to own the bridge layer yourself.
