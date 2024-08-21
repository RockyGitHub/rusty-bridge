mod sequence;

use std::{
    io::Write,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use flate2::{write::GzEncoder, Compression};
use sequence::Sequence;
use sparkplug_rs::{payload::Metric, protobuf::Message, sparkplug_b};
use tracing::{debug, error, trace, warn};

pub struct TransformSpecialHiveMQ {
    seq: Sequence,
}

impl TransformSpecialHiveMQ {
    pub fn new() -> TransformSpecialHiveMQ {
        TransformSpecialHiveMQ {
            seq: Sequence::new(),
        }
    }

    // Ideally don't return a Vec<u8>, I don't want to force allocation
    /// Takes in an arbitrary slice of bytes.  This method will deserialize to the expected type and then serialize to the format expected by cloud
    pub fn transform(&mut self, data: Vec<u8>) -> Vec<u8> {
        // TODO - investigate what is actually needed here, most of the metrics are not

        // TODO - deserialize data.  Is it possible to avoid this and mandate a certain contract via the parameters passed in?
        // this would apply to the data-ingest to provide required meta data

        //trace!("Transforming data: [{:?}]", data);
        let mut sparkplug = sparkplug_b::Payload::new();

        // Metrics
        let mut metric_id = Metric::new();
        metric_id.set_name("id".to_string());
        metric_id.set_alias(0);
        metric_id.set_datatype(15);
        metric_id.set_is_historical(false);
        metric_id.set_is_transient(false);
        metric_id.set_bytes_value("b17af608-3a01-4a92-af7c-51a69468b302".as_bytes().to_vec());

        let mut metric_body_content_type = Metric::new();
        metric_body_content_type.set_name("bodyContentType".to_string());
        metric_body_content_type.set_datatype(12);
        metric_body_content_type.set_is_historical(false);
        metric_body_content_type.set_is_transient(false);
        metric_body_content_type.set_string_value("application/json".to_string());

        let mut metric_type = Metric::new();
        metric_type.set_name("type".to_string());
        metric_type.set_datatype(12);
        metric_type.set_is_historical(false);
        metric_type.set_is_transient(false);
        metric_type.set_string_value("data".to_string());

        let mut metric_class_type = Metric::new();
        metric_class_type.set_name("class".to_string());
        metric_class_type.set_datatype(12);
        metric_class_type.set_is_historical(false);
        metric_class_type.set_is_transient(false);
        metric_class_type.set_string_value("epic".to_string());

        //sparkplug.metrics.push(metric_id);
        //sparkplug.metrics.push(metric_body_content_type);
        //sparkplug.metrics.push(metric_class_type);
        sparkplug.metrics.push(metric_type);

        let body = if false {
            sparkplug.set_uuid("COMPRESSED".to_string());

            let mut metric_compression = Metric::new();
            metric_compression.set_datatype(12);
            metric_compression.set_is_historical(false);
            metric_compression.set_is_transient(false);
            metric_compression.set_name("algorithm".to_string());
            metric_compression.set_string_value("GZIP".to_string());
            sparkplug.metrics.push(metric_compression);

            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&data).unwrap();
            encoder.finish().unwrap()
        } else {
            data
        };

        //sparkplug.set_uuid(v)
        sparkplug.set_body(body);
        sparkplug.set_seq(self.seq.pull_seq());
        sparkplug.set_timestamp(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|err| {
                    error!("System time is invalid, sparkplug timestamp will be set to 0. [{}]", err);
                    Duration::from_secs(0)
                })
                .as_millis().try_into().unwrap_or_else(|err|{
                    error!("Could not convert system UTC epoch time to u64 for sparkplugb protocol. [{}]", err);
                    0
                }),
        );

        let mut msg: Vec<u8> = Vec::new();
        sparkplug.write_to_vec(&mut msg).unwrap();

        msg
    }
}

#[test]
fn to_and_from() {
    // TODO
    let msg: [u8; 0] = [5; 0];
    let test = sparkplug_b::Payload::parse_from_bytes(&msg);
}
