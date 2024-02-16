use std::net::SocketAddr;

use crate::{config::*, process::VisionData};
use network_tables::v4::*;
use network_tables::Value::*;

pub(crate) struct NT {
    client: Client,
    detected_topic: PublishedTopic,
    tag_id_topic: PublishedTopic,
    timestamp_topic: PublishedTopic,
    rot_topic: PublishedTopic,
    transform_topic: PublishedTopic,
}

impl NT {
    pub(crate) async fn new(config: NetworkConfig) -> NT {
        let ip = SocketAddr::from((config.nt_ip, config.nt_port));
        let client = client::Client::new(ip).await;

        let detected_topic = client
            .publish_topic(
                "Vision/Detected",
                Type::Boolean,
                Some(PublishProperties::default()),
            )
            .await
            .unwrap();
        let tag_id_topic = client
            .publish_topic(
                "Vision/TagID",
                Type::Int,
                Some(PublishProperties::default()),
            )
            .await
            .unwrap();
        let timestamp_topic = client
            .publish_topic(
                "Vision/Timestamp",
                Type::Float,
                Some(PublishProperties::default()),
            )
            .await
            .unwrap();
        let rot_topic = client
            .publish_topic(
                "Vision/Rotation",
                Type::DoubleArray,
                Some(PublishProperties::default()),
            )
            .await
            .unwrap();
        let transform_topic = client
            .publish_topic(
                "Vision/Translation",
                Type::DoubleArray,
                Some(PublishProperties::default()),
            )
            .await
            .unwrap();

        NT {
            client,
            detected_topic,
            tag_id_topic,
            timestamp_topic,
            rot_topic,
            transform_topic,
        }
    }

    pub(crate) async fn publish(&mut self, data: VisionData) {
        let _ = self
            .client
            .publish_value(&self.detected_topic, &Boolean(data.detected))
            .await;
        let _ = self
            .client
            .publish_value(&self.tag_id_topic, &Integer(data.tag_id.into()))
            .await;
        let _ = self
            .client
            .publish_value(&self.timestamp_topic, &F32(0.0))
            .await;
        let _ = self
            .client
            .publish_value(
                &self.rot_topic,
                &Array(vec![
                    F64(data.rotation[0]),
                    F64(data.rotation[1]),
                    F64(data.rotation[2]),
                ]),
            )
            .await;
        let _ = self
            .client
            .publish_value(
                &self.transform_topic,
                &Array(vec![
                    F64(data.translation[0]),
                    F64(data.translation[1]),
                    F64(data.translation[2]),
                ]),
            )
            .await;
    }
}
