//! Export Service Implementation
//!
//! Stub implementation of the ExportService port.
//! Full implementation would use headless Chrome or chromiumoxide for PDF generation.

use async_trait::async_trait;

use crate::domain::{
    errors::{DomainError, DomainResult},
    ports::outbound::{ExportService, HtmlOptions, HtmlTheme, PdfOptions},
};

/// Stub export service implementation
/// TODO: Implement PDF export using headless Chrome
pub struct StubExportService;

impl StubExportService {
    pub fn new() -> Self {
        Self
    }

    /// Generate basic HTML template
    fn generate_html_template(content: &str, options: &HtmlOptions) -> String {
        let title = options
            .title
            .as_ref()
            .map(|t| t.as_str())
            .unwrap_or("Exported Note");

        let theme = options.theme.unwrap_or(HtmlTheme::Light);
        let background = match theme {
            HtmlTheme::Light => "#ffffff",
            HtmlTheme::Dark => "#1e1e1e",
        };
        let text_color = match theme {
            HtmlTheme::Light => "#000000",
            HtmlTheme::Dark => "#ffffff",
        };

        let default_styles = if options.include_styles.unwrap_or(true) {
            format!(
                r#"
        <style>
            body {{
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
                max-width: 800px;
                margin: 0 auto;
                padding: 40px 20px;
                line-height: 1.6;
                background-color: {};
                color: {};
            }}
            h1, h2, h3, h4, h5, h6 {{
                margin-top: 24px;
                margin-bottom: 16px;
                font-weight: 600;
                line-height: 1.25;
            }}
            code {{
                background-color: rgba(127, 127, 127, 0.1);
                padding: 2px 6px;
                border-radius: 3px;
                font-family: 'Courier New', monospace;
            }}
            pre {{
                background-color: rgba(127, 127, 127, 0.1);
                padding: 16px;
                border-radius: 6px;
                overflow-x: auto;
            }}
            blockquote {{
                border-left: 4px solid #ccc;
                padding-left: 16px;
                margin-left: 0;
                color: #666;
            }}
            img {{
                max-width: 100%;
                height: auto;
            }}
        </style>
        "#,
                background, text_color
            )
        } else {
            String::new()
        };

        let custom_styles = options
            .custom_css
            .as_ref()
            .map(|css| format!("<style>{}</style>", css))
            .unwrap_or_default();

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    {}
    {}
</head>
<body>
    {}
</body>
</html>"#,
            title, default_styles, custom_styles, content
        )
    }
}

impl Default for StubExportService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ExportService for StubExportService {
    async fn render_to_pdf(&self, _html: &str, _options: Option<PdfOptions>) -> DomainResult<Vec<u8>> {
        // TODO: Implement PDF rendering using headless Chrome
        // For now, return an error indicating it's not implemented
        Err(DomainError::ExternalServiceError(
            "PDF export not yet implemented. Requires headless Chrome integration.".to_string(),
        ))
    }

    fn generate_html_document(&self, content: &str, options: Option<HtmlOptions>) -> DomainResult<String> {
        let opts = options.unwrap_or_default();
        Ok(Self::generate_html_template(content, &opts))
    }

    fn is_pdf_available(&self) -> bool {
        // TODO: Check if headless Chrome is available
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_html_document() {
        let service = StubExportService::new();
        let content = "<h1>Test Note</h1><p>This is test content.</p>";

        let options = HtmlOptions {
            title: Some("Test Title".to_string()),
            theme: Some(HtmlTheme::Light),
            include_styles: Some(true),
            custom_css: None,
        };

        let html = service.generate_html_document(content, Some(options)).unwrap();

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Test Title"));
        assert!(html.contains("<h1>Test Note</h1>"));
        assert!(html.contains("font-family"));
    }

    #[test]
    fn test_generate_html_dark_theme() {
        let service = StubExportService::new();
        let content = "<p>Dark theme content</p>";

        let options = HtmlOptions {
            title: None,
            theme: Some(HtmlTheme::Dark),
            include_styles: Some(true),
            custom_css: None,
        };

        let html = service.generate_html_document(content, Some(options)).unwrap();

        assert!(html.contains("#1e1e1e")); // Dark background
        assert!(html.contains("#ffffff")); // Light text
    }

    #[test]
    fn test_is_pdf_available() {
        let service = StubExportService::new();
        assert!(!service.is_pdf_available()); // Not implemented yet
    }

    #[tokio::test]
    async fn test_render_to_pdf_not_implemented() {
        let service = StubExportService::new();
        let result = service.render_to_pdf("<html></html>", None).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not yet implemented"));
    }
}
