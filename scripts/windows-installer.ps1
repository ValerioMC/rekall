<#
.SYNOPSIS
    Builds the Windows installer for Rekall, in one command, from a fresh checkout.

.DESCRIPTION
    Windows counterpart of `make dmg`. It checks the toolchain, builds the console bundle
    (rekall-ui/dist, which the server embeds), and has the Tauri CLI build the desktop app and
    wrap it in an NSIS setup program:

        target\release\bundle\nsis\Rekall_<version>_x64-setup.exe

    The setup installs for the current user only, so running it needs no administrator rights.
    It needs the Rust MSVC toolchain, the Visual C++ Build Tools, Node 22 and pnpm. The Tauri CLI
    and icons\icon.ico are fetched or generated on the first run. The Windows-only bundle
    settings are in rekall-app\desktop\tauri.windows.conf.json.

.PARAMETER Version
    The version stamped into the installer and compiled in as REKALL_VERSION (1.2.3, no leading
    "v"), as the release workflow does for a tag. Without it the version in tauri.conf.json is used.

.PARAMETER SkipFrontend
    Reuse rekall-ui\dist as it is instead of rebuilding it.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts\windows-installer.ps1

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts\windows-installer.ps1 -Version 0.1.1
#>
[CmdletBinding()]
param(
    [ValidatePattern('^\d+\.\d+\.\d+$')]
    [string]$Version,
    [switch]$SkipFrontend
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$Root = Split-Path -Parent $PSScriptRoot
$Ui = Join-Path $Root 'rekall-ui'
$Desktop = Join-Path $Root 'rekall-app\desktop'
$Icon = Join-Path $Desktop 'icons\icon.ico'
$IconMaster = Join-Path $Root 'scripts\icon-master.png'
$BundleFolder = Join-Path $Root 'target\release\bundle\nsis'

function Write-Step([string]$Message) {
    Write-Host "==> $Message" -ForegroundColor Cyan
}

# A native command reports failure through its exit code, which PowerShell does not turn into an error.
function Invoke-Checked([string]$What, [scriptblock]$Command) {
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$What failed (exit code $LASTEXITCODE)."
    }
}

# A terminal opened before the tools were installed does not have them on its PATH yet.
function Add-ToPath([string]$Folder) {
    if ((Test-Path $Folder) -and (($env:Path -split ';') -notcontains $Folder)) {
        $env:Path = "$env:Path;$Folder"
    }
}

function Assert-Command([string]$Name, [string]$Hint) {
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "$Name was not found. $Hint"
    }
}

function Assert-Toolchain {
    Add-ToPath (Join-Path $env:USERPROFILE '.cargo\bin')
    Add-ToPath (Join-Path $env:LOCALAPPDATA 'Programs\nodejs')

    Assert-Command 'cargo' 'Install Rust: winget install Rustlang.Rustup'
    Assert-Command 'node' 'Install Node 22: winget install OpenJS.NodeJS.LTS'
    Assert-Command 'pnpm' 'Install pnpm: npm install -g pnpm@9.15.4'

    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    $tools = $null
    if (Test-Path $vswhere) {
        $tools = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    }
    if (-not $tools) {
        throw 'The Visual C++ Build Tools are missing, and Rust cannot link without them. Install: winget install Microsoft.VisualStudio.2022.BuildTools --override "--passive --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"'
    }
}

function Install-TauriCli {
    $installed = (cargo install --list) -match '^tauri-cli v2\.'
    if ($installed) {
        return
    }
    Write-Step 'Installing the Tauri CLI (first run only, several minutes)'
    Invoke-Checked 'Installing the Tauri CLI' { cargo install tauri-cli --version '^2' --locked }
}

# Windows embeds an .ico in the executable and the installer. It is rendered from the same master
# as the other icons by the Tauri CLI, and committed so a build does not depend on this step.
function Initialize-Icon {
    if (Test-Path $Icon) {
        return
    }
    Write-Step 'Rendering icons\icon.ico from scripts\icon-master.png'
    $scratch = Join-Path ([IO.Path]::GetTempPath()) ("rekall-icons-" + [Guid]::NewGuid().ToString('N'))
    try {
        Invoke-Checked 'Rendering the icon' { cargo tauri icon $IconMaster --output $scratch }
        Copy-Item (Join-Path $scratch 'icon.ico') $Icon
    }
    finally {
        Remove-Item $scratch -Recurse -Force -ErrorAction SilentlyContinue
    }
}

function Build-Frontend {
    Write-Step 'Building the console (rekall-ui)'
    Push-Location $Ui
    try {
        Invoke-Checked 'pnpm install' { pnpm install --frozen-lockfile }
        Invoke-Checked 'pnpm build' { pnpm build }
    }
    finally {
        Pop-Location
    }
}

# The Tauri CLI takes a config as a file: quoting inline JSON through PowerShell to a native
# command loses its double quotes.
function New-VersionConfig([string]$Number) {
    $file = Join-Path ([IO.Path]::GetTempPath()) ("rekall-version-" + [Guid]::NewGuid().ToString('N') + '.json')
    [IO.File]::WriteAllText($file, "{`"version`":`"$Number`"}", (New-Object Text.UTF8Encoding($false)))
    return $file
}

function Build-Installer {
    Write-Step 'Building the desktop app and its installer (the first build takes a while)'
    $versionConfig = $null
    Push-Location $Desktop
    try {
        if ($Version) {
            $env:REKALL_VERSION = $Version
            $versionConfig = New-VersionConfig $Version
            Invoke-Checked 'cargo tauri build' { cargo tauri build --bundles nsis --config $versionConfig }
        }
        else {
            Invoke-Checked 'cargo tauri build' { cargo tauri build --bundles nsis }
        }
    }
    finally {
        Pop-Location
        if ($versionConfig) {
            Remove-Item $versionConfig -Force -ErrorAction SilentlyContinue
        }
    }
}

function Find-Installer([datetime]$Since) {
    $found = Get-ChildItem $BundleFolder -Filter '*-setup.exe' -ErrorAction SilentlyContinue |
        Where-Object { $_.LastWriteTime -ge $Since } |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1
    if (-not $found) {
        throw "The build finished but no *-setup.exe was written under $BundleFolder."
    }
    return $found
}

$started = Get-Date

Write-Step 'Checking the toolchain'
Assert-Toolchain
Install-TauriCli
Initialize-Icon

if ($SkipFrontend) {
    if (-not (Test-Path (Join-Path $Ui 'dist\index.html'))) {
        throw 'rekall-ui\dist is missing, so -SkipFrontend has nothing to embed. Run without it.'
    }
}
else {
    Build-Frontend
}

Build-Installer
$installer = Find-Installer $started

$megabytes = [math]::Round($installer.Length / 1MB, 1)
Write-Host ''
Write-Host "Installer ready ($megabytes MB):" -ForegroundColor Green
Write-Host "    $($installer.FullName)"
