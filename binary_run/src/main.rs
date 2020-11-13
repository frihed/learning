use anyhow::Result;
use basis::web_server_tide;
use log::*;

#[async_std::main]
async fn main() -> Result<()>{
    log4rs::init_file("log4rs.yml", Default::default())?;
    info!("learning start ...");
    web_server_tide::start().await
}