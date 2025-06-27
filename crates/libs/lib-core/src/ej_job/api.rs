use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{db::connection::DbConnection, ej_job::db::EjJobCreate, prelude::*};

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

#[derive(Debug, Serialize, Deserialize)]
pub enum EjJobCancelReason {
    NoBuilders,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum EjJobUpdate {
    JobStarted { nb_builders: usize },
    JobCancelled(EjJobCancelReason),
    JobAddedToQueue { queue_position: usize },
    JobFinished,
}

impl EjJob {
    pub fn create(self, connection: &mut DbConnection) -> Result<EjDeployableJob> {
        let job = EjJobCreate {
            commit_hash: self.commit_hash,
            remote_url: self.remote_url,
            job_type: self.job_type as i32,
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
