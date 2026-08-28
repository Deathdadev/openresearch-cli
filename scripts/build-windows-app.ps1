# Build dist/OpenResearch for the Windows desktop app + CLI.
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Dist = Join-Path $Root "dist\OpenResearch"
$Version = (Select-String -Path (Join-Path $Root "Cargo.toml") -Pattern '^version = "(.*)"' | Select-Object -First 1).Matches.Groups[1].Value

Write-Host "==> Building release orx CLI (console subsystem)"
Push-Location $Root
cargo build --release --bin orx --locked
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
New-Item -ItemType Directory -Force -Path $Dist | Out-Null
Copy-Item (Join-Path $Root "target\release\orx.exe") (Join-Path $Dist "orx.exe")

Write-Host "==> Building GUI-subsystem OpenResearch.exe"
# Use the windows-gui feature instead of RUSTFLAGS: global linker flags also hit
# proc-macro crates and fail with "unresolved external symbol main".
cargo build --release --bin orx --features windows-gui --locked
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Copy-Item (Join-Path $Root "target\release\orx.exe") (Join-Path $Dist "OpenResearch.exe")

Write-Host "==> Assembling icons"
$iconDir = Join-Path $Root "dist\icons"
New-Item -ItemType Directory -Force -Path $iconDir | Out-Null
node (Join-Path $Root "scripts\generate-icon.mjs") (Join-Path $iconDir "icon-256.png") 256 | Out-Null
if (Get-Command magick -ErrorAction SilentlyContinue) {
  magick (Join-Path $iconDir "icon-256.png") (Join-Path $Dist "OpenResearch.ico")
}

Write-Host "==> Done: $Dist"
Pop-Location
