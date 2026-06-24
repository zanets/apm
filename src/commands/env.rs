use crate::config;

pub fn run() -> anyhow::Result<()> {
    let config_dir = config::config_dir();
    let data_dir = config::data_dir();

    println!("config    {}", config_dir.display());
    println!("data      {}", data_dir.display());
    println!("store     {}", data_dir.join("store").display());
    println!("claudemds {}", config::claudemds_dir().display());
    Ok(())
}
