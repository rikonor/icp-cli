//! ICP Distribution Library
//!
//! This library provides functionality for managing ICP CLI binary distribution,
//! including binary validation, URL management, and template rendering.

mod binary;
mod distribution;
mod error;
mod url;

pub use binary::{BinaryInfo, BinaryProcessor, ExtensionInfo};
pub use distribution::Distribution;
pub use error::{DistributionError, Result};
pub use url::UrlBuilder;

use handlebars::Handlebars;
use std::fs;
use std::path::Path;

/// Renders a template file using the provided data
pub fn render_template(
    name: &str,
    template_path: &Path,
    output_path: &Path,
    data: &impl serde::Serialize,
) -> Result<()> {
    let template_content = fs::read_to_string(template_path)?;
    let mut handlebars = Handlebars::new();

    // Configure Handlebars
    handlebars.set_strict_mode(true);
    handlebars.register_template_string(name, template_content)?;

    // Render template
    let rendered = handlebars
        .render(name, data)
        .map_err(|e| DistributionError::TemplateError(e))?;

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write rendered content
    fs::write(output_path, rendered)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use tempfile::TempDir;

    #[derive(Serialize)]
    struct TestData {
        name: String,
        value: i32,
    }

    #[test]
    fn test_render_template() {
        let temp_dir = TempDir::new().unwrap();

        // Create test template
        let template_path = temp_dir.path().join("test.tmpl");
        fs::write(&template_path, "Hello {{name}}! Value: {{value}}").unwrap();

        // Create test data
        let data = TestData {
            name: "Test".to_string(),
            value: 42,
        };

        // Render template
        let output_path = temp_dir.path().join("output.txt");
        render_template("test", &template_path, &output_path, &data).unwrap();

        // Verify output
        let output = fs::read_to_string(&output_path).unwrap();
        assert_eq!(output, "Hello Test! Value: 42");
    }

    #[test]
    fn test_render_template_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let data = TestData {
            name: "Test".to_string(),
            value: 42,
        };

        let result = render_template(
            "test",
            &temp_dir.path().join("nonexistent.tmpl"),
            &temp_dir.path().join("output.txt"),
            &data,
        );

        assert!(matches!(result, Err(DistributionError::IoError(_))));
    }
}
