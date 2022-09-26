use std::collections::HashMap;

use ::psutil::{
    cpu::{cpu_times_percpu, CpuTimes},
    disk::{DiskIoCounters, DiskIoCountersCollector},
    memory::{swap_memory, virtual_memory, SwapMemory, VirtualMemory},
    network::{NetIoCounters, NetIoCountersCollector},
    sensors::{temperatures, TemperatureSensor},
};
use chrono::{DateTime, Utc};
use log::tracing;
use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapShot {
    cpus: Vec<CpuTimes>,
    memory: VirtualMemory,
    swap: SwapMemory,
    networks: Vec<NetIoCounters>,
    disks: HashMap<String, DiskIoCounters>,
    temps: Vec<TemperatureSensor>,
    load_avgs: crate::psutil::LoadAvg,
    time: DateTime<Utc>,
}

impl SnapShot {
    #[tracing::instrument]
    pub fn new(networks_to_ignore: &[&str]) -> Result<Self> {
        let result = Self {
            cpus: cpu_times_percpu()?,
            memory: virtual_memory()?,
            swap: swap_memory()?,
            networks: NetIoCountersCollector::default()
                .net_io_counters_pernic()?
                .into_iter()
                .filter_map(|(k, v)| {
                    if networks_to_ignore.contains(&k.as_str()) {
                        Some(v)
                    } else {
                        None
                    }
                })
                .collect(),
            disks: DiskIoCountersCollector::default().disk_io_counters_per_partition()?,
            temps: temperatures()
                .into_iter()
                .collect::<std::result::Result<Vec<TemperatureSensor>, _>>()?,
            load_avgs: crate::psutil::LoadAvg::new()?,
            time: Utc::now(),
        };

        log::trace!("Snapshot taken with data\n{:#?}", result);

        Ok(result)
    }

    pub fn try_default() -> Result<Self> {
        Self::new(&[])
    }
}
