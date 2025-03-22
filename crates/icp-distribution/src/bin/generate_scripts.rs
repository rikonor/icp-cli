//! Script Generator Tool
//!
//! A CLI tool to generate installation scripts from templates.

use clap::Parser;
use icp_distribution::{
    render_template, BinaryInfo, BinaryProcessor, DistributionError, ExtensionInfo, Result,
    UrlBuilder,
};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Path to the binaries directory
    #[arg(long, default_value = "dist/binaries/icp")]
    binary_path: PathBuf,

    /// Path to the extensions directory
    #[arg(long, default_value = "dist/binaries/extensions")]
    extensions_path: PathBuf,

    /// Output directory for generated files
    #[arg(long, default_value = "dist")]
    output_dir: PathBuf,

    /// Domain for URLs (e.g., rikonor.github.io/icp-cli)
    #[arg(long, env = "ICP_DISTRIBUTION_DOMAIN")]
    domain: Option<String>,

    /// GitHub repository URL (e.g., https://github.com/rikonor/icp-cli)
    #[arg(long, default_value = "https://github.com/rikonor/icp-cli")]
    repo_url: String,
}

#[derive(Serialize)]
struct TemplateData {
    github_pages_url: String,
    github_repo_url: String,
    binaries: Vec<BinaryInfo>,
    extensions: Vec<ExtensionInfo>,
}

fn run() -> Result<()> {
    let args = Args::parse();

    // Create output directory if it doesn't exist
    fs::create_dir_all(&args.output_dir)?;

    // Setup template values
    let version = env!("CARGO_PKG_VERSION", "0.1.0");

    // Get domain from args or use default GitHub Pages URL
    let domain = args
        .domain
        .unwrap_or_else(|| "rikonor.github.io/icp-cli".to_string());

    // Create .nojekyll file to prevent Jekyll processing
    let nojekyll_path = args.output_dir.join(".nojekyll");
    std::fs::write(&nojekyll_path, "")?;
    println!("Created .nojekyll file: {:?}", nojekyll_path);

    // Process binaries and extensions
    println!("Validating binaries in: {:?}", args.binary_path);
    let processor =
        BinaryProcessor::new(args.binary_path)?.with_extensions_path(args.extensions_path)?;
    let binaries = processor.parse_binary_info()?;
    println!("Found {} valid binaries", binaries.len());

    // Setup URL builder
    let url_builder = UrlBuilder::new(&domain, &args.repo_url);
    let binary_url_base = url_builder.binary_url()?;
    let checksum_url_base = url_builder.checksum_url()?;

    // Ensure binary_url_base and checksum_url_base are used for scripts

    // Generate landing page
    // Get extensions
    let extensions = processor.parse_extensions()?;
    println!("Found {} extensions", extensions.len());

    let template_data = TemplateData {
        github_pages_url: url_builder.pages_url()?,
        github_repo_url: url_builder.repo_url()?,
        binaries,
        extensions,
    };

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| DistributionError::UrlError("Failed to get CARGO_MANIFEST_DIR".into()))?;
    let template_dir = PathBuf::from(&manifest_dir).join("templates/curl-install");

    let landing_template_path = PathBuf::from(&manifest_dir).join("templates/index.html.tmpl");
    let landing_output_path = args.output_dir.join("index.html");

    println!("Generating landing page: {:?}", landing_output_path);
    render_template(
        "index.html",
        &landing_template_path,
        &landing_output_path,
        &template_data,
    )?;

    // Generate Unix script
    let mut unix_values = HashMap::new();
    unix_values.insert("version".to_string(), version.to_string());
    unix_values.insert("install_dir".to_string(), "/usr/local/bin".to_string());
    unix_values.insert("binary_name".to_string(), "icp".to_string());
    unix_values.insert("binary_url_base".to_string(), binary_url_base.clone());
    unix_values.insert("checksum_url_base".to_string(), checksum_url_base.clone());

    let unix_template_path = template_dir.join("install.sh.tmpl");
    let unix_output_path = args.output_dir.join("install.sh");

    println!(
        "Generating Unix installation script: {:?}",
        unix_output_path
    );
    render_template(
        "install.sh",
        &unix_template_path,
        &unix_output_path,
        &unix_values,
    )?;

    // Make the Unix script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&unix_output_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&unix_output_path, perms)?;
    }

    // Generate Windows script
    let mut windows_values = HashMap::new();
    windows_values.insert("version".to_string(), version.to_string());
    windows_values.insert(
        "install_dir".to_string(),
        "$env:ProgramFiles\\icp".to_string(),
    );
    windows_values.insert("binary_name".to_string(), "icp.exe".to_string());
    windows_values.insert("binary_url_base".to_string(), binary_url_base);
    windows_values.insert("checksum_url_base".to_string(), checksum_url_base);

    let windows_template_path = template_dir.join("install.ps1.tmpl");
    let windows_output_path = args.output_dir.join("install.ps1");

    println!(
        "Generating Windows installation script: {:?}",
        windows_output_path
    );
    render_template(
        "install.ps1",
        &windows_template_path,
        &windows_output_path,
        &windows_values,
    )?;

    // Validate generated files
    for &path in &[
        &unix_output_path,
        &windows_output_path,
        &landing_output_path,
    ] {
        if !path.exists() {
            return Err(DistributionError::MissingFile(path.to_owned()));
        }
        println!("Validated file exists: {:?}", path);
    }

    println!("Installation scripts and landing page successfully generated!");
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}
