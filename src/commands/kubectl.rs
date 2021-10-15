use crate::flightctl::kubeclient;
use crate::flightctl::{ApplicationConfig, Config, Release};

pub fn run(config: &Config, release: &Release, cmd: &Vec<String>) -> anyhow::Result<()> {
    let application = config.find_application(&release)?;

    match &application.config {
        ApplicationConfig::Kubectl { .. } => {
            let client = kubeclient::new(&release.context);
            client.run_command(cmd)
        }
    }
}
