use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use crate::{
    adapters_rs::{LinearIssue, ReviewOutput},
    persistence_rs::Job,
};

#[derive(Clone)]
pub struct PrCreator {
    github_token: String,
    github_repo: String,
    base_branch: String,
    http: Client,
}

impl PrCreator {
    pub fn new(github_token: String, github_repo: String, base_branch: String) -> Self {
        Self {
            github_token,
            github_repo,
            base_branch,
            http: Client::new(),
        }
    }

    pub async fn create(
        &self,
        job: &Job,
        issue: &LinearIssue,
        review: &ReviewOutput,
        branch: &str,
        files: &[String],
    ) -> Result<(u64, String)> {
        let title = format!("feat({}): {}", issue.identifier, issue.title);
        let body = format!(
            "## 🤖 job: {}\n\n**Issue**: {} — {}\n\n## Review\n{}\n\n## Files\n{}",
            job.id,
            issue.identifier,
            issue.title,
            review.summary,
            files
                .iter()
                .map(|f| format!("- `{f}`"))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let url = format!("https://api.github.com/repos/{}/pulls", self.github_repo);
        let resp: GithubPr = self
            .http
            .post(url)
            .bearer_auth(&self.github_token)
            .header("User-Agent", "code247-agent")
            .json(&json!({
                "title": title,
                "head": branch,
                "base": self.base_branch,
                "body": body,
            }))
            .send()
            .await
            .context("falha ao chamar GitHub pulls API")?
            .error_for_status()
            .context("GitHub retornou erro ao criar PR")?
            .json()
            .await
            .context("resposta GitHub inválida")?;

        Ok((resp.number, resp.html_url))
    }
}

#[derive(Deserialize)]
struct GithubPr {
    number: u64,
    html_url: String,
}
