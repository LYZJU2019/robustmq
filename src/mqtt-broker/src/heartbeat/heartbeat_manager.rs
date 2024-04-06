use crate::server::MQTTProtocol;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone)]
pub struct HeartbeatManager {
    shard_num: u64,
    pub heartbeat_data: HashMap<u64, HeartbeatShard>,
}

impl HeartbeatManager {
    pub fn new(shard_num: u64) -> Self {
        return HeartbeatManager {
            shard_num,
            heartbeat_data: HashMap::new(),
        };
    }

    pub fn report_hearbeat(&mut self, connect_id: u64, live_time: ConnectionLiveTime) {
        let hash_num = self.calc_shard_hash_num(connect_id);
        if let Some(mut row) = self.heartbeat_data.remove(&hash_num) {
            row.report_hearbeat(connect_id, live_time);
            self.heartbeat_data.insert(connect_id, row);
        } else {
            let mut row = HeartbeatShard::new();
            row.report_hearbeat(connect_id, live_time);
            self.heartbeat_data.insert(connect_id, row);
        }
    }

    pub fn remove_connect(&mut self, connect_id: u64) {
        let hash_num = self.calc_shard_hash_num(connect_id);
        if let Some(mut row) = self.heartbeat_data.remove(&hash_num) {
            row.remove_connect(connect_id);
            self.heartbeat_data.insert(connect_id, row);
        }
    }

    pub fn calc_shard_hash_num(&self, connect_id: u64) -> u64 {
        return connect_id % self.shard_num;
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HeartbeatShard {
    pub heartbeat_data: HashMap<u64, ConnectionLiveTime>,
}

impl HeartbeatShard {
    pub fn new() -> Self {
        return HeartbeatShard {
            heartbeat_data: HashMap::new(),
        };
    }

    pub fn report_hearbeat(&mut self, connect_id: u64, live_time: ConnectionLiveTime) {
        self.heartbeat_data.insert(connect_id, live_time);
    }

    pub fn remove_connect(&mut self, connect_id: u64) {
        self.heartbeat_data.remove(&connect_id);
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConnectionLiveTime {
    pub protobol: MQTTProtocol,
    pub keep_live: u16,
    pub heartbeat: u128,
}
