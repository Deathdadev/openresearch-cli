# Package dist/OpenResearch into dist/OpenResearchSetup.exe with Inno Setup.
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$AppDir = Join-Path $Root "dist\OpenResearch"
$Iss = Join-Path $Root "windows\installer.iss"
$Version = (Select-String -Path (Join-Path $Root "Cargo.toml") -Pattern '^version = "(.*)"' | Select-Object -First 1).Matches.Groups[1].Value

if (-not (Test-Path $AppDir)) {
  Write-Error "dist/OpenResearch not found — run scripts/build-windows-app.ps1 first"
}

$iscc = Get-Command ISCC.exe -ErrorAction SilentlyContinue
if (-not $iscc) {
  Write-Error "ISCC.exe (Inno Setup) not found on PATH"
}

$issContent = Get-Content $Iss -Raw
$issContent = $issContent -replace '#define MyAppVersion ".*"', "#define MyAppVersion `"$Version`""
$tempIss = Join-Path $Root "dist\OpenResearch.iss"
Set-Content -Path $tempIss -Value $issContent -Encoding UTF8

Write-Host "==> Building OpenResearchSetup.exe"
& $iscc.Source $tempIss
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==> Done: $(Join-Path $Root 'dist\OpenResearchSetup.exe')"
