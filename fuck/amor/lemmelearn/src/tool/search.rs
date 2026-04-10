use crate::tool::ToolResult;
use reqwest::Client;

const TOOL_DESC: &str = r#"# Tool: web_search
Description: Search the web using DuckDuckGo.
Usage: "SEARCH: your search query"
Returns: Clean text results with titles and snippets
"#;

pub struct WebSearch {
    client: Client,
}

impl WebSearch {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub fn description(&self) -> &str {
        TOOL_DESC
    }

    pub async fn execute(&self, query: &str) -> ToolResult {
        let url = "https://html.duckduckgo.com/html/";
        
        let resp = self.client
            .post(url)
            .form(&[("q", query)])
            .send()
            .await;

        match resp {
            Ok(html) => {
                let text = html.text().await.unwrap_or_default();
                let mut results = Vec::new();
                
                // Find all result blocks
                let mut in_result = false;
                let mut title = String::new();
                let mut snippet = String::new();
                let mut url = String::new();
                
                for line in text.lines() {
                    let line = line.trim();
                    
                    // Start of result
                    if line.contains("result__body") {
                        in_result = true;
                        title.clear();
                        snippet.clear();
                        url.clear();
                    }
                    
                    // Extract title
                    if in_result && line.contains("result__title") {
                        if let Some(start) = line.find(">") {
                            if let Some(end) = line[start..].find("<") {
                                title = line[start+1..start+end]
                                    .replace("<b>", "")
                                    .replace("</b>", "")
                                    .trim()
                                    .to_string();
                            }
                        }
                    }
                    
                    // Extract snippet
                    if in_result && line.contains("result__snippet") {
                        if let Some(start) = line.find(">") {
                            if let Some(end) = line[start..].find("<") {
                                snippet = line[start+1..start+end]
                                    .replace("<b>", "")
                                    .replace("</b>", "")
                                    .replace("&amp;", "&")
                                    .replace("&quot;", "\"")
                                    .replace("&apos;", "'")
                                    .replace("&lt;", "<")
                                    .replace("&gt;", ">")
                                    .trim()
                                    .to_string();
                            }
                        }
                    }
                    
                    // Extract link
                    if in_result && line.contains("result__url") {
                        if let Some(start) = line.find(">") {
                            if let Some(end) = line[start..].find("<") {
                                url = line[start+1..start+end].trim().to_string();
                            }
                        }
                    }
                    
                    // End of result - save it
                    if in_result && line.contains("result__footer") {
                        if !title.is_empty() {
                            let mut result = format!("📌 {}\n", title);
                            if !snippet.is_empty() {
                                result.push_str(&format!("   {}\n", snippet));
                            }
                            if !url.is_empty() {
                                result.push_str(&format!("   🔗 {}\n", url));
                            }
                            results.push(result);
                        }
                        in_result = false;
                    }
                }
                
                let result = if results.is_empty() {
                    text.lines()
                        .filter(|l| l.contains("result__title") || l.contains("result__snippet"))
                        .filter_map(|l| {
                            let cleaned = l.replace("<b>", "").replace("</b>", "");
                            cleaned.find(">").and_then(|s| {
                                cleaned[s..].find("<").map(|e| cleaned[s+1..s+e].trim().to_string())
                            })
                        })
                        .take(5)
                        .enumerate()
                        .map(|(i, r)| format!("{}. {}", i + 1, r))
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    results.join("")
                };
                
                let result = if result.is_empty() {
                    "No results found".to_string()
                } else if result.len() > 800 {
                    format!("{}...", &result[..800])
                } else {
                    result
                };
                
                ToolResult::ok(result)
            }
            Err(e) => ToolResult::err(format!("Search failed: {}", e)),
        }
    }
}

impl Default for WebSearch {
    fn default() -> Self {
        Self::new()
    }
}
