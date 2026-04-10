# AMOR Project Notes

## Current Status (April 8, 2026)

### What is AMOR?
AI assistant running on Raspberry Pi 4B (Arch Linux). Uses Groq API with moonshotai/kimi-k2-instruct-0905 model. Can use tools autonomously by calling them in response then getting result back.

### Working Features ✅
- Chat with Groq API
- Tool calling loop (model → tool → result → model answer)
- All 12 tools working:
  - websearch, file_read, file_write, ls
  - remind, memsave, memrecall
  - script_create, script_run, script_delete, ls_scripts
  - bash (FREE - no hesitation needed)
- Config from shit.cfg (API keys, model selection)
- Master prompt from master.md sent as system message
- Memory system saves only what user tells it
- Scripts folder for Python scripts

### Known Issues ⚠️
- file_read / file_write syntax is finicky (todo)
- websearch returns raw HTML (parsed but messy)
- No conversation history limit (grows forever)
- No error handling for bad paths

---

## Thought Process (as I build)

### Step 1: Config & Files Loading
**Why:** Need to load API key, model, master prompt from files instead of hardcoding.

**How:** Created config.rs that reads amorshi/ folder from current working directory.

**Files created:**
- amorshi/shit.cfg (JSON with accounts, active_account, model)
- amorshi/master.md (system prompt)
- amorshi/reminders.md, tools.md, memory.md

### Step 2: Master Prompt
**Why:** Tell the model who it is, how to use tools.

**Current prompt (simple):**
```
# AMOR
You are AMOR on Raspberry Pi 4B. Be chill and loose.
Tools: websearch, file_read, file_write, ls, remind, memsave, memrecall, scripts, bash
bash has NO HESITATION - run freely when useful!

**Fixed 4/8:** Model was saying "I'll remember that" without calling tool. Fixed by adding "When asked to do something, call the tool NOW".
```

**Thought:** Keep it short because longer prompts = more tokens = hit rate limits faster.

### Step 3: Tool System
**Why:** Model needs to call tools, not just respond text.

**Approach:**
1. Parse tool calls from model response
2. Execute tool
3. Add result to chat history
4. Send back to model
5. Model responds with answer

**Tool syntax design:**
- Use `"query"` quotes because model understands it
- Put on separate line for easier parsing
- Example: `websearch:"python tutorial"`

### Refinement: One tool at a time
**Problem:** Model dumps multiple commands, adds extra text.
**Fix:** Updated master.md to say "Run ONE tool call. Wait for result."

**Fixed 4/8:** Tool loop now clean.

Flow now:
1. User → API
2. Model calls tool → execute → show "→ result" to user
3. Loop back → model sees result → gives clean answer
4. User sees: tool result → final answer

### Step 4: Each Tool

#### websearch
Thought: Use DuckDuckGo HTML API (simpler than JSON). Parse results to get titles/snippets, not full HTML.

#### file_read/file_write
Current syntax: `file_read:"path"`, `file_write:"path|content"`
Issue: Model sometimes puts tool call mid-sentence. Need to search entire response, not just line by line.

#### memsave/memrecall
Why: User doesn't want full chat history. Only important stuff.
Thought: Save to memory.md with timestamp. Search by substring match.

#### scripts
Why: Run Python for sensor, etc.
Created: amorshi/scripts/ folder
Thought: User wants to create script, run it, delete it. Separate folder keeps clean.

#### bash
Why: Need to run any shell command.
Thought: Give it free rein. No permission needed. User said "no hesitation".

### Step 5: Tool Loop
Problem: Model might call multiple tools or call tool again after result.
Solution: While loop, max 3 iterations to prevent infinite.

---

## Files Created

```
lemmelearn/
├── Cargo.toml
├── src/
│   ├── main.rs         # Chat loop, tool execution loop
│   ├── config.rs      # Load amorshi files
│   └── tool/
│       ├── mod.rs     # ToolExecutor, tool parsing
│       ├── search.rs  # DuckDuckGo websearch
│       ├── files.rs  # READ, WRITE, LS
│       ├── reminders.rs
│       ├── memory.rs  # memsave, memrecall
│       ├── scripts.rs # script_create/run/delete
│       └── bash.rs   # shell commands
├── amorshi/
│   ├── shit.cfg      # API keys + model
│   ├── master.md    # system prompt
│   ├── memory.md   # saved memories
│   ├── reminders.md
│   ├── tools.md
│   └── scripts/    # python scripts
└── todo.md

Root:
├── forme.md        # THIS FILE
└── MVP.md        # original spec
```

---

## Running AMOR

```bash
cd lemmelearn && cargo run
```

Then type. Tools work when model calls them.

Example session:
```
> who are you
AMOR, on Pi 4B.

> bash:"echo hi"
[bash] → hi

> memsave:"user is skedaddle"
Saved

> memrecall:"skedaddle"
Found: user is skedaddle
```

---

## For Future Reference

### API Key
`REPLACE_WITH_GROQ_KEY`

### Model
`moonshotai/kimi-k2-instruct-0905`

### Todo
- Fix file_read/file_write syntax
- Add conversation limit/truncate
- Better HTML parsing for websearch
- Add tests

### Context Reset
If context resets, read this file and I'll remember everything.
