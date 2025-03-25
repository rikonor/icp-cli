use clap::Parser;
use icp_distribution::{BinaryAsset, ExtensionAsset, HomebrewFormulaContext, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(about = "Generate Homebrew formula from template")]
struct Args {
    /// Version number
    #[arg(long)]
    version: String,

    /// Path to checksums file
    #[arg(long)]
    checksums: PathBuf,

    /// Path to extensions JSON file
    #[arg(long)]
    extensions: PathBuf,

    /// URL for Intel binary
    #[arg(long)]
    intel_url: String,

    /// URL for ARM binary
    #[arg(long)]
    arm_url: String,

    /// Output path for generated formula
    #[arg(long)]
    output: PathBuf,
}

fn parse_checksums(path: &Path) -> Result<HashMap<String, String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut checksums = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        if let Some((hash, filename)) = line.split_once("  ") {
            checksums.insert(filename.to_string(), hash.to_string());
        }
    }

    Ok(checksums)
}

fn parse_extensions(
    path: &Path,
    checksums: &HashMap<String, String>,
) -> Result<Vec<ExtensionAsset>> {
    let content = fs::read_to_string(path)?;
    let extensions: Vec<Value> = serde_json::from_str(&content)?;

    extensions
        .iter()
        .map(|ext| {
            let name = ext["name"]
                .as_str()
                .ok_or_else(|| {
                    icp_distribution::DistributionError::TemplateError(
                        handlebars::RenderError::new("Missing extension name"),
                    )
                })?
                .to_string();

            let url = ext["url"]
                .as_str()
                .ok_or_else(|| {
                    icp_distribution::DistributionError::TemplateError(
                        handlebars::RenderError::new("Missing extension URL"),
                    )
                })?
                .to_string();

            // Get SHA256 from checksums using the extension filename
            let filename = format!("{}.component.wasm", name);
            let sha256 = checksums.get(&filename).ok_or_else(|| {
                icp_distribution::DistributionError::TemplateError(handlebars::RenderError::new(
                    &format!("Missing checksum for extension: {}", filename),
                ))
            })?;

            Ok(ExtensionAsset {
                name,
                url,
                sha256: sha256.clone(),
            })
        })
        .collect()
}

fn get_binary_sha256(checksums: &HashMap<String, String>, filename: &str) -> Result<String> {
    checksums
        .get(filename)
        .ok_or_else(|| {
            icp_distribution::DistributionError::TemplateError(handlebars::RenderError::new(
                &format!("Missing checksum for binary: {}", filename),
            ))
        })
        .map(String::from)
}

fn extract_filename(url: &str) -> Result<&str> {
    url.split('/').last().ok_or_else(|| {
        icp_distribution::DistributionError::TemplateError(handlebars::RenderError::new(
            "Invalid binary URL",
        ))
    })
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Parse checksums file
    let checksums = parse_checksums(&args.checksums)?;

    // Extract filenames first
    let intel_filename = extract_filename(&args.intel_url)?;
    let arm_filename = extract_filename(&args.arm_url)?;

    // Get checksums using filenames
    let intel_sha256 = get_binary_sha256(&checksums, intel_filename)?;
    let arm_sha256 = get_binary_sha256(&checksums, arm_filename)?;

    // Create binary assets
    let intel_binary = BinaryAsset {
        url: args.intel_url,
        sha256: intel_sha256,
    };

    let arm_binary = BinaryAsset {
        url: args.arm_url,
        sha256: arm_sha256,
    };

    // Parse extensions
    let extensions = parse_extensions(&args.extensions, &checksums)?;

    // Create formula context
    let formula_context =
        HomebrewFormulaContext::new(args.version, intel_binary, arm_binary, extensions);

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
