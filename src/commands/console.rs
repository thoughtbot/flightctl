use crate::flightctl::kubeclient;
use crate::flightctl::{ApplicationConfig, Config, Console, Release};

pub fn run(config: &Config, release: &Release, cmd: Vec<String>) -> anyhow::Result<()> {
    let application = config.find_application(&release)?;

    match &application.config {
        ApplicationConfig::Kubectl { console, selector } => {
            let client = kubeclient::new(&release.context);
            let base_selector = kubeclient::Selector::new(selector.clone());

            match console {
                Some(Console::Exec {
                    command,
                    container,
                    selector,
                }) => {
                    let console_selector =
                        base_selector.extend(&kubeclient::Selector::new(selector.clone()));
                    let pod = client.get_available_pod(console_selector)?;
                    pod.exec(&container, if cmd.is_empty() { command } else { &cmd })?;
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
