use serde::{Deserialize, Serialize};

use crate::{
    ej_client::api::EjClientPost,
    ej_job::api::{EjDeployableJob, EjJob},
};

#[derive(Debug, Serialize, Deserialize)]
pub enum EjServerMessage {
    Build(EjDeployableJob),
    Run(EjDeployableJob),
    Close,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EjClientMessage {}

#[derive(Debug, Serialize, Deserialize)]
pub enum EjSocketMessage {
    CreateRootUser(EjClientPost),
    Dispatch(EjJob),
}
