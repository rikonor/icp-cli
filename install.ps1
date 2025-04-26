# ICP CLI Installation Script for Windows
# This script downloads and installs the ICP CLI binary for Windows systems.
# Version: 0.1.19

$ErrorActionPreference = "Stop"

# Configuration
$installDir = "$env:ProgramFiles\icp"
$binaryName = "icp.exe"
$binaryUrl = "https://rikonor.github.io/icp-cli/binaries/icp/windows-amd64.exe"
$checksumUrl = "https://rikonor.github.io/icp-cli/binaries/icp/windows-amd64.sha256"

# Helper Functions
function Write-Status {
    param($Message)
    Write-Host "==> $Message" -ForegroundColor Blue
}

function Write-Error {
    param($Message)
    Write-Host "ERROR: $Message" -ForegroundColor Red
    exit 1
}

# Main Installation Process
try {
    Write-Status "Starting ICP CLI 0.1.19 installation..."

    # Check if running with admin privileges
    $isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if (-not $isAdmin) {
        Write-Error "This script requires administrator privileges. Please run PowerShell as Administrator."
    }

    # Create installation directory
    Write-Status "Creating installation directory at $installDir"
    if (!(Test-Path $installDir)) {
        New-Item -ItemType Directory -Force -Path $installDir | Out-Null
    }

    # Download binary
    Write-Status "Downloading ICP CLI binary from $binaryUrl"
    try {
        Invoke-WebRequest -Uri $binaryUrl -OutFile "$installDir\$binaryName" -UseBasicParsing
    } catch {
        Write-Error "Failed to download binary: $_"
    }

    # Verify checksum
    Write-Status "Verifying binary checksum"
    try {
        $expectedChecksum = (Invoke-WebRequest -Uri $checksumUrl -UseBasicParsing).Content.Trim()
        $actualChecksum = Get-FileHash -Algorithm SHA256 -Path "$installDir\$binaryName" | Select-Object -ExpandProperty Hash
        if ($actualChecksum -ne $expectedChecksum) {
            Remove-Item -Path "$installDir\$binaryName" -Force
            Write-Error "Checksum verification failed! Expected: $expectedChecksum, Got: $actualChecksum"
        }
    } catch {
        Write-Error "Failed to verify checksum: $_"
    }

    # Add to PATH
    Write-Status "Adding ICP CLI to system PATH"
    try {
        $envPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
        if (!$envPath.Contains($installDir)) {
            [Environment]::SetEnvironmentVariable("Path", "$envPath;$installDir", "Machine")
        }
    } catch {
        Write-Error "Failed to update PATH: $_"
    }

    Write-Status "Installation complete! You may need to restart your terminal for PATH changes to take effect."
    Write-Status "Try running 'icp --version' to verify the installation."

} catch {
    Write-Error "An unexpected error occurred: $_"
}
