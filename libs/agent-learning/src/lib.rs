//! Local learning metadata and FTS5 index. No raw model transcripts are duplicated here.
//! Connections are short-lived and migrations are guarded by SQLite's user_version.
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{path::Path, time::Duration};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Experience {
    pub id: i64,
    pub scope: String,
    pub source_run_id: String,
    pub kind: String,
    pub content: String,
    pub state: String,
    pub evidence: String,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SkillProposal {
    pub id: i64,
    pub skill_name: String,
    pub base_hash: String,
    pub proposed_markdown: String,
    pub source_experience_id: i64,
    pub status: String,
}
pub struct LearningDb { conn: Connection }

impl LearningDb {
    pub fn open(path: &Path) -> Result<Self,String> {
        if let Some(parent)=path.parent() { std::fs::create_dir_all(parent).map_err(|e|e.to_string())?; }
        let conn=Connection::open(path).map_err(|e|e.to_string())?;
        conn.busy_timeout(Duration::from_secs(5)).map_err(|e|e.to_string())?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA secure_delete=ON;")
            .map_err(|e|e.to_string())?;
        let mut db=Self{conn};
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&mut self) -> Result<(),String> {
        let version:i64=self.conn.query_row("PRAGMA user_version",[],|r|r.get(0)).map_err(|e|e.to_string())?;
        if version>1 { return Err(format!("unsupported newer learning schema version {version}")); }
        if version==0 {
            let tx=self.conn.transaction().map_err(|e|e.to_string())?;
            tx.execute_batch(r#"
                CREATE TABLE task_outcomes (
                    run_id TEXT PRIMARY KEY, session_id TEXT NOT NULL,
                    status TEXT NOT NULL, validator TEXT NOT NULL,
                    evidence TEXT NOT NULL, recorded_at INTEGER NOT NULL DEFAULT (unixepoch())
                );
                CREATE INDEX outcome_session ON task_outcomes(session_id,recorded_at);
                CREATE TABLE experiences (
                    id INTEGER PRIMARY KEY, scope TEXT NOT NULL, source_run_id TEXT NOT NULL,
                    kind TEXT NOT NULL, content TEXT NOT NULL,
                    evidence TEXT NOT NULL, state TEXT NOT NULL DEFAULT 'candidate'
                        CHECK(state IN ('candidate','approved','rejected')),
                    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
                    UNIQUE(scope,source_run_id,kind,content)
                );
                CREATE VIRTUAL TABLE experiences_fts USING fts5(content, content='experiences',content_rowid='id');
                CREATE TRIGGER experience_ai AFTER INSERT ON experiences BEGIN
                    INSERT INTO experiences_fts(rowid,content) VALUES (new.id,new.content);
                END;
                CREATE TRIGGER experience_ad AFTER DELETE ON experiences BEGIN
                    INSERT INTO experiences_fts(experiences_fts,rowid,content)
                    VALUES ('delete',old.id,old.content);
                END;
                CREATE TRIGGER experience_au AFTER UPDATE OF content ON experiences BEGIN
                    INSERT INTO experiences_fts(experiences_fts,rowid,content)
                    VALUES ('delete',old.id,old.content);
                    INSERT INTO experiences_fts(rowid,content) VALUES (new.id,new.content);
                END;
                CREATE TABLE skill_proposals (
                    id INTEGER PRIMARY KEY, skill_name TEXT NOT NULL,
                    base_hash TEXT NOT NULL, proposed_markdown TEXT NOT NULL,
                    source_experience_id INTEGER NOT NULL REFERENCES experiences(id) ON DELETE CASCADE,
                    status TEXT NOT NULL DEFAULT 'proposed'
                        CHECK(status IN ('proposed','approved','rejected','applied')),
                    created_at INTEGER NOT NULL DEFAULT (unixepoch())
                );
                CREATE TABLE eval_cases (
                    id TEXT PRIMARY KEY, domain TEXT NOT NULL,
                    description TEXT NOT NULL, expected_hash TEXT NOT NULL,
                    created_at INTEGER NOT NULL DEFAULT (unixepoch())
                );
                CREATE TABLE eval_results (
                    id INTEGER PRIMARY KEY, case_id TEXT NOT NULL REFERENCES eval_cases(id),
                    candidate TEXT NOT NULL, passed INTEGER NOT NULL CHECK(passed IN (0,1)),
                    evidence TEXT NOT NULL, created_at INTEGER NOT NULL DEFAULT (unixepoch())
                );
                PRAGMA user_version=1;
            "#).map_err(|e|e.to_string())?;
            tx.commit().map_err(|e|e.to_string())?;
        }
        // Fail explicitly on SQLite builds without FTS5; do not silently downgrade search.
        self.conn.query_row("SELECT count(*) FROM experiences_fts", [], |_r| Ok(()))
            .map_err(|e|format!("SQLite FTS5 unavailable: {e}"))?;
        Ok(())
    }

    pub fn record_outcome(&self, run: &str, session: &str, status: &str,
       validator: &str, evidence: &str) -> Result<(),String> {
        self.conn.execute("INSERT INTO task_outcomes(run_id,session_id,status,validator,evidence)
            VALUES(?1,?2,?3,?4,?5) ON CONFLICT(run_id) DO UPDATE SET
            status=excluded.status,validator=excluded.validator,evidence=excluded.evidence,
            recorded_at=unixepoch()",
            params![run,session,status,validator,evidence]).map_err(|e|e.to_string())?;
        Ok(())
    }
    pub fn add_experience(&self,scope:&str,run:&str,kind:&str,content:&str,evidence:&str) -> Result<i64,String>{
        if content.trim().is_empty() || content.len()>8000 { return Err("invalid experience length".into()); }
        self.conn.execute("INSERT OR IGNORE INTO experiences(scope,source_run_id,kind,content,evidence)
            VALUES(?1,?2,?3,?4,?5)",params![scope,run,kind,content,evidence]).map_err(|e|e.to_string())?;
        self.conn.query_row("SELECT id FROM experiences WHERE scope=?1 AND source_run_id=?2
            AND kind=?3 AND content=?4",params![scope,run,kind,content],|r|r.get(0))
            .map_err(|e|e.to_string())
    }
    pub fn set_experience_state(&self,id:i64,state:&str)->Result<(),String>{
        if !["candidate","approved","rejected"].contains(&state) {return Err("invalid state".into());}
        let n=self.conn.execute("UPDATE experiences SET state=?1 WHERE id=?2",params![state,id]).map_err(|e|e.to_string())?;
        if n==0{return Err("experience not found".into());}
        Ok(())
    }
    pub fn get_experience(&self,id:i64)->Result<Option<Experience>,String>{
        self.conn.query_row("SELECT id,scope,source_run_id,kind,content,state,evidence FROM experiences WHERE id=?1",
        params![id],|r|Ok(Experience{id:r.get(0)?,scope:r.get(1)?,source_run_id:r.get(2)?,
         kind:r.get(3)?,content:r.get(4)?,state:r.get(5)?,evidence:r.get(6)?})).optional().map_err(|e|e.to_string())
    }
    pub fn search_experiences(&self,scope:&str,query:&str,limit:usize)->Result<Vec<Experience>,String>{
        if query.trim().is_empty() || query.len()>256 {return Ok(Vec::new());}
        // FTS syntax is not exposed to the model: quote literals to avoid operators and malformed queries.
        let literal=format!("\"{}\"",query.replace('"', "\"\""));
        let mut stmt=self.conn.prepare("SELECT e.id,e.scope,e.source_run_id,e.kind,e.content,e.state,e.evidence
            FROM experiences_fts f JOIN experiences e ON e.id=f.rowid
            WHERE f.content MATCH ?1 AND e.scope=?2 AND e.state='approved'
            ORDER BY bm25(experiences_fts) LIMIT ?3").map_err(|e|e.to_string())?;
        let rows=stmt.query_map(params![literal,scope,limit.min(20) as i64],|r|Ok(Experience{
            id:r.get(0)?,scope:r.get(1)?,source_run_id:r.get(2)?,kind:r.get(3)?,
            content:r.get(4)?,state:r.get(5)?,evidence:r.get(6)?
        })).map_err(|e|e.to_string())?;
        rows.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())
    }
    pub fn propose_skill(&self,name:&str,hash:&str,markdown:&str,experience:i64)->Result<i64,String>{
        if name.is_empty()||markdown.len()>20000||markdown.trim().is_empty() {return Err("invalid skill proposal".into());}
        let state=self.get_experience(experience)?.ok_or("experience not found")?;
        if state.state!="approved" {return Err("only approved experience can propose skills".into());}
        self.conn.execute("INSERT INTO skill_proposals(skill_name,base_hash,proposed_markdown,source_experience_id)
            VALUES(?1,?2,?3,?4)",params![name,hash,markdown,experience]).map_err(|e|e.to_string())?;
        Ok(self.conn.last_insert_rowid())
    }
    pub fn get_skill_proposal(&self,id:i64)->Result<Option<SkillProposal>,String>{
        self.conn.query_row("SELECT id,skill_name,base_hash,proposed_markdown,source_experience_id,status
            FROM skill_proposals WHERE id=?1",params![id],|r|Ok(SkillProposal{
            id:r.get(0)?,skill_name:r.get(1)?,base_hash:r.get(2)?,proposed_markdown:r.get(3)?,
            source_experience_id:r.get(4)?,status:r.get(5)?
        })).optional().map_err(|e|e.to_string())
    }
    pub fn set_skill_proposal_status(&self,id:i64,from:&str,to:&str)->Result<bool,String>{
        let n=self.conn.execute("UPDATE skill_proposals SET status=?1 WHERE id=?2 AND status=?3",
        params![to,id,from]).map_err(|e|e.to_string())?;
        Ok(n==1)
    }
    pub fn record_eval_case(&self,id:&str,domain:&str,description:&str,hash:&str)->Result<(),String>{
        self.conn.execute("INSERT OR IGNORE INTO eval_cases(id,domain,description,expected_hash)
        VALUES(?1,?2,?3,?4)",params![id,domain,description,hash]).map_err(|e|e.to_string())?;
        let recorded:String=self.conn.query_row("SELECT expected_hash FROM eval_cases WHERE id=?1",
            params![id],|r|r.get(0)).map_err(|e|e.to_string())?;
        if !recorded.eq_ignore_ascii_case(hash) {return Err("evaluation case hash changed".into());}
        Ok(())
    }
    pub fn record_eval_result(&self,case:&str,candidate:&str,passed:bool,evidence:&str)->Result<(),String>{
        self.conn.execute("INSERT INTO eval_results(case_id,candidate,passed,evidence) VALUES(?1,?2,?3,?4)",
        params![case,candidate,passed,evidence]).map_err(|e|e.to_string())?;
        Ok(())
    }
    pub fn baseline_covers(&self,candidate:&str,baseline:&str)->Result<bool,String>{
        let missing:i64=self.conn.query_row(
          "SELECT count(*) FROM (SELECT DISTINCT case_id FROM eval_results WHERE candidate=?1) c
           WHERE NOT EXISTS (SELECT 1 FROM eval_results b WHERE b.case_id=c.case_id AND b.candidate=?2)",
          params![candidate,baseline],|r|r.get(0)).map_err(|e|e.to_string())?;
        Ok(missing==0)
    }
    pub fn eval_score(&self,candidate:&str)->Result<(i64,i64),String>{
        self.conn.query_row("SELECT count(*),coalesce(sum(passed),0)
            FROM (SELECT case_id,min(passed) AS passed FROM eval_results WHERE candidate=?1 GROUP BY case_id)",
        params![candidate],|r|Ok((r.get(0)?,r.get(1)?))).map_err(|e|e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn fts_only_approved_and_persists() {
        let p=std::env::temp_dir().join(format!("xiaoban-learning-{}.sqlite",std::process::id()));
        let _=std::fs::remove_file(&p);
        let db=LearningDb::open(&p).unwrap();
        let id=db.add_experience("personal","r1","procedure","organize invoice folders","checked by user").unwrap();
        assert!(db.search_experiences("personal","invoice",8).unwrap().is_empty());
        db.set_experience_state(id,"approved").unwrap();
        assert_eq!(db.search_experiences("personal","invoice",8).unwrap().len(),1);
        drop(db);
        assert_eq!(LearningDb::open(&p).unwrap().search_experiences("personal","invoice",8).unwrap().len(),1);
        let _=std::fs::remove_file(&p);
        let _=std::fs::remove_file(p.with_extension("sqlite-wal"));
        let _=std::fs::remove_file(p.with_extension("sqlite-shm"));
    }
}