//! Self-update for the downloadable Windows desktop app.
//!
//! Mirrors [`super::macos_app`]: fetch `windows-app.json`, download the setup
//! executable, verify its digest, and run a silent reinstall into the existing
//! prefix.

use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::error::{anyhow, Result};

const MANIFEST_ASSET: &str = "windows-app.json";

#[derive(Debug, Deserialize)]
pub struct AppManifest {
    pub version: String,
    pub tag: String,
    pub asset: String,
    pub sha256: String,
}

pub async fn fetch_manifest(timeout: Duration) -> Result<Option<AppManifest>> {
    let url = format!(
        "{}/releases/latest/download/{}",
        super::REPO_URL,
        MANIFEST_ASSET
    );
    let res = super::http()
        .get(&url)
        .header("user-agent", super::UA)
        .timeout(timeout)
        .send()
        .await
        .map_err(|e| anyhow!("Could not fetch the Windows app manifest: {}", e))?;
    if res.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    let status = res.status();
    if !status.is_success() {
        return Err(anyhow!(
            "App manifest request failed ({} {})",
            status.as_u16(),
            status.canonical_reason().unwrap_or("")
        ));
    }
    Ok(Some(serde_json::from_str(&res.text().await?)?))
}

pub async fn update(root: &Path, current: &Version, dry_run: bool, background: bool) -> Result<()> {
    let published = fetch_manifest(Duration::from_secs(10)).await?;
    let latest = published
        .as_ref()
        .map(|manifest| {
            Version::parse(&manifest.version).map_err(|e| {
                anyhow!(
                    "Could not parse the published app version {:?}: {}",
                    manifest.version,
                    e
                )
            })
        })
        .transpose()?;

    if let Some(latest) = &latest {
        super::write_check_cache(&latest.to_string());
    }

    let Some((manifest, latest)) = published
        .zip(latest)
        .filter(|(_, latest)| super::is_outdated(current, latest))
    else {
        if !background {
            println!("OpenResearch {} is up to date.", current);
        }
        return Ok(());
    };

    if dry_run {
        println!(
            "OpenResearch {} → {} is available. Re-run without --dry-run to update.",
            current, latest
        );
        return Ok(());
    }

    ensure_replaceable(root)?;
    if !background {
        eprintln!("Updating OpenResearch {} → {} ...", current, latest);
    }

    let bytes =
        super::fetch_release_asset(&manifest.tag, &manifest.asset, Duration::from_secs(120))
            .await?;
    verify_digest(&bytes, &manifest.sha256)?;

    let setup =
        std::env::temp_dir().join(format!("OpenResearchSetup-{}.exe", uuid::Uuid::new_v4()));
    std::fs::write(&setup, &bytes)
        .map_err(|e| anyhow!("Could not write {}: {}", setup.display(), e))?;

    let status = crate::process::command(&setup)
        .args(["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| anyhow!("Could not run the setup program: {}", e));
    let _ = std::fs::remove_file(&setup);
    let status = status?;
    if !status.success() {
        return Err(anyhow!(
            "The setup program exited with {}. The previous install is untouched.",
            status
        ));
    }

    super::record_installed(&latest.to_string());
    if !background {
        println!("✓ Updated OpenResearch {} → {}.", current, latest);
        println!("Restart the app to run the new version.");
    }
    Ok(())
}

fn ensure_replaceable(root: &Path) -> Result<()> {
    let parent = root
        .parent()
        .ok_or_else(|| anyhow!("{} has no parent directory", root.display()))?;
    super::probe_writable(parent).map_err(|e| {
        anyhow!(
            "Can't write to {} ({e}), so OpenResearch can't update itself.",
            parent.display()
        )
    })?;
    Ok(())
}

fn verify_digest(bytes: &[u8], expected: &str) -> Result<()> {
    let actual = format!("{:x}", Sha256::digest(bytes));
    if actual.eq_ignore_ascii_case(expected.trim()) {
        Ok(())
    } else {
        Err(anyhow!(
            "Download checksum mismatch (expected {expected}, got {actual})."
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_check_is_case_insensitive() {
        verify_digest(
            b"hello",
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
        )
        .unwrap();
    }
}
