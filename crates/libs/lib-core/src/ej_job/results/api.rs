use crate::{ej_config::ej_config::EjConfig, ej_job::api::EjJobType, prelude::*};
use std::collections::HashMap;

use lib_models::{
    db::connection::DbConnection,
    job::{
        ejjob::EjJobDb, ejjob_logs::EjJobLogCreate, ejjob_results::EjJobResultCreate,
        ejjob_status::EjJobStatus,
    },
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait EjJobResult {
    fn save(self, connection: &mut DbConnection) -> Result<()>;
    fn job_id(&self) -> Uuid;
    fn builder_id(&self) -> Uuid;
}

#[derive(Debug)]
pub struct EjRunOutput<'a> {
    pub config: &'a EjConfig,
    pub logs: HashMap<Uuid, Vec<String>>,
    pub results: HashMap<Uuid, String>,
}

pub type EjBoardConfigId = Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct EjBuilderBuildResult {
    pub job_id: Uuid,
    pub builder_id: Uuid,
    pub logs: HashMap<EjBoardConfigId, Vec<String>>,
    pub successful: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EjBuilderRunResult {
    pub job_id: Uuid,
    pub builder_id: Uuid,
    pub logs: HashMap<EjBoardConfigId, Vec<String>>,
    pub results: HashMap<EjBoardConfigId, String>,
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
impl EjJobResult for EjBuilderBuildResult {
    fn save(self, connection: &mut DbConnection) -> Result<()> {
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

        for (board_config_id, logs) in self.logs.iter() {
            let log = EjJobLogCreate {
                ejjob_id: self.job_id.clone(),
                ejboard_config_id: *board_config_id,
                log: logs.join(""),
            };
            log.save(connection)?;
        }
        Ok(())
    }

    fn job_id(&self) -> Uuid {
        self.job_id
    }

    fn builder_id(&self) -> Uuid {
        self.builder_id
    }
}

impl EjJobResult for EjBuilderRunResult {
    fn save(self, connection: &mut DbConnection) -> Result<()> {
        let job = EjJobDb::fetch_by_id(&self.job_id, connection)?;

        let job_type: EjJobType = job.fetch_type(connection)?.into();
        if job_type != EjJobType::BuildAndRun {
            return Err(Error::InvalidJobType);
        }

        let job_status = if self.successful {
            EjJobStatus::success()
        } else {
            EjJobStatus::failed()
        };
        job.update_status(job_status, connection)?;

        for (board_config_id, logs) in self.logs.iter() {
            let logs = EjJobLogCreate {
                ejjob_id: self.job_id.clone(),
                ejboard_config_id: *board_config_id,
                log: logs.join(""),
            };
            logs.save(connection)?;
        }

        for (board_config_id, result) in self.results.iter() {
            let result = EjJobResultCreate {
                ejjob_id: self.job_id.clone(),
                ejboard_config_id: *board_config_id,
                result: result.to_string(),
            };
            result.save(connection)?;
        }
        Ok(())
    }
    fn job_id(&self) -> Uuid {
        self.job_id
    }

    fn builder_id(&self) -> Uuid {
        self.builder_id
    }
}
