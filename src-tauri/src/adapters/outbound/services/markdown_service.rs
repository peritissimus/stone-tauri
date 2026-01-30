//! Markdown Service Implementation
//!
//! Implements the MarkdownProcessor port using pulldown-cmark for parsing
//! and conversion operations.

use async_trait::async_trait;
use pulldown_cmark::{html, Event, Options, Parser, Tag};
use regex::Regex;
use std::collections::HashMap;

use crate::domain::{
    errors::DomainResult,
    ports::outbound::{MarkdownLink, MarkdownMetadata, MarkdownProcessor, ParsedMarkdown},
};

/// Markdown processor implementation using pulldown-cmark
///
/// Note: Task markers and timestamps are now parsed on the frontend via prosemirror-markdown.
/// This service is primarily used for:
/// - HTML export/preview
/// - Wiki link extraction
/// - Frontmatter parsing
/// - Title extraction
pub struct PulldownMarkdownService {
    wiki_link_regex: Regex,
    frontmatter_regex: Regex,
}

impl PulldownMarkdownService {
    pub fn new() -> Self {
        Self {
            // Matches [[link]] or [[link|display text]]
            wiki_link_regex: Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").unwrap(),
            // Matches YAML frontmatter (--- ... ---)
            frontmatter_regex: Regex::new(r"^---\s*\n(.*?)\n---\s*\n").unwrap(),
        }
    }

    /// Parse YAML frontmatter from content
    fn parse_yaml_frontmatter(&self, yaml: &str) -> DomainResult<MarkdownMetadata> {
        let mut metadata = MarkdownMetadata {
            title: None,
            tags: None,
            created: None,
            modified: None,
            extra: HashMap::new(),
        };

        // Simple YAML parser for common fields
        for line in yaml.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim().trim_matches('"').trim_matches('\'');

                match key {
                    "title" => metadata.title = Some(value.to_string()),
                    "created" => metadata.created = Some(value.to_string()),
                    "modified" | "updated" => metadata.modified = Some(value.to_string()),
                    "tags" => {
                        // Handle both array format and comma-separated
                        if value.starts_with('[') && value.ends_with(']') {
                            let tags_str = value.trim_matches(|c| c == '[' || c == ']');
                            let tags: Vec<String> = tags_str
                                .split(',')
                                .map(|t| t.trim().trim_matches('"').trim_matches('\'').to_string())
                                .filter(|t| !t.is_empty())
                                .collect();
                            metadata.tags = Some(tags);
                        } else {
                            let tags: Vec<String> = value
                                .split(',')
                                .map(|t| t.trim().to_string())
                                .filter(|t| !t.is_empty())
                                .collect();
                            metadata.tags = Some(tags);
                        }
                    }
                    _ => {
                        // Store other fields in extra
                        metadata.extra.insert(
                            key.to_string(),
                            serde_json::Value::String(value.to_string()),
                        );
                    }
                }
            }
        }

        Ok(metadata)
    }

    /// Remove HTML tags from text
    fn strip_html_tags(&self, html: &str) -> String {
        let tag_regex = Regex::new(r"<[^>]*>").unwrap();
        let result = tag_regex.replace_all(html, "");
        // Decode common HTML entities
        result
            .replace("&nbsp;", " ")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
    }
}

impl Default for PulldownMarkdownService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MarkdownProcessor for PulldownMarkdownService {
    fn html_to_markdown(&self, html: &str) -> DomainResult<String> {
        // Simple HTML to Markdown conversion
        // For more complex conversions, we'd use the html2md crate
        // For now, implement a basic version
        let mut markdown = html.to_string();

        // Convert common HTML tags to markdown
        markdown = markdown.replace("<br>", "\n");
        markdown = markdown.replace("<br/>", "\n");
        markdown = markdown.replace("<br />", "\n");

        // Headers
        for i in 1..=6 {
            let open_tag = format!("<h{}>", i);
            let close_tag = format!("</h{}>", i);
            let hashes = "#".repeat(i);

            let regex = Regex::new(&format!(r"{}\s*(.*?)\s*{}", regex::escape(&open_tag), regex::escape(&close_tag))).unwrap();
            markdown = regex.replace_all(&markdown, format!("{} $1\n", hashes)).to_string();
        }

        // Bold
        markdown = Regex::new(r"<strong>(.*?)</strong>")
            .unwrap()
            .replace_all(&markdown, "**$1**")
            .to_string();
        markdown = Regex::new(r"<b>(.*?)</b>")
            .unwrap()
            .replace_all(&markdown, "**$1**")
            .to_string();

        // Italic
        markdown = Regex::new(r"<em>(.*?)</em>")
            .unwrap()
            .replace_all(&markdown, "*$1*")
            .to_string();
        markdown = Regex::new(r"<i>(.*?)</i>")
            .unwrap()
            .replace_all(&markdown, "*$1*")
            .to_string();

        // Code
        markdown = Regex::new(r"<code>(.*?)</code>")
            .unwrap()
            .replace_all(&markdown, "`$1`")
            .to_string();

        // Links
        markdown = Regex::new(r#"<a\s+href="([^"]*)"[^>]*>(.*?)</a>"#)
            .unwrap()
            .replace_all(&markdown, "[$2]($1)")
            .to_string();

        // Paragraphs
        markdown = markdown.replace("<p>", "\n");
        markdown = markdown.replace("</p>", "\n");

        // Lists
        markdown = markdown.replace("<ul>", "");
        markdown = markdown.replace("</ul>", "");
        markdown = markdown.replace("<ol>", "");
        markdown = markdown.replace("</ol>", "");
        markdown = Regex::new(r"<li>(.*?)</li>")
            .unwrap()
            .replace_all(&markdown, "- $1\n")
            .to_string();

        // Strip any remaining HTML tags
        markdown = self.strip_html_tags(&markdown);

        // Clean up extra newlines
        markdown = Regex::new(r"\n{3,}")
            .unwrap()
            .replace_all(&markdown, "\n\n")
            .to_string();

        Ok(markdown.trim().to_string())
    }

    async fn markdown_to_html(&self, markdown: &str) -> DomainResult<String> {
        // Note: This method is now primarily used for exports/previews only.
        // The frontend handles markdown parsing directly via prosemirror-markdown,
        // so we no longer need to post-process for custom node types (task markers,
        // timestamps, etc.) - those are parsed on the frontend.
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

        let parser = Parser::new_ext(markdown, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        Ok(html_output)
    }

    fn parse_frontmatter(&self, markdown: &str) -> DomainResult<ParsedMarkdown> {
        if let Some(captures) = self.frontmatter_regex.captures(markdown) {
            let yaml = captures.get(1).map(|m| m.as_str()).unwrap_or("");
            let metadata = self.parse_yaml_frontmatter(yaml)?;

            // Remove frontmatter from content
            let content = self.frontmatter_regex.replace(markdown, "").to_string();

            Ok(ParsedMarkdown { content, metadata })
        } else {
            // No frontmatter found
            Ok(ParsedMarkdown {
                content: markdown.to_string(),
                metadata: MarkdownMetadata {
                    title: None,
                    tags: None,
                    created: None,
                    modified: None,
                    extra: HashMap::new(),
                },
            })
        }
    }

    fn update_frontmatter(
        &self,
        markdown: &str,
        metadata: &MarkdownMetadata,
    ) -> DomainResult<String> {
        // Build YAML frontmatter
        let mut yaml = String::from("---\n");

        if let Some(title) = &metadata.title {
            yaml.push_str(&format!("title: \"{}\"\n", title));
        }

        if let Some(tags) = &metadata.tags {
            let tags_str = tags
                .iter()
                .map(|t| format!("\"{}\"", t))
                .collect::<Vec<_>>()
                .join(", ");
            yaml.push_str(&format!("tags: [{}]\n", tags_str));
        }

        if let Some(created) = &metadata.created {
            yaml.push_str(&format!("created: \"{}\"\n", created));
        }

        if let Some(modified) = &metadata.modified {
            yaml.push_str(&format!("modified: \"{}\"\n", modified));
        }

        // Add extra fields
        for (key, value) in &metadata.extra {
            match value {
                serde_json::Value::String(s) => yaml.push_str(&format!("{}: \"{}\"\n", key, s)),
                serde_json::Value::Number(n) => yaml.push_str(&format!("{}: {}\n", key, n)),
                serde_json::Value::Bool(b) => yaml.push_str(&format!("{}: {}\n", key, b)),
                _ => yaml.push_str(&format!("{}: {}\n", key, value)),
            }
        }

        yaml.push_str("---\n\n");

        // Remove existing frontmatter if present
        let content_without_frontmatter = if self.frontmatter_regex.is_match(markdown) {
            self.frontmatter_regex.replace(markdown, "").to_string()
        } else {
            markdown.to_string()
        };

        Ok(format!("{}{}", yaml, content_without_frontmatter))
    }

    fn extract_title(&self, markdown: &str) -> DomainResult<Option<String>> {
        // First check frontmatter
        if let Ok(parsed) = self.parse_frontmatter(markdown) {
            if let Some(title) = parsed.metadata.title {
                return Ok(Some(title));
            }
        }

        // Then look for first H1
        let parser = Parser::new(markdown);
        let mut in_heading = false;
        let mut heading_text = String::new();

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) if level.clone() as u32 == 1 => {
                    in_heading = true;
                }
                Event::End(pulldown_cmark::TagEnd::Heading(_)) if in_heading => {
                    return Ok(Some(heading_text.trim().to_string()));
                }
                Event::Text(text) if in_heading => {
                    heading_text.push_str(&text);
                }
                _ => {}
            }
        }

        Ok(None)
    }

    fn extract_plain_text(&self, markdown: &str) -> DomainResult<String> {
        let parser = Parser::new(markdown);
        let mut plain_text = String::new();

        for event in parser {
            if let Event::Text(text) = event {
                plain_text.push_str(&text);
                plain_text.push(' ');
            }
        }

        Ok(plain_text.trim().to_string())
    }

    fn extract_links(&self, markdown: &str) -> DomainResult<Vec<MarkdownLink>> {
        let parser = Parser::new(markdown);
        let mut links = Vec::new();
        let mut current_link_url: Option<String> = None;
        let mut current_link_text = String::new();

        for event in parser {
            match event {
                Event::Start(Tag::Link { dest_url, .. }) => {
                    current_link_url = Some(dest_url.to_string());
                    current_link_text.clear();
                }
                Event::End(pulldown_cmark::TagEnd::Link) => {
                    if let Some(url) = current_link_url.take() {
                        links.push(MarkdownLink {
                            text: current_link_text.clone(),
                            href: url,
                        });
                        current_link_text.clear();
                    }
                }
                Event::Text(text) if current_link_url.is_some() => {
                    current_link_text.push_str(&text);
                }
                _ => {}
            }
        }

        Ok(links)
    }

    fn extract_wiki_links(&self, markdown: &str) -> DomainResult<Vec<String>> {
        let mut wiki_links = Vec::new();

        for captures in self.wiki_link_regex.captures_iter(markdown) {
            if let Some(link) = captures.get(1) {
                wiki_links.push(link.as_str().to_string());
            }
        }

        Ok(wiki_links)
    }

    fn html_to_plain_text(&self, html: &str) -> DomainResult<String> {
        let text = self.strip_html_tags(html);
        // Clean up extra whitespace
        let text = Regex::new(r"\s+")
            .unwrap()
            .replace_all(&text, " ")
            .to_string();
        Ok(text.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_markdown_to_html() {
        let service = PulldownMarkdownService::new();
        let markdown = "# Hello\n\nThis is **bold** and *italic*.";
        let html = service.markdown_to_html(markdown).await.unwrap();

        assert!(html.contains("<h1>"));
        assert!(html.contains("Hello"));
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
    }

    #[tokio::test]
    async fn test_html_to_markdown() {
        let service = PulldownMarkdownService::new();
        let html = "<h1>Hello</h1><p>This is <strong>bold</strong> and <em>italic</em>.</p>";
        let markdown = service.html_to_markdown(html).unwrap();

        assert!(markdown.contains("# Hello"));
        assert!(markdown.contains("**bold**"));
        assert!(markdown.contains("*italic*"));
    }

    #[test]
    fn test_parse_frontmatter() {
        let service = PulldownMarkdownService::new();
        let markdown = r#"---
title: "My Note"
tags: [rust, programming]
created: "2024-01-01"
---

# Content here"#;

        let parsed = service.parse_frontmatter(markdown).unwrap();
        assert_eq!(parsed.metadata.title, Some("My Note".to_string()));
        assert_eq!(
            parsed.metadata.tags,
            Some(vec!["rust".to_string(), "programming".to_string()])
        );
        assert!(parsed.content.contains("# Content here"));
        assert!(!parsed.content.contains("---"));
    }

    #[test]
    fn test_update_frontmatter() {
        let service = PulldownMarkdownService::new();
        let markdown = "# Hello World";
        let metadata = MarkdownMetadata {
            title: Some("Test Title".to_string()),
            tags: Some(vec!["test".to_string()]),
            created: None,
            modified: None,
            extra: HashMap::new(),
        };

        let result = service.update_frontmatter(markdown, &metadata).unwrap();
        assert!(result.contains("---"));
        assert!(result.contains("title: \"Test Title\""));
        assert!(result.contains("tags: [\"test\"]"));
        assert!(result.contains("# Hello World"));
    }

    #[test]
    fn test_extract_title_from_h1() {
        let service = PulldownMarkdownService::new();
        let markdown = "# My Title\n\nSome content";
        let title = service.extract_title(markdown).unwrap();
        assert_eq!(title, Some("My Title".to_string()));
    }

    #[test]
    fn test_extract_title_from_frontmatter() {
        let service = PulldownMarkdownService::new();
        let markdown = r#"---
title: "Frontmatter Title"
---

# H1 Title

Content"#;

        let title = service.extract_title(markdown).unwrap();
        // Should prefer frontmatter title
        assert_eq!(title, Some("Frontmatter Title".to_string()));
    }

    #[test]
    fn test_extract_plain_text() {
        let service = PulldownMarkdownService::new();
        let markdown = "# Title\n\nThis is **bold** and *italic*.";
        let plain = service.extract_plain_text(markdown).unwrap();
        assert_eq!(plain, "Title This is bold and italic.");
    }

    #[test]
    fn test_extract_links() {
        let service = PulldownMarkdownService::new();
        let markdown = "[Google](https://google.com) and [GitHub](https://github.com)";
        let links = service.extract_links(markdown).unwrap();

        assert_eq!(links.len(), 2);
        assert_eq!(links[0].text, "Google");
        assert_eq!(links[0].href, "https://google.com");
        assert_eq!(links[1].text, "GitHub");
        assert_eq!(links[1].href, "https://github.com");
    }

    #[test]
    fn test_extract_wiki_links() {
        let service = PulldownMarkdownService::new();
        let markdown = "Link to [[Other Note]] and [[Another Note|Display Text]].";
        let wiki_links = service.extract_wiki_links(markdown).unwrap();

        assert_eq!(wiki_links.len(), 2);
        assert_eq!(wiki_links[0], "Other Note");
        assert_eq!(wiki_links[1], "Another Note");
    }

    #[test]
    fn test_html_to_plain_text() {
        let service = PulldownMarkdownService::new();
        let html = "<h1>Title</h1><p>This is <strong>bold</strong> text.</p>";
        let plain = service.html_to_plain_text(html).unwrap();
        assert_eq!(plain, "Title This is bold text.");
    }
}
