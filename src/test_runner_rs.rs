use std::path::PathBuf;

use anyhow::Result;
use serde::Serialize;
use tokio::process::Command;

#[derive(Clone)]
pub struct TestRunner {
    repo_root: PathBuf,
}

#[derive(Serialize, Clone)]
pub struct ValidationResult {
    pub passed: bool,
    pub cargo_output: String,
    pub npm_output: Option<String>,
    pub errors: Vec<String>,
}

impl TestRunner {
    pub fn new(repo_root: impl Into<PathBuf>) -> Self {
        Self {
            repo_root: repo_root.into(),
        }
    }

    pub async fn validate(&self) -> Result<ValidationResult> {
        let mut errors = Vec::new();

        let cargo = run_cmd(&self.repo_root, "cargo", &["check"]).await?;
        if !cargo.0 {
            errors.push("cargo check falhou".to_string());
        }

        let npm = run_cmd(&self.repo_root, "npm", &["run", "typecheck"])
            .await
            .ok();
        if let Some((ok, _)) = &npm {
            if !*ok {
                errors.push("npm run typecheck falhou".to_string());
            }
        }

        Ok(ValidationResult {
            passed: errors.is_empty(),
            cargo_output: cargo.1,
            npm_output: npm.map(|(_, out)| out),
            errors,
        })
    }
}

async fn run_cmd(cwd: &PathBuf, cmd: &str, args: &[&str]) -> Result<(bool, String)> {
    let output = Command::new(cmd)
        .args(args)
        .current_dir(cwd)
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    Ok((output.status.success(), format!("{stdout}\n{stderr}")))
}
