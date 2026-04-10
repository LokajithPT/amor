# TODO - AMOR Build

## Phase 1: Config & Files Loading
- [x] Create `amorshi/` folder next to executable
- [x] Create `amorshi/shit.cfg` - JSON config with API keys, model selection
- [x] Create `amorshi/master.md` - system prompt (placeholder)
- [x] Create `amorshi/reminders.md` - reminders (placeholder)
- [x] Create `amorshi/tools.md` - tool descriptions (placeholder)
- [x] Load all files on startup in Rust

## Phase 2: Tools System
- [x] Parse `tools.md` format
- [x] Create tool syntax parser (websearch:, file_read:, etc.)
- [x] Wire tools in `tool/` to work with parsed commands
- [x] Tool execution loop (model calls tool → result → model answers)

## Phase 3: Memory & Scripts
- [x] memsave:"remember this" - save to memory
- [x] memrecall:"search term" - recall from memory
- [x] script_create:"name|content" - create script
- [x] script_run:"name" - run script
- [x] script_delete:"name" - delete script
- [x] ls_scripts - list scripts

## Phase 3: Memory System
- [ ] Load conversation history on startup
- [ ] Save messages to memory

## Phase 4: Testing
- [ ] Test file loading works
- [ ] Test config parsing
- [ ] Test tool execution

---

## Notes
- Config format: JSON
- amorshi folder: from working directory (where cargo run)
- Groq only (no local models for now)