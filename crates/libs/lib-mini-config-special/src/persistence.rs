pub fn get_persistence(tmp: String) -> mini_config_core::Result<persistence_sled::Config> {
    Ok(persistence_sled::Config { _reserved: 0 })
}
