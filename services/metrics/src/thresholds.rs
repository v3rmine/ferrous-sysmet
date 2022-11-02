use std::time::Duration;

use crate::{psutil::LoadAvg, Result};

use log::{debug, trace, tracing};
use psutil::{
    cpu::{cpu_count, CpuPercentCollector},
    disk::disk_usage,
    memory::{swap_memory, virtual_memory},
};

#[tracing::instrument(level = "debug")]
pub fn load_avg_percent() -> Result<(f64, f64, f64)> {
    let cpu_count = cpu_count() as f64;
    trace!(cpu_count = cpu_count as u32);
    let LoadAvg { one, five, fifteen } = LoadAvg::new()?;
    let load_percent = |load: f64| load / cpu_count;
    let result = (load_percent(one), load_percent(five), load_percent(fifteen));
    debug!(
        one_percent = result.0,
        fice_percent = result.1,
        fifteen_percent = result.2,
        "Calculated Average Load"
    );
    Ok(result)
}

// NOTE: Recommended by python psutil (0.1s = 100ms)
// SOURCE: https://psutil.readthedocs.io/en/latest/#psutil.cpu_percent
const CPU_USAGE_INTERVAL: u64 = 100;

#[tracing::instrument(level = "debug")]
pub fn cpu_usage_percent() -> Result<f32> {
    let mut collector = CpuPercentCollector::new()?;
    trace!(
        "Sleeping for {CPU_USAGE_INTERVAL}ms waiting to have an interval to calculate CPU usage"
    );
    // NOTE: Needed because CPU usage must be calculated on an interval
    std::thread::sleep(Duration::from_millis(CPU_USAGE_INTERVAL));
    let result = collector.cpu_percent()?;
    debug!(cpu_usage_percent = result, "Calculated CPU usage");
    Ok(result)
}

#[tracing::instrument(level = "debug")]
pub fn memory_usage_percent() -> Result<(f32, f32)> {
    let swap = swap_memory()?.percent();
    let ram = virtual_memory()?.percent();
    debug!(
        swap_usage_percent = swap,
        ram_usage_percent = ram,
        "Calculated Memory usage"
    );
    Ok((ram, swap))
}

#[tracing::instrument(level = "debug")]
pub fn disk_usage_percent() -> Result<f32> {
    let result = disk_usage("/")?.percent();
    debug!(disk_usage_percent = result, "Calculated disk usage");
    Ok(result)
}
