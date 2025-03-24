use clap::Parser;
use icp_distribution::{BinaryAsset, ExtensionAsset, HomebrewFormulaContext, Result};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(about = "Generate Homebrew formula from template")]
struct Args {
    /// Path to JSON context file
    #[arg(long)]
    context: PathBuf,

    /// Output path for generated formula
    #[arg(long)]
    output: PathBuf,
}

fn get_str_value(value: &Value, path: &[&str]) -> Result<String> {
    let mut current = value;
    for &key in path {
        current = current.get(key).ok_or_else(|| {
            icp_distribution::DistributionError::TemplateError(handlebars::RenderError::new(
                format!("Missing key: {}", key),
            ))
        })?;
    }
    current.as_str().map(String::from).ok_or_else(|| {
        icp_distribution::DistributionError::TemplateError(handlebars::RenderError::new(format!(
            "Invalid string value at path: {}",
            path.join(".")
        )))
    })
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Read and parse context
    let context_str = fs::read_to_string(&args.context)?;
    let context: Value = serde_json::from_str(&context_str)?;

    // Extract data from context
    let version = get_str_value(&context, &["version"])?;

    let intel_binary = BinaryAsset {
        url: get_str_value(&context, &["intel_binary", "url"])?,
        sha256: get_str_value(&context, &["intel_binary", "sha256"])?,
    };

    let arm_binary = BinaryAsset {
        url: get_str_value(&context, &["arm_binary", "url"])?,
        sha256: get_str_value(&context, &["arm_binary", "sha256"])?,
    };

    let extensions = context["extensions"]
        .as_array()
        .ok_or_else(|| {
            icp_distribution::DistributionError::TemplateError(handlebars::RenderError::new(
                "Missing extensions array",
            ))
        })?
        .iter()
        .map(|ext| {
            Ok(ExtensionAsset {
                name: get_str_value(ext, &["name"])?,
                url: get_str_value(ext, &["url"])?,
                sha256: get_str_value(ext, &["sha256"])?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // Create formula context
    let formula_context =
        HomebrewFormulaContext::new(version, intel_binary, arm_binary, extensions);

    // Get template path relative to crate root
    let template_path = env::current_dir()?
        .join("crates")
        .join("icp-distribution")
        .join("templates")
        .join("homebrew")
        .join("formula.rb.tmpl");

    // Render formula
    icp_distribution::render_template("homebrew", &template_path, &args.output, &formula_context)?;

    Ok(())
}
