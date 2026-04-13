# Tools

## Web Search
- Use: SEARCH: query
- Description: Search the web using DuckDuckGo

## File Operations
- Use: READ: /path/to/file
- Use: WRITE: /path | content
- Use: LS: /path

## Reminders
- Use: REMIND: message | minutes

## Workers (background tasks)
- spawn|<task_id>|<command>: Start background worker that runs command and self-kills when done
- status|<task_id>: Check if worker is still running
- kill|<task_id>: Stop a worker
- list: List all active workers

JSON format:
{"tool_calls": [{"function": {"name": "worker", "arguments": "{\"command\": \"spawn|long_task|echo hello\"}"}}]}
{"tool_calls": [{"function": {"name": "worker", "arguments": "{\"command\": \"status|long_task\"}"}}]}