use crate::flightctl::kubeclient;
use crate::flightctl::{ApplicationConfig, Config, Release};

pub fn run(config: &Config, release: &Release) -> anyhow::Result<()> {
    let application = config.find_application(&release)?;

    match &application.config {
        ApplicationConfig::Kubectl { selector, .. } => {
            let client = kubeclient::new(&release.context);
            client.get_workloads(kubeclient::Selector::new(selector.clone()))
        }
    }
}
