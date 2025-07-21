use actix_web_lab::sse;
use anyhow::Result;
use log::warn;
use serde::Serialize;
use sysinfo::{Disks, Networks, System};
use tokio_util::sync::CancellationToken;

#[derive(Serialize)]
pub struct StaticHostResourceData {
    pub os: String,
    pub num_cores: usize,
    pub total_memory: u64,
}

#[derive(Serialize)]
pub struct HostResourceData {
    pub cpu_usage: Option<CpuUsage>,
    pub allocated_memory: Option<u64>,
    pub disk_usage: Option<Vec<RWUsage>>,
    pub network_usage: Option<Vec<RWUsage>>,
}

#[derive(Serialize)]
pub struct CpuUsage {
    pub total_usage: f32,
    pub cores: Vec<f32>,
}

#[derive(Serialize)]
pub struct RWUsage {
    pub device: String,
    pub read: u64,
    pub write: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<u64>,
}

impl StaticHostResourceData {
    pub fn fetch() -> Self {
        let sys = System::new_all();
        let os = System::name().unwrap_or_else(|| "Unknown OS".to_string());
        let num_cores = sys.cpus().len();
        let total_memory = sys.total_memory();
        Self { os, num_cores, total_memory }
    }
}

impl HostResourceData {
    pub async fn fetch_continuously(sender: tokio::sync::mpsc::Sender<sse::Event>, cancellation_token: Option<CancellationToken>) -> Result<()> {
        let mut system = System::new_all();
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Burst);
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let data = Self::fetch(&mut system);
                    let message = sse::Data::new(serde_json::to_string(&data)?).event("resource_update");
                    if sender.send(message.into()).await.is_err() {
                        warn!("Failed to send resource update event to client, this is most likely due to the client disconnecting.");
                        break;
                    }
                }
                _ = async {
                    if let Some(token) = &cancellation_token {
                        token.cancelled().await;
                    }
                    std::future::pending::<()>().await
                } => {
                    break;
                }
            }
        }

        Ok(())
    }
    pub fn fetch(system: &mut System) -> Self {
        system.refresh_all();
        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();

        let cpu_usage = system.global_cpu_usage();
        let cores = system.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
        let allocated_memory = system.used_memory();

        let disk_usage = disks
            .iter()
            .map(|disk| {
                let device = disk.mount_point().to_str().unwrap_or("Unknown").to_string();
                let usage = disk.usage();
                let read = usage.read_bytes;
                let write = usage.written_bytes;

                RWUsage { device, read, write, mtu: None }
            })
            .collect();

        let network_usage = networks
            .iter()
            .map(|(device, data)| {
                let received_bytes = data.received();
                let sent_bytes = data.transmitted();
                let max_transfer_unit = data.mtu();
                RWUsage { device: device.to_string(), read: received_bytes, write: sent_bytes, mtu: Some(max_transfer_unit) }
            })
            .collect();

        Self {
            cpu_usage: Some(CpuUsage { total_usage: cpu_usage, cores }),
            allocated_memory: Some(allocated_memory),
            disk_usage: Some(disk_usage),
            network_usage: Some(network_usage),
        }
    }
}
