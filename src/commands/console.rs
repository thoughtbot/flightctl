use crate::flightctl::kubeclient;
use crate::flightctl::{ApplicationConfig, Config, Console, Release};

pub fn run_default(config: &Config, release: &Release) -> anyhow::Result<()> {
    let application = config.find_application(&release)?;

    match &application.config {
        ApplicationConfig::Kubectl { console, .. } => match console {
            Some(Console::Exec { command, .. }) => run_command(config, release, command),
            None => Err(anyhow::Error::msg(format!(
                "No console configured for application: {}",
                application.name
            ))),
        },
    }
}

pub fn run_command(config: &Config, release: &Release, cmd: &Vec<String>) -> anyhow::Result<()> {
    let application = config.find_application(&release)?;

    match &application.config {
        ApplicationConfig::Kubectl { console, selector } => {
            let client = kubeclient::new(&release.context);
            let base_selector = kubeclient::Selector::new(selector.clone());

            match console {
                Some(Console::Exec {
                    container,
                    selector,
                    ..
                }) => {
                    let console_selector =
                        base_selector.extend(&kubeclient::Selector::new(selector.clone()));
                    let pod = client.get_available_pod(console_selector)?;
                    client.exec(&pod, container, cmd)?;
                    Ok(())
                }
                None => Err(anyhow::Error::msg(format!(
                    "No console configured for application: {}",
                    application.name
                ))),
            }
        }
    }
}
