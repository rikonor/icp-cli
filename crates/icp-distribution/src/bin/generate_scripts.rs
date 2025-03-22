//! Script Generator Tool
//!
//! A CLI tool to generate installation scripts from templates.

use clap::Parser;
use icp_distribution::render_template;
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

    /// Output directory for generated files
    #[arg(long, default_value = "dist")]
    output_dir: PathBuf,

    /// Domain for URLs (e.g., rikonor.github.io/icp-cli)
    #[arg(long, env = "ICP_DISTRIBUTION_DOMAIN")]
    domain: Option<String>,
}

#[derive(Serialize)]
struct BinaryInfo {
    name: String,
    target: String,
    variant: String,
    checksum: String,
}

#[derive(Serialize)]
struct TemplateData {
    github_pages_url: String,
    github_repo_url: String,
    binaries: Vec<BinaryInfo>,
}

fn parse_binary_info(binary_path: &PathBuf) -> Result<Vec<BinaryInfo>, Box<dyn std::error::Error>> {
    let mut binaries = Vec::new();

    // Read checksums file
    let checksums = fs::read_to_string(binary_path.join("checksums.txt"))?;
    let checksums_map: HashMap<String, String> = checksums
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                Some((parts[1].to_string(), parts[0].to_string()))
            } else {
                None
            }
        })
        .collect();

    // Process each binary file
    for entry in fs::read_dir(binary_path)? {
        let entry = entry?;
        let filename = entry.file_name().to_string_lossy().to_string();
        if filename == "checksums.txt" {
            continue;
        }

        // Parse filename like: icp-x86_64-apple-darwin-standard
        let parts: Vec<&str> = filename.split('-').collect();
        if parts.len() >= 4 {
            binaries.push(BinaryInfo {
                name: filename.clone(),
                target: parts[1..parts.len() - 1].join("-"),
                variant: parts.last().unwrap().to_string(),
                checksum: checksums_map.get(&filename).cloned().unwrap_or_default(),
            });
        }
    }

    Ok(binaries)
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
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

    // Parse binary information
    let binaries = parse_binary_info(&args.binary_path)?;

    // Generate landing page
    let template_data = TemplateData {
        github_pages_url: format!("https://{}", domain),
        github_repo_url: "https://github.com/rikonor/icp-cli".to_string(),
        binaries,
    };

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("Failed to get CARGO_MANIFEST_DIR environment variable");
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

    // Use HTTPS for all URLs
    let base_url = format!("https://{}/binaries/icp", domain);
    unix_values.insert("binary_url_base".to_string(), base_url.clone());
    unix_values.insert("checksum_url_base".to_string(), base_url.clone());

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
    windows_values.insert(
        "binary_url_base".to_string(),
        format!("https://{}/binaries/icp", domain),
    );
    windows_values.insert(
        "checksum_url_base".to_string(),
        format!("https://{}/binaries/icp", domain),
    );

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
    for path in &[
        &unix_output_path,
        &windows_output_path,
        &landing_output_path,
    ] {
        if !path.exists() {
            return Err(format!("Failed to generate file: {:?}", path).into());
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
