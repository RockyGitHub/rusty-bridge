use serde::Serialize;
use sysinfo::{Components, CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

#[derive(Serialize)]
pub struct DynamicSystemMetrics {
    machine_id: String,
    processor_use: f32,
    memory_total: u64,
    memory_available: u64,
    temperature: f32,
}

impl DynamicSystemMetrics {
    pub fn new(machine_id: String) -> crate::Result<DynamicSystemMetrics> {
        let mut sys = System::new_all();
        let processor = sys.global_cpu_info();
        let processor_use = processor.cpu_usage();
        sys.refresh_memory();
        let memory_total = sys.total_memory();
        let memory_available = sys.available_memory();
        let temperature = match Components::new_with_refreshed_list().first() {
            Some(component) => component.temperature(),
            None => f32::MIN,
        };

        let metrics = DynamicSystemMetrics {
            machine_id,
            processor_use,
            memory_total,
            memory_available,
            temperature,
        };

        Ok(metrics)
    }

    pub fn update(&mut self) {
        let cpu_kind = CpuRefreshKind::new().with_cpu_usage();
        let memory_kind = MemoryRefreshKind::new().with_ram();
        let refresh_kind = RefreshKind::new()
            .with_cpu(cpu_kind)
            .with_memory(memory_kind);
        let sys = System::new_with_specifics(refresh_kind);
        let processor_use = sys.global_cpu_info().cpu_usage();
        let memory_total = sys.total_memory();
        let memory_available = sys.available_memory();
        let temperature = match Components::new_with_refreshed_list().first() {
            Some(component) => component.temperature(),
            None => f32::MIN,
        };

        self.processor_use = processor_use;
        self.memory_total = memory_total;
        self.memory_available = memory_available;
        self.temperature = temperature;
    }
}
