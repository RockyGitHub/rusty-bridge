use edge_reporter::ReporterConfig;

pub fn get_edge_reporter(tmp: String) -> mini_config_core::Result<ReporterConfig> {
    // TODO

    // TODO - hardcoded values
    let reporter_config = ReporterConfig {
        endpoint: "http://127.0.0.1:8999/edge_report".to_string(),
        system_name: "MySystem".to_string(),
        interval_s: 60,
    };

    Ok(reporter_config)
}
