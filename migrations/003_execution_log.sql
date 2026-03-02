CREATE TABLE IF NOT EXISTS execution_log (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  job_id TEXT NOT NULL,
  stage TEXT NOT NULL,
  input_hash TEXT,
  output_text TEXT NOT NULL,
  output_hash TEXT,
  tokens_used INTEGER,
  model_used TEXT,
  duration_ms INTEGER,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (job_id) REFERENCES jobs(id)
);

CREATE TABLE IF NOT EXISTS code_changes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  job_id TEXT NOT NULL,
  file_path TEXT NOT NULL,
  original_content TEXT,
  planned_content TEXT,
  reviewed_content TEXT,
  final_content TEXT NOT NULL,
  commit_sha TEXT,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (job_id) REFERENCES jobs(id)
);

CREATE TABLE IF NOT EXISTS conversations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  job_id TEXT,
  source TEXT NOT NULL,
  direction TEXT NOT NULL,
  message_type TEXT,
  payload TEXT NOT NULL,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_execution_log_job ON execution_log(job_id);
CREATE INDEX idx_code_changes_job ON code_changes(job_id);
CREATE INDEX idx_conversations_job ON conversations(job_id);
