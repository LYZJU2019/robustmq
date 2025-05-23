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

use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use common_base::tools::now_second;
use metadata_struct::adapter::record::Record;
use metadata_struct::mqtt::message::MqttMessage;
use protocol::mqtt::common::{Publish, PublishProperties, QoS};
use storage_adapter::storage::StorageAdapter;
use tokio::select;
use tokio::sync::broadcast::{self};
use tokio::time::sleep;
use tracing::{debug, error, info};

use super::common::{
    exclusive_publish_message_qos1, exclusive_publish_message_qos2, get_pkid, loop_commit_offset,
    min_qos, publish_message_qos,
};
use super::manager::SubscribeManager;
use super::meta::Subscriber;
use crate::handler::cache::{CacheManager, QosAckPacketInfo};
use crate::handler::error::MqttBrokerError;
use crate::handler::message::is_message_expire;
use crate::handler::sub_option::{get_retain_flag_by_retain_as_published, is_send_msg_by_bo_local};
use crate::server::connection_manager::ConnectionManager;
use crate::storage::message::MessageStorage;
use crate::subscribe::meta::SubPublishParam;

pub struct ExclusivePush<S> {
    cache_manager: Arc<CacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    message_storage: Arc<S>,
}

impl<S> ExclusivePush<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        message_storage: Arc<S>,
        cache_manager: Arc<CacheManager>,
        subscribe_manager: Arc<SubscribeManager>,
        connection_manager: Arc<ConnectionManager>,
    ) -> Self {
        ExclusivePush {
            message_storage,
            cache_manager,
            subscribe_manager,
            connection_manager,
        }
    }

    pub async fn start(&self) {
        loop {
            self.start_push_thread().await;
            self.try_thread_gc().await;
            sleep(Duration::from_secs(1)).await;
        }
    }

    async fn try_thread_gc(&self) {
        // Periodically verify that a push task is running, but the subscribe task has stopped
        // If so, stop the process and clean up the data
        for (exclusive_key, sx) in self.subscribe_manager.exclusive_push_thread.clone() {
            if !self
                .subscribe_manager
                .exclusive_push
                .contains_key(&exclusive_key)
            {
                if let Err(e) = sx.send(true) {
                    error!(
                        "exclusive push thread gc failed, exclusive_key: {:?}, error: {:?}",
                        exclusive_key, e
                    );
                }
                info!(
                    "exclusive push thread gc success, exclusive_key: {:?}",
                    exclusive_key
                );
                self.subscribe_manager
                    .exclusive_push_thread
                    .remove(&exclusive_key);
            }
        }
    }

    // Handles exclusive subscription push tasks
    // Exclusively subscribed messages are pushed directly to the consuming client
    async fn start_push_thread(&self) {
        for (exclusive_key, subscriber) in self.subscribe_manager.exclusive_push.clone() {
            if self
                .subscribe_manager
                .exclusive_push_thread
                .contains_key(&exclusive_key)
            {
                continue;
            }

            let (sub_thread_stop_sx, mut sub_thread_stop_rx) = broadcast::channel(1);

            let message_storage = MessageStorage::new(self.message_storage.clone());
            let cache_manager = self.cache_manager.clone();
            let connection_manager = self.connection_manager.clone();
            let subscribe_manager = self.subscribe_manager.clone();

            // Subscribe to the data push thread
            self.subscribe_manager
                .exclusive_push_thread
                .insert(exclusive_key.clone(), sub_thread_stop_sx.clone());

            tokio::spawn(async move {
                info!("Exclusive push thread for client_id [{}], sub_path: [{}], topic_id [{}] was started successfully",
                        subscriber.client_id, subscriber.sub_path, subscriber.topic_id);

                let group_id = build_group_name(&subscriber);
                let qos = build_pub_qos(&cache_manager, &subscriber);
                let sub_ids = build_sub_ids(&subscriber);

                let mut offset = match message_storage.get_group_offset(&group_id).await {
                    Ok(offset) => offset,
                    Err(e) => {
                        error!("{}", e);
                        subscribe_manager
                            .exclusive_push_thread
                            .remove(&exclusive_key);
                        return;
                    }
                };

                loop {
                    select! {
                        val = sub_thread_stop_rx.recv() =>{
                            if let Ok(flag) = val {
                                if flag {
                                    info!(
                                        "Exclusive Push thread for client_id [{}], sub_path: [{}], topic_id [{}] was stopped successfully",
                                        subscriber.client_id,
                                        subscriber.sub_path,
                                        subscriber.topic_id
                                    );

                                    subscribe_manager.exclusive_push_thread.remove(&exclusive_key);
                                    break;
                                }
                            }
                        },
                        val = pub_message(
                                &connection_manager,
                                &message_storage,
                                &cache_manager,
                                &subscriber,
                                &group_id,
                                &qos,
                                &sub_ids,
                                offset,
                                &sub_thread_stop_sx
                            ) => {
                                match val{
                                    Ok(offset_op) => {
                                        if let Some(off) = offset_op{
                                            offset = off + 1;
                                        }else{
                                            sleep(Duration::from_millis(100)).await;
                                        }
                                    }
                                    Err(e) => {
                                        error!(
                                            "Push message to client failed, failure message: {}, topic:{}, group{}",
                                            e.to_string(),
                                            subscriber.topic_id.clone(),
                                            group_id.clone()
                                        );
                                        sleep(Duration::from_millis(100)).await;
                                    }
                                }
                            }
                    }
                }
            });
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn pub_message<S>(
    connection_manager: &Arc<ConnectionManager>,
    message_storage: &MessageStorage<S>,
    cache_manager: &Arc<CacheManager>,
    subscriber: &Subscriber,
    group_id: &str,
    qos: &QoS,
    sub_ids: &[usize],
    offset: u64,
    sub_thread_stop_sx: &broadcast::Sender<bool>,
) -> Result<Option<u64>, MqttBrokerError>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let record_num = 5;
    let client_id = subscriber.client_id.clone();

    let results = message_storage
        .read_topic_message(&subscriber.topic_id, offset, record_num)
        .await?;

    if results.is_empty() {
        return Ok(None);
    }

    for record in results.iter() {
        let record_offset = record.offset.unwrap();

        // build publish params
        let sub_pub_param = if let Some(params) =
            build_pub_message(record.to_owned(), group_id, qos, subscriber, sub_ids).await?
        {
            params
        } else {
            continue;
        };

        let pkid = sub_pub_param.pkid;
        match qos {
            QoS::AtMostOnce => {
                publish_message_qos(
                    cache_manager,
                    connection_manager,
                    &sub_pub_param,
                    sub_thread_stop_sx,
                )
                .await?;
            }

            QoS::AtLeastOnce => {
                let (wait_puback_sx, _) = broadcast::channel(1);
                cache_manager.add_ack_packet(
                    &client_id,
                    pkid,
                    QosAckPacketInfo {
                        sx: wait_puback_sx.clone(),
                        create_time: now_second(),
                    },
                );

                exclusive_publish_message_qos1(
                    cache_manager,
                    connection_manager,
                    &sub_pub_param,
                    sub_thread_stop_sx,
                    &wait_puback_sx,
                )
                .await?;

                cache_manager.remove_ack_packet(&client_id, pkid);
            }

            QoS::ExactlyOnce => {
                let (wait_ack_sx, _) = broadcast::channel(1);
                cache_manager.add_ack_packet(
                    &client_id,
                    pkid,
                    QosAckPacketInfo {
                        sx: wait_ack_sx.clone(),
                        create_time: now_second(),
                    },
                );

                exclusive_publish_message_qos2(
                    cache_manager,
                    connection_manager,
                    &sub_pub_param,
                    sub_thread_stop_sx,
                    &wait_ack_sx,
                )
                .await?;

                cache_manager.remove_ack_packet(&client_id, pkid);
            }
        }

        // commit offset
        loop_commit_offset(
            message_storage,
            &subscriber.topic_id,
            group_id,
            record_offset,
        )
        .await?;
    }

    Ok(Some(results.last().unwrap().offset.unwrap()))
}

async fn build_pub_message(
    record: Record,
    group_id: &str,
    qos: &QoS,
    subscriber: &Subscriber,
    sub_ids: &[usize],
) -> Result<Option<SubPublishParam>, MqttBrokerError> {
    let msg = MqttMessage::decode_record(record.clone())?;

    if is_message_expire(&msg) {
        debug!("Message dropping: message expires, is not pushed to the client, and is discarded");
        return Ok(None);
    }

    if !is_send_msg_by_bo_local(subscriber.nolocal, &subscriber.client_id, &msg.client_id) {
        debug!(
            "Message dropping: message is not pushed to the client, because the client_id is the same as the subscriber, client_id: {}, topic_id: {}",
            subscriber.client_id, subscriber.topic_id
        );
        return Ok(None);
    }

    let retain = get_retain_flag_by_retain_as_published(subscriber.preserve_retain, msg.retain);

    let mut publish = Publish {
        dup: false,
        qos: qos.to_owned(),
        pkid: (now_second() % 65535) as u16,
        retain,
        topic: Bytes::from(subscriber.topic_name.clone()),
        payload: msg.payload,
    };

    let properties = PublishProperties {
        payload_format_indicator: msg.format_indicator,
        message_expiry_interval: Some(msg.expiry_interval as u32),
        topic_alias: None,
        response_topic: msg.response_topic,
        correlation_data: msg.correlation_data,
        user_properties: msg.user_properties,
        subscription_identifiers: sub_ids.into(),
        content_type: msg.content_type,
    };

    let pkid = get_pkid();
    publish.pkid = pkid;

    let sub_pub_param = SubPublishParam::new(
        subscriber.clone(),
        publish,
        Some(properties),
        record.timestamp as u128,
        group_id.to_string(),
        pkid,
    );
    Ok(Some(sub_pub_param))
}

fn build_group_name(subscriber: &Subscriber) -> String {
    format!(
        "system_sub_{}_{}_{}",
        subscriber.client_id, subscriber.sub_path, subscriber.topic_id
    )
}

fn build_pub_qos(cache_manager: &Arc<CacheManager>, subscriber: &Subscriber) -> QoS {
    let cluster_qos = cache_manager.get_cluster_info().protocol.max_qos;
    min_qos(cluster_qos, subscriber.qos)
}

fn build_sub_ids(subscriber: &Subscriber) -> Vec<usize> {
    let mut sub_ids = Vec::new();
    if let Some(id) = subscriber.subscription_identifier {
        sub_ids.push(id);
    }
    sub_ids
}

#[cfg(test)]
mod test {
    use common_base::tools::now_second;

    #[tokio::test]
    async fn pkid_test() {
        println!("{}", (now_second() % 65535) as u16)
    }
}
