use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    db::connection::DbConnection,
    ej_config::ej_config::{EjConfig, EjDispatcherConfig},
    ej_job::{
        db::{EjJobCreate, EjJobDb},
        logs::db::EjJobLogCreate,
        results::db::EjJobResultCreate,
        status::db::EjJobStatus,
    },
    prelude::*,
};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum EjJobType {
    Build = 0,
    Run = 1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EjJob {
    pub job_type: EjJobType,
    pub commit_hash: String,
    pub remote_url: String,
    pub remote_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EjDeployableJob {
    pub id: Uuid,
    pub job_type: EjJobType,
    pub commit_hash: String,
    pub remote_url: String,
    pub remote_token: Option<String>,
}

#[derive(Debug)]
pub struct EjRunOutput<'a> {
    pub config: &'a EjConfig,
    pub logs: HashMap<(usize, usize), Vec<String>>,
    pub results: HashMap<(usize, usize), String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EjBuildResult {
    pub job_id: Uuid,
    pub builder_id: Uuid,
    pub config: EjDispatcherConfig,
    pub logs: HashMap<(usize, usize), Vec<String>>,
    pub successful: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EjRunResult {
    pub job_id: Uuid,
    pub builder_id: Uuid,
    pub config: EjDispatcherConfig,
    pub logs: HashMap<(usize, usize), Vec<String>>,
    pub results: HashMap<(usize, usize), String>,
    pub successful: bool,
}

impl<'a> EjRunOutput<'a> {
    pub fn new(config: &'a EjConfig) -> Self {
        Self {
            config,
            logs: HashMap::new(),
            results: HashMap::new(),
        }
    }
    pub fn reset(&mut self) {
        self.logs.clear();
        self.results.clear();
    }
}

impl EjJob {
    pub fn create(self, connection: &mut DbConnection) -> Result<EjDeployableJob> {
        let job = EjJobCreate {
            commit_hash: self.commit_hash,
            remote_url: self.remote_url,
        };
        let job = job.save(connection)?;

        Ok(EjDeployableJob {
            id: job.id,
            job_type: job.job_type.into(),
            commit_hash: job.commit_hash,
            remote_url: job.remote_url,
            remote_token: self.remote_token,
        })
    }
}
impl EjBuildResult {
    pub fn save(self, connection: &mut DbConnection) -> Result<Self> {
        let job = EjJobDb::fetch_by_id(&self.job_id, connection)?;
        let job_type: EjJobType = job.fetch_type(connection)?.into();
        if job_type != EjJobType::Build {
            return Err(Error::InvalidJobType);
        }

        let job_status = if self.successful {
            EjJobStatus::success()
        } else {
            EjJobStatus::failed()
        };
        job.update_status(job_status, connection)?;

        for ((board_idx, board_config_idx), logs) in self.logs.iter() {
            let board = &self.config.boards[*board_idx];
            let config = &board.configs[*board_config_idx];
            let log = EjJobLogCreate {
                ejjob_id: self.job_id.clone(),
                ejboard_config_id: config.id.clone(),
                log: logs.join(""),
            };
            log.save(connection)?;
        }
        Ok(self)
    }
}

impl EjRunResult {
    pub fn save(self, connection: &mut DbConnection) -> Result<Self> {
        let job = EjJobDb::fetch_by_id(&self.job_id, connection)?;

        let job_type: EjJobType = job.fetch_type(connection)?.into();
        if job_type != EjJobType::Run {
            return Err(Error::InvalidJobType);
        }

        let job_status = if self.successful {
            EjJobStatus::success()
        } else {
            EjJobStatus::failed()
        };
        job.update_status(job_status, connection)?;

        for ((board_idx, board_config_idx), logs) in self.logs.iter() {
            let board = &self.config.boards[*board_idx];
            let config = &board.configs[*board_config_idx];

            let logs = EjJobLogCreate {
                ejjob_id: self.job_id.clone(),
                ejboard_config_id: config.id.clone(),
                log: logs.join(""),
            };
            logs.save(connection)?;
        }

        for ((board_idx, board_config_idx), result) in self.results.iter() {
            let board = &self.config.boards[*board_idx];
            let config = &board.configs[*board_config_idx];
            let result = EjJobResultCreate {
                ejjob_id: self.job_id.clone(),
                ejboard_config_id: config.id.clone(),
                result: result.clone(),
            };
            result.save(connection)?;
        }
        Ok(self)
    }
}
