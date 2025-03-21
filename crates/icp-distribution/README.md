# ICP Distribution

This crate provides tools for generating distribution artifacts for the ICP CLI, including installation scripts and package manager configurations.

## Features

- Template-based generation of installation scripts
- Cross-platform support (Windows, macOS, Linux)
- Configuration for various distribution channels

## Usage

### Basic Template Rendering

```rust
use icp_distribution::render_template;
use std::collections::HashMap;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup template values
    let mut values = HashMap::new();
    values.insert("version".to_string(), "0.1.0".to_string());
    values.insert("install_dir".to_string(), "/usr/local/bin".to_string());
    values.insert("binary_name".to_string(), "icp".to_string());
    values.insert("binary_url_base".to_string(), "https://get.icp-cli.com/binaries/icp".to_string());
    values.insert("checksum_url_base".to_string(), "https://get.icp-cli.com/binaries/icp".to_string());

    // Render Unix installation script
    render_template(
        "install.sh",
        Path::new("templates/curl-install/install.sh.tmpl"),
        Path::new("dist/install.sh"),
        values.clone(),
    )?;

    // Update Windows-specific values
    values.insert("install_dir".to_string(), "C:\\Program Files\\icp".to_string());
    values.insert("binary_name".to_string(), "icp.exe".to_string());

    // Render Windows installation script
    render_template(
        "install.ps1",
        Path::new("templates/curl-install/install.ps1.tmpl"),
        Path::new("dist/install.ps1"),
        values,
    )?;

    Ok(())
}
```

## Templates

The crate includes templates for:

- Curl-based installation (Unix shell script)
- PowerShell installation (Windows)

### Template Variables

Templates use Handlebars syntax and support the following variables:

- `{{version}}` - ICP CLI version
- `{{install_dir}}` - Installation directory
- `{{binary_name}}` - Binary filename
- `{{binary_url_base}}` - Base URL for binary downloads
- `{{checksum_url_base}}` - Base URL for checksum files

## Development

To add a new template:

1. Create the template file in `templates/` directory
2. Use Handlebars syntax for variable substitution
3. Register the template in your application code

## Integration with GitHub Pages

The generated scripts are intended to be hosted on GitHub Pages with a custom domain (get.icp-cli.com).

## Security Considerations

The installation scripts implement several security measures:

1. HTTPS Downloads
2. Checksum Verification
3. Privilege Checks
4. Permission Verification
5. Comprehensive Error Handling
