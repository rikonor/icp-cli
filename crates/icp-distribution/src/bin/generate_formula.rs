use clap::Parser;
use icp_distribution::{BinaryAsset, ExtensionAsset, HomebrewFormulaContext, Result};
use serde::Deserialize; // Added for direct deserialization
use std::env;
use std::path::PathBuf;

// Define a struct that matches the JSON structure from fetch-latest-extensions
#[derive(Deserialize, Debug)]
struct ExtensionInfoInput {
    name: String,
    // version: String, // Version not currently needed by ExtensionAsset
    url: String,
    sha256: String,
}

#[derive(Parser)]
#[command(about = "Generate Homebrew formula from template")]
struct Args {
    /// Version number
    #[arg(long)]
    version: String,

    /// JSON string containing extension info (name, version, url, sha256)
    #[arg(long)]
    extension_info_json: String,

    /// URL for Intel binary
    #[arg(long)]
    intel_url: String,

    /// SHA256 checksum for Intel binary
    #[arg(long)]
    intel_sha256: String,

    /// URL for ARM binary
    #[arg(long)]
    arm_url: String,

    /// SHA256 checksum for ARM binary
    #[arg(long)]
    arm_sha256: String,

    /// Output path for generated formula
    #[arg(long)]
    output: PathBuf,
}

// Removed parse_checksums function as checksums are now direct inputs

// Updated function to parse extensions directly from JSON string
fn parse_extensions_from_json(json_string: &str) -> Result<Vec<ExtensionAsset>> {
    let inputs: Vec<ExtensionInfoInput> = serde_json::from_str(json_string)
        .map_err(|e| icp_distribution::DistributionError::JsonError(e))?; // Handle JSON parsing error

    Ok(inputs
        .into_iter()
        .map(|input| ExtensionAsset {
            name: input.name,
            url: input.url,
            sha256: input.sha256,
        })
        .collect())
}

// Removed get_binary_sha256 function
// Removed extract_filename function (no longer needed as checksums are direct inputs)

fn main() -> Result<()> {
    let args = Args::parse();

    // Checksums are now direct arguments (args.intel_sha256, args.arm_sha256)

    // Create binary assets directly using args
    let intel_binary = BinaryAsset {
        url: args.intel_url,
        sha256: args.intel_sha256, // Use directly from args
    };

    let arm_binary = BinaryAsset {
        url: args.arm_url,
        sha256: args.arm_sha256, // Use directly from args
    };

    // Parse extensions from the JSON input string
    let extensions = parse_extensions_from_json(&args.extension_info_json)?;

    // Create formula context
    let formula_context =
        HomebrewFormulaContext::new(args.version, intel_binary, arm_binary, extensions);

    // Get template path relative to crate root
    // Consider making this path resolution more robust if needed
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
