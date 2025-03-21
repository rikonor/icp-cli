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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_render_template() {
        // Create a temporary template file
        let mut template_file = NamedTempFile::new().unwrap();
        writeln!(template_file, "Hello, {{name}}!").unwrap();
        let template_path = template_file.path();

        // Create a temporary output file
        let output_file = NamedTempFile::new().unwrap();
        let output_path = output_file.path();

        // Setup template values
        let mut values = HashMap::new();
        values.insert("name".to_string(), "World".to_string());

        // Render the template
        let result = render_template("test_template", template_path, output_path, values);

        // Verify the result
        assert!(result.is_ok());
        let rendered_content = fs::read_to_string(output_path).unwrap();
        assert_eq!(rendered_content, "Hello, World!\n");
    }
}
