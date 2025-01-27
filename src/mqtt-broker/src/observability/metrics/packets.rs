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

use lazy_static::lazy_static;
use prometheus::{register_int_gauge_vec, IntGaugeVec};
use protocol::mqtt::codec::{calc_mqtt_packet_size, MqttPacketWrapper};
use protocol::mqtt::common::{ConnectReturnCode, MqttPacket, QoS};

use crate::handler::constant::{METRICS_KEY_NETWORK_TYPE, METRICS_KEY_QOS};
use crate::server::connection::{NetworkConnection, NetworkConnectionType};

lazy_static! {
    // Number of packets received
    static ref PACKETS_RECEIVED: IntGaugeVec = register_int_gauge_vec!(
        "packets_received",
        "Number of packets received",
        &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();

    // Number of packets connect received
    static ref PACKETS_CONNECT_RECEIVED: IntGaugeVec = register_int_gauge_vec!(
        "packets_connect_received",
        "Number of packets connect received",
        &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();

    // Number of packets publish received
    static ref PACKETS_PUBLISH_RECEIVED: IntGaugeVec = register_int_gauge_vec!(
        "packets_publish_received",
        "Number of packets publish received",
        &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();

    // Number of packets connack received
    static ref PACKETS_CONNACK_RECEIVED: IntGaugeVec = register_int_gauge_vec!(
        "packets_connack_received",
        "Number of packets connack received",
        &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();

    // Number of packets puback received
    static ref PACKETS_PUBACK_RECEIVED: IntGaugeVec = register_int_gauge_vec!(
    "packets_puback_received",
    "Number of packets puback received",
    &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();

    // Number of packets pubrec received
    static ref PACKETS_PUBREC_RECEIVED: IntGaugeVec = register_int_gauge_vec!(
        "packets_pubrec_received",
        "Number of packets pubrec received",
        &[METRICS_KEY_NETWORK_TYPE]
        )
        .unwrap();

    // Number of packets pubrel received
    static ref PACKETS_PUBREL_RECEIVED: IntGaugeVec = register_int_gauge_vec!(
        "packets_pubrel_received",
        "Number of packets pubrel received",
        &[METRICS_KEY_NETWORK_TYPE]
        )
        .unwrap();

    // Number of packets pubcomp received
    static ref PACKETS_PUBCOMP_RECEIVED: IntGaugeVec = register_int_gauge_vec!(
        "packets_pubcomp_received",
        "Number of packets pubcomp received",
        &[METRICS_KEY_NETWORK_TYPE]
        )
        .unwrap();

    // Number of packets subscrible received
    static ref PACKETS_SUBSCRIBLE_RECEIVED: IntGaugeVec = register_int_gauge_vec!(
    "packets_subscrible_received",
    "Number of packets subscrible received",
    &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();

    // Number of packets unsubscrible received
    static ref PACKETS_UNSUBSCRIBLE_RECEIVED: IntGaugeVec = register_int_gauge_vec!(
    "packets_unsubscrible_received",
    "Number of packets unsubscrible received",
    &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();

    // Number of packets pingreq received
    static ref PACKETS_PINGREQ_RECEIVED: IntGaugeVec = register_int_gauge_vec!(
    "packets_pingreq_received",
    "Number of packets pingreq received",
    &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();

    // Number of packets disconnect received
    static ref PACKETS_DISCONNECT_RECEIVED: IntGaugeVec = register_int_gauge_vec!(
    "packets_disconnect_received",
    "Number of packets disconnect received",
    &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();

    // Number of packets auth received
    static ref PACKETS_AUTH_RECEIVED: IntGaugeVec = register_int_gauge_vec!(
    "packets_auth_received",
    "Number of packets auth received",
    &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();


    // Number of error packets received
    static ref PACKETS_RECEIVED_ERROR: IntGaugeVec = register_int_gauge_vec!(
        "packets_received_error",
        "Number of error packets received",
        &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();

    // Number of packets.connack.auth_error received
    static ref PACKETS_CONNACK_AUTH_ERROR: IntGaugeVec = register_int_gauge_vec!(
        "packets_connack_auth_error",
        "Number of connack auth error packets received",
        &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();

    // Number of packets.connack.error received
    static ref PACKETS_CONNACK_ERROR: IntGaugeVec = register_int_gauge_vec!(
        "packets_connack_error",
        "Number of connack error packets received",
        &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();

    // Number of packets sent
    static ref PACKETS_SENT: IntGaugeVec = register_int_gauge_vec!(
        "packets_sent",
        "Number of packets sent",
        &[METRICS_KEY_NETWORK_TYPE,METRICS_KEY_QOS]
    )
    .unwrap();

    // Number of packets connack sent
    static ref PACKETS_CONNACK_SENT: IntGaugeVec = register_int_gauge_vec!(
        "packets_connack_sent",
        "Number of packets connack sent",
        &[METRICS_KEY_NETWORK_TYPE,METRICS_KEY_QOS]
    )
    .unwrap();

    // Number of packets publish sent
    static ref PACKETS_PUBLISH_SENT: IntGaugeVec = register_int_gauge_vec!(
        "packets_publish_sent",
        "Number of packets publish sent",
        &[METRICS_KEY_NETWORK_TYPE,METRICS_KEY_QOS]
    )
    .unwrap();

    // Number of packets puback sent
    static ref PACKETS_PUBACK_SENT: IntGaugeVec = register_int_gauge_vec!(
        "packets_puback_sent",
        "Number of packets puback sent",
        &[METRICS_KEY_NETWORK_TYPE,METRICS_KEY_QOS]
    )
    .unwrap();

    // Number of packets pubrec sent
    static ref PACKETS_PUBREC_SENT: IntGaugeVec = register_int_gauge_vec!(
        "packets_pubrec_sent",
        "Number of packets pubrec sent",
        &[METRICS_KEY_NETWORK_TYPE,METRICS_KEY_QOS]
    )
    .unwrap();

   // Number of packets pubrel sent
   static ref PACKETS_PUBREL_SENT: IntGaugeVec = register_int_gauge_vec!(
    "packets_pubrel_sent",
    "Number of packets pubrel sent",
    &[METRICS_KEY_NETWORK_TYPE,METRICS_KEY_QOS]
    )
    .unwrap();

   // Number of packets pubcomp sent
   static ref PACKETS_PUBCOMP_SENT: IntGaugeVec = register_int_gauge_vec!(
    "packets_pubcomp_sent",
    "Number of packets pubcomp sent",
    &[METRICS_KEY_NETWORK_TYPE,METRICS_KEY_QOS]
    )
    .unwrap();

   // Number of packets suback sent
   static ref PACKETS_SUBACK_SENT: IntGaugeVec = register_int_gauge_vec!(
    "packets_suback_sent",
    "Number of packets suback sent",
    &[METRICS_KEY_NETWORK_TYPE,METRICS_KEY_QOS]
    )
    .unwrap();

   // Number of packets unsuback sent
   static ref PACKETS_UNSUBACK_SENT: IntGaugeVec = register_int_gauge_vec!(
    "packets_unsuback_sent",
    "Number of packets unsuback sent",
    &[METRICS_KEY_NETWORK_TYPE,METRICS_KEY_QOS]
    )
    .unwrap();

   // Number of packets pingresp sent
   static ref PACKETS_PINGRESP_SENT: IntGaugeVec = register_int_gauge_vec!(
    "packets_pingresp_sent",
    "Number of packets pingresp sent",
    &[METRICS_KEY_NETWORK_TYPE,METRICS_KEY_QOS]
    )
    .unwrap();

   // Number of packets disconnect sent
   static ref PACKETS_DISCONNECT_SENT: IntGaugeVec = register_int_gauge_vec!(
    "packets_disconnect_sent",
    "Number of packets disconnect sent",
    &[METRICS_KEY_NETWORK_TYPE,METRICS_KEY_QOS]
    )
    .unwrap();

   // Number of packets auth sent
   static ref PACKETS_AUTH_SENT: IntGaugeVec = register_int_gauge_vec!(
    "packets_auth_sent",
    "Number of packets auth sent",
    &[METRICS_KEY_NETWORK_TYPE,METRICS_KEY_QOS]
    )
    .unwrap();

    // Number of bytes received
    static ref BYTES_RECEIVED: IntGaugeVec = register_int_gauge_vec!(
        "bytes_received",
        "Number of bytes received",
        &[METRICS_KEY_NETWORK_TYPE]
    )
    .unwrap();

    // Number of bytes sent
    static ref BYTES_SENT: IntGaugeVec = register_int_gauge_vec!(
        "bytes_sent",
        "Number of bytes sent",
        &[METRICS_KEY_NETWORK_TYPE,METRICS_KEY_QOS]
    )
    .unwrap();

    // Number of reserved messages received
    static ref RETAIN_PACKETS_RECEIVED: IntGaugeVec = register_int_gauge_vec!(
        "retain_packets_received",
        "Number of reserved messages received",
        &[METRICS_KEY_QOS]
    )
    .unwrap();

    static ref RETAIN_PACKETS_SEND: IntGaugeVec = register_int_gauge_vec!(
        "retain_packets_sent",
        "Number of reserved messages sent",
        &[METRICS_KEY_QOS]
    )
    .unwrap();

}

// Record the packet-related metrics received by the server for failed resolution
pub fn record_received_error_metrics(network_type: NetworkConnectionType) {
    PACKETS_RECEIVED_ERROR
        .with_label_values(&[&network_type.to_string()])
        .inc();
}

// Record metrics related to packets received by the server
pub fn record_received_metrics(
    connection: &NetworkConnection,
    pkg: &MqttPacket,
    network_type: &NetworkConnectionType,
) {
    let payload_size = if let Some(protocol) = connection.protocol.clone() {
        let wrapper = MqttPacketWrapper {
            protocol_version: protocol.into(),
            packet: pkg.clone(),
        };
        calc_mqtt_packet_size(wrapper)
    } else {
        0
    };

    BYTES_RECEIVED
        .with_label_values(&[&network_type.to_string()])
        .add(payload_size as i64);

    PACKETS_RECEIVED
        .with_label_values(&[&network_type.to_string()])
        .inc();

    match pkg {
        MqttPacket::Connect(_, _, _, _, _, _) => PACKETS_CONNECT_RECEIVED
            .with_label_values(&[&network_type.to_string()])
            .inc(),

        MqttPacket::ConnAck(_, _) => PACKETS_CONNACK_RECEIVED
            .with_label_values(&[&network_type.to_string()])
            .inc(),

        MqttPacket::Publish(_, _) => PACKETS_PUBLISH_RECEIVED
            .with_label_values(&[&network_type.to_string()])
            .inc(),

        MqttPacket::PubAck(_, _) => PACKETS_PUBACK_RECEIVED
            .with_label_values(&[&network_type.to_string()])
            .inc(),

        MqttPacket::PubRec(_, _) => PACKETS_PUBREC_RECEIVED
            .with_label_values(&[&network_type.to_string()])
            .inc(),

        MqttPacket::PubRel(_, _) => PACKETS_PUBREL_RECEIVED
            .with_label_values(&[&network_type.to_string()])
            .inc(),

        MqttPacket::PubComp(_, _) => PACKETS_PUBCOMP_RECEIVED
            .with_label_values(&[&network_type.to_string()])
            .inc(),

        MqttPacket::PingReq(_) => PACKETS_PINGREQ_RECEIVED
            .with_label_values(&[&network_type.to_string()])
            .inc(),

        MqttPacket::Disconnect(_, _) => PACKETS_DISCONNECT_RECEIVED
            .with_label_values(&[&network_type.to_string()])
            .inc(),

        MqttPacket::Auth(_, _) => PACKETS_AUTH_RECEIVED
            .with_label_values(&[&network_type.to_string()])
            .inc(),

        MqttPacket::Subscribe(_, _) => PACKETS_SUBSCRIBLE_RECEIVED
            .with_label_values(&[&network_type.to_string()])
            .inc(),

        MqttPacket::Unsubscribe(_, _) => PACKETS_UNSUBSCRIBLE_RECEIVED
            .with_label_values(&[&network_type.to_string()])
            .inc(),
        _ => unreachable!("This branch only matches for packets could not be received"),
    }
}

// Record metrics related to messages pushed to the client
pub fn record_sent_metrics(packet_wrapper: &MqttPacketWrapper, network_type: String) {
    let qos_str = if let MqttPacket::Publish(publish, _) = packet_wrapper.packet.clone() {
        format!("{}", publish.qos as u8)
    } else {
        "-1".to_string()
    };

    let payload_size = calc_mqtt_packet_size(packet_wrapper.to_owned());

    PACKETS_SENT
        .with_label_values(&[&network_type, &qos_str])
        .inc();

    BYTES_SENT
        .with_label_values(&[&network_type, &qos_str])
        .add(payload_size as i64);

    match &packet_wrapper.packet {
        MqttPacket::ConnAck(conn_ack, _) => {
            PACKETS_CONNACK_SENT
                .with_label_values(&[&network_type, &qos_str])
                .inc();
            // 判断是否为 NotAuthorized
            if conn_ack.code == ConnectReturnCode::NotAuthorized {
                PACKETS_CONNACK_AUTH_ERROR
                    .with_label_values(&[&network_type.to_string()])
                    .inc();
            }
            // 判断是否为 connack error
            if conn_ack.code != ConnectReturnCode::Success {
                PACKETS_CONNACK_ERROR
                    .with_label_values(&[&network_type.to_string()])
                    .inc();
            }
        }
        MqttPacket::Publish(_, _) => PACKETS_PUBLISH_SENT
            .with_label_values(&[&network_type, &qos_str])
            .inc(),
        MqttPacket::PubAck(_, _) => PACKETS_PUBACK_SENT
            .with_label_values(&[&network_type, &qos_str])
            .inc(),
        MqttPacket::PubRec(_, _) => PACKETS_PUBREC_SENT
            .with_label_values(&[&network_type, &qos_str])
            .inc(),
        MqttPacket::PubRel(_, _) => PACKETS_PUBREL_SENT
            .with_label_values(&[&network_type, &qos_str])
            .inc(),
        MqttPacket::PubComp(_, _) => PACKETS_PUBCOMP_SENT
            .with_label_values(&[&network_type, &qos_str])
            .inc(),
        MqttPacket::SubAck(_, _) => PACKETS_SUBACK_SENT
            .with_label_values(&[&network_type, &qos_str])
            .inc(),
        MqttPacket::UnsubAck(_, _) => PACKETS_UNSUBACK_SENT
            .with_label_values(&[&network_type, &qos_str])
            .inc(),
        MqttPacket::PingResp(_) => PACKETS_PINGRESP_SENT
            .with_label_values(&[&network_type, &qos_str])
            .inc(),
        MqttPacket::Disconnect(_, _) => PACKETS_DISCONNECT_SENT
            .with_label_values(&[&network_type, &qos_str])
            .inc(),
        MqttPacket::Auth(_, _) => PACKETS_CONNACK_SENT
            .with_label_values(&[&network_type, &qos_str])
            .inc(),
        _ => unreachable!("This branch only matches for packets could not be sent"),
    }
}

pub fn record_retain_recv_metrics(qos: QoS) {
    let qos_str = (qos as u8).to_string();
    RETAIN_PACKETS_RECEIVED.with_label_values(&[&qos_str]).inc();
}

pub fn record_retain_sent_metrics(qos: QoS) {
    let qos_str = (qos as u8).to_string();
    RETAIN_PACKETS_SEND.with_label_values(&[&qos_str]).inc();
}

#[cfg(test)]
mod tests {
    use protocol::mqtt::codec::{calc_mqtt_packet_size, MqttPacketWrapper};
    use protocol::mqtt::common::{MqttPacket, UnsubAck};

    #[test]
    fn calc_mqtt_packet_size_test() {
        let unsub_ack = UnsubAck {
            pkid: 1,
            reasons: Vec::new(),
        };

        let packet = MqttPacket::UnsubAck(unsub_ack, None);
        let packet_wrapper = MqttPacketWrapper {
            protocol_version: 4,
            packet,
        };
        assert_eq!(calc_mqtt_packet_size(packet_wrapper), 4);
    }
}
