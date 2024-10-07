// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use bincode::{deserialize, serialize};
use clients::{placement::openraft::OpenRaftServiceManager, poll::ClientPool};
use common_base::error::common::CommonError;
use mobc::Connection;
use openraft::{
    error::{InstallSnapshotError, RPCError, RaftError},
    network::RPCOption,
    raft::{
        AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest,
        InstallSnapshotResponse, VoteRequest, VoteResponse,
    },
    RaftNetwork,
};
use protocol::placement_center::generate::openraft::{AppendRequest, SnapshotRequest};
use std::sync::Arc;

use crate::raftv2::{error::to_error, typeconfig::TypeConfig};

pub struct NetworkConnection {
    addr: String,
    client_poll: Arc<ClientPool>,
}
impl NetworkConnection {
    pub fn new(addr: String, client_poll: Arc<ClientPool>) -> Self {
        return NetworkConnection { addr, client_poll };
    }

    async fn c(&mut self) -> Result<Connection<OpenRaftServiceManager>, CommonError> {
        return Ok(self
            .client_poll
            .placement_center_openraft_services_client(self.addr.clone())
            .await?);
    }
}

#[allow(clippy::blocks_in_conditions)]
impl RaftNetwork<TypeConfig> for NetworkConnection {
    async fn append_entries(
        &mut self,
        req: AppendEntriesRequest<TypeConfig>,
        _option: RPCOption,
    ) -> Result<AppendEntriesResponse<TypeConfig>, RPCError<TypeConfig, RaftError<TypeConfig>>>
    {
        let mut c = match self.c().await {
            Ok(conn) => conn,
            Err(e) => return Err(to_error(e)),
        };

        let value = match serialize(&req) {
            Ok(data) => data,
            Err(e) => return Err(to_error(CommonError::CommmonError(e.to_string()))),
        };

        let request = AppendRequest { value };

        let reply = match c.append(request).await {
            Ok(reply) => reply.into_inner(),
            Err(e) => return Err(to_error(CommonError::CommmonError(e.to_string()))),
        };

        let result = match deserialize(&reply.value) {
            Ok(data) => data,
            Err(e) => return Err(to_error(CommonError::CommmonError(e.to_string()))),
        };

        return Ok(result);
    }

    async fn install_snapshot(
        &mut self,
        req: InstallSnapshotRequest<TypeConfig>,
        _option: RPCOption,
    ) -> Result<
        InstallSnapshotResponse<TypeConfig>,
        RPCError<TypeConfig, RaftError<TypeConfig, InstallSnapshotError>>,
    > {
        let mut c = match self.c().await {
            Ok(conn) => conn,
            Err(e) => return Err(to_error(e)),
        };

        let value = match serialize(&req) {
            Ok(data) => data,
            Err(e) => return Err(to_error(CommonError::CommmonError(e.to_string()))),
        };

        let request = SnapshotRequest { value };

        let reply = match c.snapshot(request).await {
            Ok(reply) => reply.into_inner(),
            Err(e) => return Err(to_error(CommonError::CommmonError(e.to_string()))),
        };
        let result = match deserialize(&reply.value) {
            Ok(data) => data,
            Err(e) => return Err(to_error(CommonError::CommmonError(e.to_string()))),
        };

        return Ok(result);
    }

    async fn vote(
        &mut self,
        req: VoteRequest<TypeConfig>,
        _option: RPCOption,
    ) -> Result<VoteResponse<TypeConfig>, RPCError<TypeConfig, RaftError<TypeConfig>>> {
        let mut c = match self.c().await {
            Ok(conn) => conn,
            Err(e) => return Err(to_error(e)),
        };

        let value = match serialize(&req) {
            Ok(data) => data,
            Err(e) => return Err(to_error(CommonError::CommmonError(e.to_string()))),
        };

        let request = protocol::placement_center::generate::openraft::VoteRequest { value };

        let reply = match c.vote(request).await {
            Ok(reply) => reply.into_inner(),
            Err(e) => return Err(to_error(CommonError::CommmonError(e.to_string()))),
        };
        let result = match deserialize(&reply.value) {
            Ok(data) => data,
            Err(e) => return Err(to_error(CommonError::CommmonError(e.to_string()))),
        };

        return Ok(result);
    }
}