use super::kubectl;
use std::collections::HashMap;

#[derive(Debug)]
pub struct KubeClient {
    context: String,
}

#[derive(Debug)]
pub struct Pod<'a> {
    context: &'a String,
    name: String,
}

#[derive(Debug)]
pub struct Selector {
    labels: HashMap<String, String>,
}

pub fn new(context: &str) -> KubeClient {
    KubeClient {
        context: String::from(context),
    }
}

impl KubeClient {
    pub fn get_available_pod(&self, selector: Selector) -> anyhow::Result<Pod> {
        let output = kubectl::run_get_output(&[
            "--context",
            &self.context,
            "get",
            "pod",
            "--selector",
            &selector.to_string(),
            "--field-selector",
            "status.phase=Running",
            "--output",
            "name",
        ])?;
        let pod_name = String::from_utf8(output.stdout)?;
        Ok(Pod {
            context: &self.context,
            name: String::from(pod_name.trim()),
        })
    }

    pub fn run_command<S>(&self, command: &Vec<S>) -> anyhow::Result<()>
    where
        S: AsRef<str>,
    {
        kubectl::run_print(
            &[
                vec!["--context", &self.context],
                command.iter().map(|s| s.as_ref()).collect(),
            ]
            .concat(),
        )
    }
}

impl Selector {
    pub fn new(labels: HashMap<String, String>) -> Selector {
        Selector { labels: labels }
    }

    pub fn extend(&self, other: &Selector) -> Selector {
        Selector {
            labels: self
                .labels
                .clone()
                .into_iter()
                .chain(other.labels.clone())
                .collect(),
        }
    }

    pub fn to_string(&self) -> String {
        self.labels
            .iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect::<Vec<String>>()
            .join(",")
    }
}

impl Pod<'_> {
    pub fn exec<S>(&self, container: &str, command: &Vec<S>) -> anyhow::Result<()>
    where
        S: AsRef<str>,
    {
        kubectl::run_print(
            &[
                vec![
                    "--context",
                    &self.context,
                    "exec",
                    "--stdin",
                    "--tty",
                    &self.name,
                    "--container",
                    container,
                    "--",
                ],
                command.iter().map(|s| s.as_ref()).collect(),
            ]
            .concat(),
        )
    }
}
