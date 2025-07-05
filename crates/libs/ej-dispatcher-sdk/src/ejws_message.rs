//! WebSocket message types for builder communication.

use std::{fmt, time::Duration};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ejjob::{EjDeployableJob, EjJobCancelReason};

/// Messages sent from dispatcher to builder via WebSocket.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EjWsServerMessage {
    /// Build job assignment.
    Build(EjDeployableJob),
    /// Build and run job assignment.
    BuildAndRun(EjDeployableJob),
    /// Cancel job with reason and ID.
    Cancel(EjJobCancelReason, Uuid),
    /// Close WebSocket connection.
    Close,
}

/// Messages sent from builder to dispatcher via WebSocket.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EjWsClientMessage {}
