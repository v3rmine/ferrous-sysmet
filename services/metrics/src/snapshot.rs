use std::collections::HashMap;

use ::psutil::{
    cpu::{cpu_times_percpu, CpuTimes},
    disk::{DiskIoCounters, DiskIoCountersCollector},
    memory::{swap_memory, virtual_memory, SwapMemory, VirtualMemory},
    network::{NetIoCounters, NetIoCountersCollector},
    sensors::{temperatures, TemperatureSensor},
};
use chrono::{DateTime, Utc};
use log::{debug, tracing};
use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapShot {
    pub cpus: Vec<CpuTimes>,
    pub memory: VirtualMemory,
    pub swap: SwapMemory,
    pub networks: Vec<NetIoCounters>,
    pub disks: HashMap<String, DiskIoCounters>,
    pub temps: Vec<TemperatureSensor>,
    pub load_avgs: crate::psutil::LoadAvg,
    pub time: DateTime<Utc>,
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

    #[tracing::instrument(skip(self))]
    pub fn get_cpu_time(&self) -> (f64, f64) {
        let result = self.cpus.iter().fold((0.0, 0.0), |(busy, total), cpu| {
            (
                busy + cpu.busy().as_secs_f64(),
                total + cpu.total().as_secs_f64(),
            )
        });
        debug!(active_time = result.0, total_time = result.1);
        result
    }

    pub fn try_default() -> Result<Self> {
        Self::new(&[])
    }
}
