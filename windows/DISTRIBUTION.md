# Distributing OpenResearch on Windows

`scripts/build-windows-app.ps1` builds the install layout under `dist/OpenResearch/`.
`scripts/package-windows-app.ps1` packages it into `dist/OpenResearchSetup.exe` with
Inno Setup.

CI (`.github/workflows/release-windows-app.yml`) attaches the setup executable and
`windows-app.json` after a release when `WINDOWS_SIGNING_ENABLED=true`.

## Release assets

- `OpenResearchSetup.exe` — per-user installer (`%LOCALAPPDATA%\Programs\OpenResearch`)
- `windows-app.json` — updater manifest consumed by `src/updates/windows_app.rs`

```json
{ "version": "0.1.116", "tag": "v0.1.116", "asset": "OpenResearchSetup.exe", "sha256": "…" }
```

Upload order: delete old manifest → upload setup exe → publish new manifest.

## Local build

Requires [Inno Setup 6](https://jrsoftware.org/isinfo.php) on PATH as `ISCC.exe`.

```powershell
powershell -File scripts/build-windows-app.ps1
powershell -File scripts/package-windows-app.ps1
```

Unsigned builds work for local testing. Set `WINDOWS_SIGNING_ENABLED=true` and provide
an Authenticode certificate in the `release-signing` environment for signed releases.

## App updater

Installed desktop apps update via `orx update`, which downloads `windows-app.json`,
verifies the setup SHA-256, and runs the installer silently (`/VERYSILENT`).

The CLI installer uses `openresearch-cli-installer.ps1` from cargo-dist separately.
