//! Build script for icp-distribution crate
//!
//! This script runs during the build process to generate installation scripts.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

// Handlebars is a dependency for icp-distribution crate
extern crate handlebars;
use handlebars::Handlebars;

fn main() {
    // Print cargo directives
    println!("cargo:rerun-if-changed=templates/");

    // Only run script generation in release builds to avoid unnecessary generation during development
    if env::var("PROFILE").unwrap_or_else(|_| "dev".to_string()) == "release" {
        println!("cargo:warning=Generating installation scripts");

        // Get paths
        let manifest_dir =
            env::var("CARGO_MANIFEST_DIR").expect("Could not determine manifest directory");

        // Generate the scripts
        match generate_scripts(&manifest_dir) {
            Ok(_) => println!("cargo:warning=Installation scripts successfully generated"),
            Err(e) => panic!("Failed to generate installation scripts: {}", e),
        }
    }
}

fn generate_scripts(manifest_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Define paths relative to the crate root
    let base_dir = PathBuf::from(manifest_dir);
    let output_dir = base_dir.join("dist");
    let template_dir = base_dir.join("templates/curl-install");

    // Create output directory if it doesn't exist
    fs::create_dir_all(&output_dir)?;

    // Setup template values
    let version = env!("CARGO_PKG_VERSION", "0.1.0");
    let domain = "get.icp-cli.com";

    // Initialize Handlebars
    let mut handlebars = Handlebars::new();

    // Generate Unix script
    let mut unix_values = HashMap::new();
    unix_values.insert("version".to_string(), version.to_string());
    unix_values.insert("install_dir".to_string(), "/usr/local/bin".to_string());
    unix_values.insert("binary_name".to_string(), "icp".to_string());
    unix_values.insert(
        "binary_url_base".to_string(),
        format!("https://{}/binaries/icp", domain),
    );
    unix_values.insert(
        "checksum_url_base".to_string(),
        format!("https://{}/binaries/icp", domain),
    );

    let unix_template_path = template_dir.join("install.sh.tmpl");
    let unix_output_path = output_dir.join("install.sh");

    let unix_template = fs::read_to_string(&unix_template_path)?;
    handlebars.register_template_string("install.sh", unix_template)?;
    let unix_rendered = handlebars.render("install.sh", &unix_values)?;

    // Write the Unix script
    let mut unix_file = fs::File::create(&unix_output_path)?;
    unix_file.write_all(unix_rendered.as_bytes())?;

    // Make the Unix script executable on Unix platforms
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

    let windows_template = fs::read_to_string(&windows_template_path)?;
    handlebars.register_template_string("install.ps1", windows_template)?;
    let windows_rendered = handlebars.render("install.ps1", &windows_values)?;

    // Write the Windows script
    let mut windows_file = fs::File::create(&windows_output_path)?;
    windows_file.write_all(windows_rendered.as_bytes())?;

    Ok(())
}
