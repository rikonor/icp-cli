# ICP CLI Quick Install

This directory contains installation scripts for quickly installing the ICP CLI across different platforms. The scripts provide a simple one-line installation process similar to other popular tools.

## Installation Methods

### Windows (PowerShell)

```powershell
Invoke-WebRequest -Uri https://get.icp-cli.com/install.ps1 -OutFile install.ps1; .\install.ps1
```

This command:

1. Downloads the PowerShell installation script
2. Executes the script to install ICP CLI
3. Requires administrator privileges
4. Installs to `C:\Program Files\icp`
5. Adds the installation directory to the system PATH

### macOS and Linux

```bash
curl -sL https://get.icp-cli.com/install.sh | sh
```

This command:

1. Downloads the shell installation script
2. Pipes it directly to the shell for execution
3. Does not require root privileges
4. Installs to `/usr/local/bin`
5. Updates PATH in both `.bashrc` and `.zshrc` if they exist

## Security Features

Both installation scripts implement several security measures:

1. **HTTPS Downloads**: All downloads (scripts and binaries) use HTTPS
2. **Checksum Verification**: Binary integrity is verified using SHA-256 checksums
3. **Privilege Checks**:
   - Windows script requires admin privileges
   - Unix script prevents running as root/sudo
4. **Permission Verification**: Scripts check for necessary write permissions
5. **Error Handling**: Comprehensive error checking and reporting

## Script Behavior

The installation scripts:

1. **Platform Detection**:

   - Windows script targets amd64 architecture
   - Unix script automatically detects platform (darwin/linux) and architecture

2. **Dependency Checking**:

   - Windows script verifies PowerShell version and admin rights
   - Unix script checks for required commands (curl, shasum, chmod)

3. **Installation Process**:

   - Downloads the appropriate binary for the platform
   - Verifies the binary's checksum
   - Places the binary in a standard location
   - Configures the system PATH
   - Sets appropriate permissions

4. **Error Handling**:
   - Provides clear error messages
   - Cleans up partial downloads on failure
   - Verifies each step before proceeding

## Domain Setup

The scripts expect the following infrastructure:

- Domain: `get.icp-cli.com`
- SSL: Required for all endpoints
- Endpoints:
  - `/install.ps1`: Windows installation script
  - `/install.sh`: Unix installation script
  - `/binaries/`: Directory containing platform-specific binaries
  - `/binaries/[platform-arch].sha256`: Checksum files for verification

## Development Notes

When updating these scripts:

1. Maintain consistent error handling across platforms
2. Keep security measures in place
3. Test on all supported platforms
4. Update binary URLs and checksums appropriately
5. Maintain backward compatibility where possible

## Troubleshooting

Common issues users might encounter:

1. **Windows**:

   - "Access Denied": Run PowerShell as Administrator
   - PATH not updated: Restart terminal after installation

2. **Unix**:
   - "Permission denied": Check write permissions to `/usr/local/bin`
   - Binary not found: Restart terminal or source shell configuration

## Future Improvements

Planned enhancements:

1. Add support for air-gapped installations
2. Implement GPG signature verification
3. Add offline installation options
4. Support for additional platforms and architectures
