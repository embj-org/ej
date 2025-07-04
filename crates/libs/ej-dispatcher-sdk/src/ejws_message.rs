use std::{fmt, time::Duration};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ejjob::{EjDeployableJob, EjJobCancelReason};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EjWsServerMessage {
    Build(EjDeployableJob),
    BuildAndRun(EjDeployableJob),
    Cancel(EjJobCancelReason, Uuid),
    Close,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EjWsClientMessage {}
