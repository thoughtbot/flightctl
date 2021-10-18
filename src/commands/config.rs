use crate::flightctl::kubeclient;
use crate::flightctl::kubeenv;
use crate::flightctl::{ApplicationConfig, Config, Console, Release};

pub fn print(config: &Config, release: &Release) -> anyhow::Result<()> {
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
                    let container = pod
                        .spec
                        .map(|spec| spec.containers.into_iter().find(|c| &c.name == container))
                        .flatten()
                        .ok_or(anyhow::anyhow!("Couldn't find container {}", container))?;
                    let mut resolver = kubeenv::Resolver::new(&client);
                    let config = resolver.resolve(container);
                    for var in config {
                        match var.value {
                            kubeenv::ResolvedValue::Pod { value } => {
                                println!("{}: {} (from pod)", &var.name, &show_value(value))
                            }
                            kubeenv::ResolvedValue::ConfigMapKeyRef {
                                config_map,
                                key,
                                value,
                            } => println!(
                                "{}: {} (from configmap/{}.{})",
                                &var.name,
                                &show_value(value.map(|s| s.to_string())),
                                config_map,
                                key
                            ),
                            kubeenv::ResolvedValue::SecretKeyRef { secret, key } => println!(
                                "{}: ******************** (from secret/{}.{})",
                                &var.name, secret, key
                            ),
                            kubeenv::ResolvedValue::FieldRef { path } => {
                                println!("{}: (reference to {})", &var.name, &path)
                            }
                        }
                    }
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

fn show_value(value: Option<String>) -> String {
    value.unwrap_or(String::from("(unset)"))
}
