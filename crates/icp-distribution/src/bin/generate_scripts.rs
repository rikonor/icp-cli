//! Script Generator Tool
//!
//! A CLI tool to generate installation scripts from templates.

use icp_distribution::render_template;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Get the crate root directory
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("Failed to get CARGO_MANIFEST_DIR environment variable");

    // Define paths relative to the crate root
    let output_dir = PathBuf::from(&manifest_dir).join("dist");
    let template_dir = PathBuf::from(&manifest_dir).join("templates/curl-install");

    // Create output directory if it doesn't exist
    fs::create_dir_all(&output_dir)?;

    // Setup template values
    let version = env!("CARGO_PKG_VERSION", "0.1.0");

    // Get domain from environment or use default GitHub Pages URL
    let domain = std::env::var("ICP_DISTRIBUTION_DOMAIN")
        .unwrap_or_else(|_| "rikonor.github.io/icp-cli".to_string());

    // Create .nojekyll file to prevent Jekyll processing
    let nojekyll_path = output_dir.join(".nojekyll");
    std::fs::write(&nojekyll_path, "")?;
    println!("Created .nojekyll file: {:?}", nojekyll_path);

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
    let unix_output_path = output_dir.join("install.sh");

    println!(
        "Generating Unix installation script: {:?}",
        unix_output_path
    );
    render_template(
        "install.sh",
        &unix_template_path,
        &unix_output_path,
        unix_values,
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
    let windows_output_path = output_dir.join("install.ps1");

    println!(
        "Generating Windows installation script: {:?}",
        windows_output_path
    );
    render_template(
        "install.ps1",
        &windows_template_path,
        &windows_output_path,
        windows_values,
    )?;

    // Validate generated files
    for path in &[&unix_output_path, &windows_output_path] {
        if !path.exists() {
            return Err(format!("Failed to generate script: {:?}", path).into());
        }
        println!("Validated file exists: {:?}", path);
    }

    println!("Installation scripts successfully generated!");
    Ok(())
}
