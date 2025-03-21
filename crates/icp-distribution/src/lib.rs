//! ICP Distribution Crate
//!
//! This crate provides tools for generating distribution artifacts
//! for the ICP CLI, including installation scripts and package
//! manager configurations.

use handlebars::Handlebars;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Renders a template with provided values and writes it to the specified output path.
///
/// # Arguments
///
/// * `template_name` - A unique identifier for the template
/// * `template_path` - Path to the template file
/// * `output_path` - Path where the rendered output should be written
/// * `values` - HashMap containing template variables and their values
///
/// # Returns
///
/// * `Result` - Ok(()) on success, or an error
pub fn render_template(
    template_name: &str,
    template_path: &Path,
    output_path: &Path,
    values: HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read the template content
    let template_content = fs::read_to_string(template_path)?;

    // Initialize Handlebars and register the template
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string(template_name, template_content)?;

    // Render the template with provided values
    let rendered = handlebars.render(template_name, &values)?;

    // Ensure the output directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write the rendered content to the output path
    fs::write(output_path, rendered)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
        let path = dir.path().join(name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_render_template_success() {
        // Create a temporary directory for test files
        let temp_dir = TempDir::new().unwrap();

        // Create template file with verified content
        let template_path = create_file(
            &temp_dir,
            "template.txt",
            "Hello, {{name}}!\nProject: {{project}}",
        );
        let output_path = temp_dir.path().join("output.txt");

        // Setup template values
        let mut values = HashMap::new();
        values.insert("name".to_string(), "World".to_string());
        values.insert("project".to_string(), "ICP".to_string());

        // Render the template
        let result = render_template("test_template", &template_path, &output_path, values);
        assert!(
            result.is_ok(),
            "Template rendering failed: {:?}",
            result.err()
        );

        // Verify the output exists and has correct content
        assert!(output_path.exists(), "Output file was not created");
        let rendered_content = fs::read_to_string(&output_path).unwrap();
        assert_eq!(rendered_content, "Hello, World!\nProject: ICP");
    }

    #[test]
    fn test_render_template_nested_output() {
        let temp_dir = TempDir::new().unwrap();
        let template_path = create_file(&temp_dir, "template.txt", "Content: {{content}}");
        let output_path = temp_dir
            .path()
            .join("nested")
            .join("deep")
            .join("output.txt");

        let mut values = HashMap::new();
        values.insert("content".to_string(), "Nested File".to_string());

        let result = render_template("nested_template", &template_path, &output_path, values);
        assert!(
            result.is_ok(),
            "Failed to create nested output: {:?}",
            result.err()
        );
        assert!(output_path.exists(), "Nested output file was not created");
    }

    #[test]
    fn test_render_template_errors() {
        let temp_dir = TempDir::new().unwrap();

        // Test with invalid template syntax
        let invalid_template_path = create_file(&temp_dir, "invalid.txt", "Hello, {{name!}}"); // Invalid syntax
        let output_path = temp_dir.path().join("output.txt");

        let mut values = HashMap::new();
        values.insert("name".to_string(), "World".to_string());

        let result = render_template(
            "invalid_template",
            &invalid_template_path,
            &output_path,
            values,
        );
        assert!(
            result.is_err(),
            "Expected error for invalid template syntax"
        );

        // Test with non-existent template file
        let non_existent_path = temp_dir.path().join("non_existent.txt");
        let result = render_template(
            "missing_template",
            &non_existent_path,
            &output_path,
            HashMap::new(),
        );
        assert!(result.is_err(), "Expected error for missing template file");
    }
}
