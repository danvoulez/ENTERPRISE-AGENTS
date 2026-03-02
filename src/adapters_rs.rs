use anyhow::{anyhow, bail, Context, Result};
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use tokio::process::Command;

#[derive(Clone)]
pub struct AnthropicAdapter {
    model: String,
    api_key: Option<String>,
    http: Client,
}

impl AnthropicAdapter {
    pub fn new(model: String, api_key: Option<String>) -> Self {
        Self {
            model,
            api_key,
            http: Client::new(),
        }
    }

    pub async fn plan(&self, prompt: &str) -> Result<String> {
        let Some(api_key) = &self.api_key else {
            return Ok(format!("Plano local (fallback): {}", prompt));
        };

        let req = json!({
            "model": self.model,
            "max_tokens": 1200,
            "messages": [{"role":"user","content": format!("Crie um plano estruturado e objetivo para implementar: {prompt}")}]
        });

        let response: AnthropicMessageResponse = self
            .http
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&req)
            .send()
            .await
            .context("falha ao chamar Anthropic para planejamento")?
            .error_for_status()
            .context("Anthropic retornou erro em planejamento")?
            .json()
            .await
            .context("resposta Anthropic inválida (planning)")?;

        Ok(response.concat_text())
    }

    pub async fn review(&self, code: &str) -> Result<ReviewOutput> {
        let Some(api_key) = &self.api_key else {
            return Ok(ReviewOutput {
                summary: "Review local (fallback) sem issues críticas".to_string(),
                issues: vec![],
                code: code.to_string(),
            });
        };

        let req = json!({
            "model": self.model,
            "max_tokens": 1600,
            "messages": [{"role":"user","content": format!(
                "Revise o código abaixo e retorne JSON com summary, issues (severity,message) e code corrigido. Código:\n{code}"
            )}]
        });

        let response: AnthropicMessageResponse = self
            .http
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&req)
            .send()
            .await
            .context("falha ao chamar Anthropic para review")?
            .error_for_status()
            .context("Anthropic retornou erro em review")?
            .json()
            .await
            .context("resposta Anthropic inválida (review)")?;

        let text = response.concat_text();
        serde_json::from_str::<ReviewOutput>(&text).or_else(|_| {
            Ok(ReviewOutput {
                summary: text,
                issues: vec![],
                code: code.to_string(),
            })
        })
    }
}

#[derive(Clone)]
pub struct OllamaAdapter {
    model: String,
    base_url: String,
    http: Client,
}

impl OllamaAdapter {
    pub fn new(model: String, base_url: String) -> Self {
        Self {
            model,
            base_url,
            http: Client::new(),
        }
    }

    pub async fn code(&self, plan: &str) -> Result<String> {
        let req = json!({
            "model": self.model,
            "prompt": format!(
                "Implemente o plano abaixo em código real, sem stubs. \
        Retorne SOMENTE blocos de arquivo neste formato exato, sem texto fora dos blocos:\n\
        <file path=\"src/components/UserCard.tsx\">\n... código ...\n</file>\n\nPlano:\n{plan}"
            ),
            "stream": false
        });

        let response: OllamaGenerateResponse = self
            .http
            .post(format!(
                "{}/api/generate",
                self.base_url.trim_end_matches('/')
            ))
            .json(&req)
            .send()
            .await
            .context("falha ao chamar Ollama")?
            .error_for_status()
            .context("Ollama retornou erro")?
            .json()
            .await
            .context("resposta Ollama inválida")?;

        Ok(response.response)
    }
}

#[derive(Clone)]
pub struct GitAdapter {
    repo_root: String,
    branch: String,
    remote: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CommitOutput {
    pub sha: String,
    pub branch: String,
}

impl GitAdapter {
    pub fn new(repo_root: String, branch: String, remote: String) -> Self {
        Self {
            repo_root,
            branch,
            remote,
        }
    }

    pub async fn changed_files(&self) -> Result<Vec<String>> {
        let output = self.git_async(["status", "--porcelain"]).await?;
        Ok(output
            .lines()
            .filter_map(|line| line.get(3..).map(ToString::to_string))
            .collect())
    }

    pub async fn commit(
        &self,
        job_id: &str,
        title: &str,
        files: &[String],
        summary: &str,
    ) -> Result<CommitOutput> {
        if files.is_empty() {
            bail!("nenhum arquivo alterado para commit");
        }

        for file in files {
            self.git_async(["add", "--", file]).await?;
        }

        let message = format!("{title}\n\njob: {job_id}\n\n{summary}");
        self.git_async(["commit", "-m", &message]).await?;

        let sha = self.git_async(["rev-parse", "HEAD"]).await?;
        let branch = self
            .git_async(["rev-parse", "--abbrev-ref", "HEAD"])
            .await?;
        Ok(CommitOutput {
            sha: sha.trim().to_string(),
            branch: branch.trim().to_string(),
        })
    }

    pub async fn checkout_new_branch(&self, branch: &str) -> Result<()> {
        self.git_async(["checkout", &self.branch]).await?;
        self.git_async(["checkout", "-B", branch]).await?;
        Ok(())
    }

    pub async fn stash_if_needed(&self) -> Result<()> {
        let status = self.git_async(["status", "--porcelain"]).await?;
        if !status.trim().is_empty() {
            let _ = self
                .git_async(["stash", "push", "-u", "-m", "code247-auto-stash"])
                .await?;
        }
        Ok(())
    }

    pub async fn push_branch(&self, branch: &str) -> Result<()> {
        self.git_async(["push", &self.remote, branch]).await?;
        Ok(())
    }

    pub async fn git_async<const N: usize>(&self, args: [&str; N]) -> Result<String> {
        let out = Command::new("git")
            .current_dir(&self.repo_root)
            .args(args)
            .output()
            .await
            .with_context(|| format!("falha executando git {:?}", args))?;

        if !out.status.success() {
            return Err(anyhow!(
                "git {:?} falhou: {}",
                args,
                String::from_utf8_lossy(&out.stderr)
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }
}

#[derive(Clone)]
pub struct LinearAdapter {
    api_key: String,
    team_id: String,
    http: Client,
}

impl LinearAdapter {
    pub fn new(api_key: String, team_id: String) -> Self {
        Self {
            api_key,
            team_id,
            http: Client::new(),
        }
    }

    pub async fn get_issue(&self, issue_id: &str) -> Result<LinearIssue> {
        self.graphql(
            r#"query($id:String!){issue(id:$id){id identifier title description state{id name type}}}"#,
            json!({"id": issue_id}),
        )
        .await
    }

    pub async fn list_team_issues(&self, state_name: Option<&str>) -> Result<Vec<LinearIssue>> {
        let query = r#"
            query($teamId:String!, $stateName:String){
              issues(filter:{team:{id:{eq:$teamId}}, state:{name:{eq:$stateName}}}){
                nodes{ id identifier title state{id name type} }
              }
            }
        "#;
        let result: LinearIssuesResult = self
            .graphql(
                query,
                json!({"teamId": self.team_id, "stateName": state_name}),
            )
            .await?;
        Ok(result.issues.nodes)
    }

    pub async fn update_issue_state(&self, issue_id: &str, state_id: &str) -> Result<()> {
        let result: MutationOk = self
            .graphql(
                r#"mutation($id:String!, $stateId:String!){issueUpdate(id:$id,input:{stateId:$stateId}){success}}"#,
                json!({"id": issue_id, "stateId": state_id}),
            )
            .await?;
        if !result.issue_update.success {
            bail!("Linear não confirmou sucesso ao atualizar issue {issue_id}");
        }
        Ok(())
    }

    pub async fn bulk_update_issue_state(
        &self,
        issue_ids: &[String],
        state_id: &str,
    ) -> Result<()> {
        for issue_id in issue_ids {
            self.update_issue_state(issue_id, state_id).await?;
        }
        Ok(())
    }

    pub async fn find_state_id_by_type(&self, state_type: &str) -> Result<String> {
        let result: WorkflowStatesResult = self
            .graphql(
                r#"query($teamId:String!){team(id:$teamId){states{id name type}}}"#,
                json!({"teamId": self.team_id}),
            )
            .await?;
        result
            .team
            .states
            .into_iter()
            .find(|s| s.r#type.eq_ignore_ascii_case(state_type))
            .map(|s| s.id)
            .ok_or_else(|| anyhow!("estado Linear do tipo {state_type} não encontrado"))
    }

    async fn graphql<T: DeserializeOwned>(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<T> {
        let response: GraphqlEnvelope<T> = self
            .http
            .post("https://api.linear.app/graphql")
            .bearer_auth(&self.api_key)
            .json(&json!({"query": query, "variables": variables}))
            .send()
            .await
            .context("falha ao chamar Linear")?
            .error_for_status()
            .context("Linear retornou HTTP error")?
            .json()
            .await
            .context("resposta Linear inválida")?;

        if let Some(errors) = response.errors {
            let joined = errors
                .into_iter()
                .map(|e| e.message)
                .collect::<Vec<_>>()
                .join("; ");
            bail!("erro GraphQL Linear: {joined}");
        }
        response
            .data
            .ok_or_else(|| anyhow!("Linear retornou data vazia"))
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ReviewIssue {
    pub severity: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ReviewOutput {
    pub summary: String,
    pub issues: Vec<ReviewIssue>,
    pub code: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicMessageResponse {
    content: Vec<AnthropicContent>,
}

impl AnthropicMessageResponse {
    fn concat_text(&self) -> String {
        self.content
            .iter()
            .filter_map(|c| c.text.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

#[derive(Debug, Deserialize)]
struct GraphqlEnvelope<T> {
    data: Option<T>,
    errors: Option<Vec<GraphqlError>>,
}

#[derive(Debug, Deserialize)]
struct GraphqlError {
    message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LinearIssue {
    pub id: String,
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub state: LinearState,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LinearState {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Debug, Deserialize)]
struct LinearIssuesResult {
    issues: LinearIssueNodes,
}

#[derive(Debug, Deserialize)]
struct LinearIssueNodes {
    nodes: Vec<LinearIssue>,
}

#[derive(Debug, Deserialize)]
struct WorkflowStatesResult {
    team: TeamStates,
}

#[derive(Debug, Deserialize)]
struct TeamStates {
    states: Vec<LinearState>,
}

#[derive(Debug, Deserialize)]
struct MutationOk {
    #[serde(rename = "issueUpdate")]
    issue_update: MutationSuccess,
}

#[derive(Debug, Deserialize)]
struct MutationSuccess {
    success: bool,
}
