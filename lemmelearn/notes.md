# AMOR Development Notes

## Current Status (2026-04-09)

### Working ✅
- Telegram bot receiving and responding
- API calls using curl 
- Memory (memsave/memrecall)
- Web search
- File operations
- Console mode
- `amor serve` / `amor` commands with auto-restart
- TTS with Ryan voice (only speaks for reminders/alerts)
- **REMINDERS ARE NOW WORKING!** 🎉

### Latest Success (2026-04-09 19:04)
- Reminder fired after 2 minutes!
- Model still tries Python scripts but fallback catches them
- Added redirect for execute_command if it looks like a timer
- Updated master.md with STRICT warnings about reminders

## Files Modified
- `/home/skedaddle/code/amor/lemmelearn/amorshi/master.md` - Updated with strong identity + strict reminder rules
- `/home/skedaddle/code/amor/lemmelearn/src/tool/mod.rs` - Added timer redirect from execute_command
- `/home/skedaddle/code/amor/lemmelearn/src/main.rs` - reminder tool handling

## Commands
- `amor serve` - Start Telegram bot (24/7)
- `amor` - Console mode (kills service, auto-restarts on exit)
- `amor kill` - Kill service manually

---

## Previous Session Notes (kept for reference):

### import statements 
- **serde** : with serialize, deserialize, debug, clone
- **env** -> to read env of groq

### Structs
- `Message` : role: String, content: String
- `ChatRequest` : model: String, messages: Vec<Message>
- `Choice` : message: String  
- `ChatResponse` : choices: Vec<Choice>

### Main Function
- API key -> groq api key
- client -> reqwest client
- chat_hist -> vector of message struct
- model -> the model

### Flow
1. Get input
2. Push to chat_hist
3. Build payload
4. Send to Groq API
5. Parse response
6. Print and push to chat_hist