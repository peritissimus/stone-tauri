/// LinkExtractor - Pure domain service for extracting links from markdown
///
/// Handles wiki-style links [[Note Title]] and standard markdown links [text](url)
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Types of links found in markdown
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkType {
    Wiki,
    Markdown,
    Url,
}

/// Extracted link from markdown content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedLink {
    pub link_type: LinkType,
    pub target: String,     // Link target (note title for wiki, path for markdown, URL for external)
    pub text: String,       // Display text
    pub line_number: usize, // 1-based
    pub start_index: usize, // Character offset in entire document
    pub end_index: usize,   // Character offset in entire document
}

/// LinkExtractor - Pure functions for link extraction
pub struct LinkExtractor;

impl LinkExtractor {
    /// Extract wiki-style links [[Note Title]] or [[Note Title|Display Text]]
    pub fn extract_wiki_links(markdown: &str) -> Vec<ExtractedLink> {
        let mut links = Vec::new();
        let pattern = Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").unwrap();

        let mut char_offset = 0;

        for (line_num, line) in markdown.lines().enumerate() {
            for caps in pattern.captures_iter(line) {
                if let Some(target_match) = caps.get(1) {
                    let target = target_match.as_str().trim().to_string();
                    let text = caps
                        .get(2)
                        .map(|m| m.as_str().trim().to_string())
                        .unwrap_or_else(|| target.clone());

                    if let Some(full_match) = caps.get(0) {
                        links.push(ExtractedLink {
                            link_type: LinkType::Wiki,
                            target,
                            text,
                            line_number: line_num + 1,
                            start_index: char_offset + full_match.start(),
                            end_index: char_offset + full_match.end(),
                        });
                    }
                }
            }

            char_offset += line.len() + 1; // +1 for newline
        }

        links
    }

    /// Extract standard markdown links [text](url)
    pub fn extract_markdown_links(markdown: &str) -> Vec<ExtractedLink> {
        let mut links = Vec::new();
        let pattern = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
        let url_pattern = Regex::new(r"^https?://").unwrap();

        let mut char_offset = 0;

        for (line_num, line) in markdown.lines().enumerate() {
            for caps in pattern.captures_iter(line) {
                if let (Some(text_match), Some(target_match)) = (caps.get(1), caps.get(2)) {
                    let text = text_match.as_str().trim().to_string();
                    let target = target_match.as_str().trim().to_string();
                    let is_external = url_pattern.is_match(&target);

                    if let Some(full_match) = caps.get(0) {
                        links.push(ExtractedLink {
                            link_type: if is_external {
                                LinkType::Url
                            } else {
                                LinkType::Markdown
                            },
                            target,
                            text,
                            line_number: line_num + 1,
                            start_index: char_offset + full_match.start(),
                            end_index: char_offset + full_match.end(),
                        });
                    }
                }
            }

            char_offset += line.len() + 1;
        }

        links
    }

    /// Extract all links (wiki + markdown)
    pub fn extract_all_links(markdown: &str) -> Vec<ExtractedLink> {
        let mut wiki_links = Self::extract_wiki_links(markdown);
        let mut markdown_links = Self::extract_markdown_links(markdown);

        // Combine and sort by position
        wiki_links.append(&mut markdown_links);
        wiki_links.sort_by_key(|link| link.start_index);
        wiki_links
    }

    /// Extract only internal note links (wiki + local markdown links)
    pub fn extract_internal_links(markdown: &str) -> Vec<ExtractedLink> {
        Self::extract_all_links(markdown)
            .into_iter()
            .filter(|link| matches!(link.link_type, LinkType::Wiki | LinkType::Markdown))
            .collect()
    }

    /// Get unique note titles referenced in the content
    pub fn get_referenced_note_titles(markdown: &str) -> Vec<String> {
        let internal_links = Self::extract_internal_links(markdown);
        let mut titles = std::collections::HashSet::new();

        for link in internal_links {
            match link.link_type {
                LinkType::Wiki => {
                    titles.insert(link.target);
                }
                LinkType::Markdown => {
                    // For markdown links, extract filename without extension
                    if !link.target.starts_with('/') {
                        if let Some(filename) = link.target.split('/').last() {
                            let name = filename.trim_end_matches(".md");
                            titles.insert(name.to_string());
                        }
                    }
                }
                _ => {}
            }
        }

        titles.into_iter().collect()
    }

    /// Check if content contains a link to a specific note
    pub fn has_link_to(markdown: &str, note_title: &str) -> bool {
        let titles = Self::get_referenced_note_titles(markdown);
        let normalized_target = note_title.to_lowercase();
        titles
            .iter()
            .any(|t| t.to_lowercase() == normalized_target)
    }
}

