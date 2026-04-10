use crate::tool::ToolResult;

const TOOL_DESC: &str = r#"# Tool: file_ops
Description: Read, write, and list files.
Usage: 
  - "READ: /path/to/file" - read a file
  - "WRITE: /path | content" - write content to file
  - "LS: /path" - list directory contents
"#;

pub struct FileOps;

impl FileOps {
    pub fn new() -> Self {
        Self
    }

    pub fn description(&self) -> &str {
        TOOL_DESC
    }

    pub fn execute(&self, cmd: &str) -> ToolResult {
        let cmd = cmd.trim();

        if let Some(path) = cmd.strip_prefix("READ: ") {
            return self.read_file(path);
        }
        if let Some((path, content)) = cmd.strip_prefix("WRITE: ").and_then(|c| c.split_once('|')) {
            return self.write_file(path, content);
        }
        if let Some(path) = cmd.strip_prefix("LS: ") {
            return self.list_dir(path);
        }
        if let Some((path_and_line, new_content)) =
            cmd.strip_prefix("EDIT: ").and_then(|c| c.split_once('|'))
        {
            let parts: Vec<&str> = path_and_line.splitn(2, ':').collect();
            if parts.len() == 2 {
                if let Ok(line_num) = parts[1].parse::<usize>() {
                    return self.edit_file(parts[0], line_num, new_content);
                }
            }
            return ToolResult::err(
                "EDIT format: path|line_num|new_content. Example: EDIT: /file.rs|10|new line text",
            );
        }

        ToolResult::err("Unknown file_ops command. Use READ:, WRITE:, EDIT:, or LS:")
    }

    fn read_file(&self, path: &str) -> ToolResult {
        match std::fs::read_to_string(path) {
            Ok(content) => ToolResult::ok(content),
            Err(e) => ToolResult::err(format!("Failed to read {}: {}", path, e)),
        }
    }

    fn write_file(&self, path: &str, content: &str) -> ToolResult {
        match std::fs::write(path, content) {
            Ok(_) => ToolResult::ok(format!("Wrote to {}", path)),
            Err(e) => ToolResult::err(format!("Failed to write {}: {}", path, e)),
        }
    }

    fn edit_file(&self, path: &str, line_num: usize, new_content: &str) -> ToolResult {
        match std::fs::read_to_string(path) {
            Ok(mut content) => {
                let lines: Vec<&str> = content.lines().collect();
                let total_lines = lines.len();

                if line_num == 0 || line_num > total_lines {
                    return ToolResult::err(format!(
                        "Line {} out of range (1-{}). Use line number to edit.",
                        line_num, total_lines
                    ));
                }

                let idx = line_num - 1;
                let mut new_lines: Vec<String> = lines
                    .iter()
                    .enumerate()
                    .map(|(i, l)| {
                        if i == idx {
                            new_content.to_string()
                        } else {
                            (*l).to_string()
                        }
                    })
                    .collect();

                content = new_lines.join("\n");
                match std::fs::write(path, &content) {
                    Ok(_) => ToolResult::ok(format!("Edited line {} of {}", line_num, path)),
                    Err(e) => ToolResult::err(format!("Failed to write {}: {}", path, e)),
                }
            }
            Err(e) => ToolResult::err(format!("Failed to read {}: {}", path, e)),
        }
    }

    fn list_dir(&self, path: &str) -> ToolResult {
        match std::fs::read_dir(path) {
            Ok(entries) => {
                let mut lines = Vec::new();
                for entry in entries.flatten() {
                    let path = entry.path();
                    let type_str = if path.is_dir() { "/" } else { "" };
                    if let Some(name) = path.file_name() {
                        lines.push(format!("{}{}", name.to_string_lossy(), type_str));
                    }
                }
                ToolResult::ok(lines.join("\n"))
            }
            Err(e) => ToolResult::err(format!("Failed to list {}: {}", path, e)),
        }
    }
}

impl Default for FileOps {
    fn default() -> Self {
        Self::new()
    }
}
