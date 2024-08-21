use mini_config_core::MiniConfigInterface;
use mini_config_core::Result;

//#[cfg(feature = "dev")]
pub fn new_config() -> Result<impl MiniConfigInterface> {
    #[cfg(feature = "dev")]
    let fetcher = mini_config_dev::MiniConfigDev::new()?;

    #[cfg(feature = "special")]
    let fetcher = mini_config_special::MiniConfigSpecial::new()?;

    Ok(fetcher)
}

//#[cfg(feature = "special")]
//pub fn new_config() -> Result<impl MiniConfigInterface> {
//let fetcher = mini_config_special::Special(MiniConfigSpecial::new()?);

//Ok(fetcher)
//}
