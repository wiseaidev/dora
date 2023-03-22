use std::{fmt, net::SocketAddr, path::PathBuf};

use crate::{
    config::{DataId, NodeId, NodeRunConfig},
    descriptor::{OperatorDefinition, ResolvedNode},
};
use dora_message::Metadata;
use uuid::Uuid;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct NodeConfig {
    pub dataflow_id: DataflowId,
    pub node_id: NodeId,
    pub run_config: NodeRunConfig,
    pub daemon_communication: DaemonCommunication,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DaemonCommunication {
    Shmem {
        daemon_control_region_id: SharedMemoryId,
        daemon_events_region_id: SharedMemoryId,
    },
    Tcp {
        socket_addr: SocketAddr,
    },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RuntimeConfig {
    pub node: NodeConfig,
    pub operators: Vec<OperatorDefinition>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum DaemonRequest {
    Register {
        dataflow_id: DataflowId,
        node_id: NodeId,
    },
    Subscribe,
    SendMessage {
        output_id: DataId,
        metadata: Metadata<'static>,
        data: Option<Data>,
    },
    CloseOutputs(Vec<DataId>),
    Stopped,
    NextEvent {
        drop_tokens: Vec<DropToken>,
    },
}

impl DaemonRequest {
    pub fn expects_tcp_reply(&self) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match self {
            DaemonRequest::SendMessage { .. } => false,
            _ => true,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum Data {
    Vec(Vec<u8>),
    SharedMemory {
        shared_memory_id: String,
        len: usize,
        drop_token: DropToken,
    },
}

impl Data {
    pub fn drop_token(&self) -> Option<DropToken> {
        match self {
            Data::Vec(_) => None,
            Data::SharedMemory { drop_token, .. } => Some(*drop_token),
        }
    }
}

impl fmt::Debug for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vec(v) => f
                .debug_struct("Vec")
                .field("len", &v.len())
                .finish_non_exhaustive(),
            Self::SharedMemory {
                shared_memory_id,
                len,
                drop_token,
            } => f
                .debug_struct("SharedMemory")
                .field("shared_memory_id", shared_memory_id)
                .field("len", len)
                .field("drop_token", drop_token)
                .finish(),
        }
    }
}

type SharedMemoryId = String;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum DaemonReply {
    Result(Result<(), String>),
    PreparedMessage { shared_memory_id: SharedMemoryId },
    NextEvents(Vec<NodeEvent>),
    Empty,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum NodeEvent {
    Stop,
    Input {
        id: DataId,
        metadata: Metadata<'static>,
        data: Option<Data>,
    },
    InputClosed {
        id: DataId,
    },
    AllInputsClosed,
    OutputDropped {
        drop_token: DropToken,
    },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DropEvent {
    pub tokens: Vec<DropToken>,
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct DropToken(Uuid);

impl DropToken {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum InputData {
    SharedMemory(SharedMemoryInput),
    Vec(Vec<u8>),
}

impl InputData {
    pub fn drop_token(&self) -> Option<DropToken> {
        match self {
            InputData::SharedMemory(data) => Some(data.drop_token),
            InputData::Vec(_) => None,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SharedMemoryInput {
    pub shared_memory_id: SharedMemoryId,
    pub len: usize,
    pub drop_token: DropToken,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum DaemonCoordinatorEvent {
    Spawn(SpawnDataflowNodes),
    StopDataflow { dataflow_id: DataflowId },
    Destroy,
    Watchdog,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum DaemonCoordinatorReply {
    SpawnResult(Result<(), String>),
    StopResult(Result<(), String>),
    DestroyResult(Result<(), String>),
    WatchdogAck,
}

pub type DataflowId = Uuid;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SpawnDataflowNodes {
    pub dataflow_id: DataflowId,
    pub working_dir: PathBuf,
    pub nodes: Vec<ResolvedNode>,
    pub daemon_communication: DaemonCommunicationConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DaemonCommunicationConfig {
    Tcp,
    Shmem,
}

impl Default for DaemonCommunicationConfig {
    fn default() -> Self {
        Self::Tcp
    }
}