import fs from 'node:fs';
import path from 'node:path';
import Database from 'better-sqlite3';

export const openDatabase = (dbPath: string): Database.Database => {
  fs.mkdirSync(path.dirname(dbPath), { recursive: true });
  const db = new Database(dbPath);
  db.pragma('journal_mode = WAL');
  return db;
};

export const runMigrations = (db: Database.Database): void => {
  const migrations = ['migrations/001_initial.sql', 'migrations/002_audit.sql', 'migrations/003_execution_log.sql'];
  for (const file of migrations) {
    db.exec(fs.readFileSync(file, 'utf-8'));
  }
};
