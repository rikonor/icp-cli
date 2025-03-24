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

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HomebrewFormulaContext {
    pub version: String,
    pub intel_binary: BinaryAsset,
    pub arm_binary: BinaryAsset,
    pub extensions: Vec<ExtensionAsset>,
}

#[derive(Debug, Serialize)]
pub struct BinaryAsset {
    pub url: String,
    pub sha256: String,
}

#[derive(Debug, Serialize)]
pub struct ExtensionAsset {
    pub name: String,
    pub url: String,
    pub sha256: String,
}

impl HomebrewFormulaContext {
    pub fn new(
        version: String,
        intel_binary: BinaryAsset,
        arm_binary: BinaryAsset,
        extensions: Vec<ExtensionAsset>,
    ) -> Self {
        Self {
            version,
            intel_binary,
            arm_binary,
            extensions,
        }
    }
}

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

    #[test]
    fn test_homebrew_formula_generation() {
        let temp_dir = TempDir::new().unwrap();

        // Create test template
        let template_path = temp_dir.path().join("formula.rb.tmpl");
        fs::write(
            &template_path,
            r#"class IcpCli < Formula
  version "{{version}}"

  if Hardware::CPU.intel?
    url "{{intel_binary.url}}"
    sha256 "{{intel_binary.sha256}}"
  else
    url "{{arm_binary.url}}"
    sha256 "{{arm_binary.sha256}}"
  end

  {{#each extensions}}
  resource "{{name}}" do
    url "{{url}}"
    sha256 "{{sha256}}"
  end
  {{/each}}
end"#,
        )
        .unwrap();

        // Create test data
        let context = HomebrewFormulaContext::new(
            "1.0.0".to_string(),
            BinaryAsset {
                url: "https://example.com/intel".to_string(),
                sha256: "intel_hash".to_string(),
            },
            BinaryAsset {
                url: "https://example.com/arm".to_string(),
                sha256: "arm_hash".to_string(),
            },
            vec![ExtensionAsset {
                name: "multiply".to_string(),
                url: "https://example.com/multiply.wasm".to_string(),
                sha256: "multiply_hash".to_string(),
            }],
        );

        // Render formula
        let output_path = temp_dir.path().join("icp-cli.rb");
        render_template("homebrew", &template_path, &output_path, &context).unwrap();

        // Verify output
        let output = fs::read_to_string(&output_path).unwrap();
        assert!(output.contains(r#"version "1.0.0""#));
        assert!(output.contains(r#"url "https://example.com/intel""#));
        assert!(output.contains(r#"sha256 "intel_hash""#));
        assert!(output.contains(r#"url "https://example.com/arm""#));
        assert!(output.contains(r#"sha256 "arm_hash""#));
        assert!(output.contains(r#"resource "multiply""#));
        assert!(output.contains(r#"url "https://example.com/multiply.wasm""#));
        assert!(output.contains(r#"sha256 "multiply_hash""#));
    }
}
