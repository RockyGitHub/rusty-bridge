use machineid_rs::{Encryption, HWIDComponent};
use serde::Serialize;
use sysinfo::System;

use crate::Error;

const MACHINE_ID_KEY: &str = "edgereports";

#[derive(Serialize)]
pub struct StaticSystemMetrics {
    machine_id: String,
    processor_serial: String,
    system_name: String,
    local_ip: String,
}

impl StaticSystemMetrics {
    pub fn new(system_name: String) -> crate::Result<StaticSystemMetrics> {
        let local_ip = "127.0.0.1".to_string();
        //let machine_id = machineid_rs::IdBuilder::new(Encryption::SHA256).add_component(HWIDComponent::CPUID)
        let sys = System::new_all();
        let processor = sys.global_cpu_info();
        let processor_serial = processor.vendor_id().to_string();
        let machine_id = machineid_rs::IdBuilder::new(Encryption::MD5)
            .add_component(HWIDComponent::CPUID)
            .add_component(HWIDComponent::Username)
            .add_component(HWIDComponent::MacAddress)
            .build(MACHINE_ID_KEY)
            .map_err(|err| Error::Initialization(err.to_string()))?;

        let metrics = StaticSystemMetrics {
            machine_id,
            system_name,
            local_ip: local_ip,
            processor_serial,
        };

        Ok(metrics)
    }

    pub fn machine_id(&self) -> &str {
        &self.machine_id
    }
}
