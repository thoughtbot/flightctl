use super::kubectl;
use k8s_openapi::api::core::v1 as k8s;
use std::collections::HashMap;

#[derive(Debug)]
pub struct KubeClient {
    context: String,
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
    pub fn get_available_pod(&self, selector: Selector) -> anyhow::Result<k8s::Pod> {
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
        let pod_names = String::from_utf8(output.stdout)?;
        let pod_name = pod_names
            .split_whitespace()
            .nth(0)
            .ok_or(anyhow::Error::msg("No console pod found"))?;
        let pod = self.fetch_resource(&pod_name.trim_end())?;
        Ok(pod)
    }

    pub fn get_workloads(&self, selector: Selector) -> anyhow::Result<()> {
        kubectl::run_print(&[
            "--context",
            &self.context,
            "get",
            "deploy",
            "--selector",
            &selector.to_string(),
        ])
    }

    pub fn exec<S>(&self, pod: &k8s::Pod, container: &str, command: &Vec<S>) -> anyhow::Result<()>
    where
        S: AsRef<str>,
    {
        let pod_name = pod.metadata.name.as_deref();
        kubectl::run_print(
            &[
                vec![
                    "--context",
                    &self.context,
                    "exec",
                    "--stdin",
                    "--tty",
                    &pod_name.unwrap_or_default(),
                    "--container",
                    container,
                    "--",
                ],
                command.iter().map(|s| s.as_ref()).collect(),
            ]
            .concat(),
        )
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

    pub fn fetch_resource<T>(&self, resource: &str) -> anyhow::Result<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let output = kubectl::run_get_output(&[
            "--context",
            &self.context,
            "get",
            &resource,
            "--output",
            "yaml",
        ])?;
        let yaml = String::from_utf8(output.stdout)?;
        let result = serde_yaml::from_str::<T>(&yaml)?;
        Ok(result)
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
