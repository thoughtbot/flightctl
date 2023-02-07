use super::kubectl;
use kube::config::{NamedAuthInfo, NamedCluster};
use std::io::Write;
use tempfile::NamedTempFile;

pub fn write_auth(auth: NamedAuthInfo) -> anyhow::Result<()> {
    let mut args = vec![
        String::from("config"),
        String::from("set-credentials"),
        auth.name,
    ];
    if let Some(auth_info) = auth.auth_info {
        if let Some(exec) = auth_info.exec {
            if let Some(command) = exec.command {
                args.push(String::from("--exec-command"));
                args.push(command);
            }

            if let Some(api_version) = exec.api_version {
                args.push(String::from("--exec-api-version"));
                args.push(api_version)
            }

            if let Some(exec_args) = exec.args {
                for arg in exec_args {
                    args.push(String::from("--exec-arg"));
                    args.push(arg);
                }
            }

            if let Some(env) = &exec.env {
                for env_map in env {
                    if let (Some(name), Some(value)) = (env_map.get("name"), env_map.get("value")) {
                        args.push(String::from("--exec-env"));
                        args.push(format!("{}={}", name, value));
                    }
                }
            }
        };
    };
    kubectl::run_print(&args)
}

pub fn write_context(name: &str, auth: &str, cluster: &str, namespace: &str) -> anyhow::Result<()> {
    kubectl::run_print(&[
        "config",
        "set-context",
        name,
        "--cluster",
        cluster,
        "--user",
        auth,
        "--namespace",
        namespace,
    ])
}

pub fn write_cluster(definition: NamedCluster) -> anyhow::Result<()> {
    let cluster = &definition.cluster.ok_or(anyhow::Error::msg(format!(
        "Missing cluster definition for {}",
        &definition.name
    )))?;
    let mut args = vec![
        String::from("config"),
        String::from("set-cluster"),
        definition.name,
    ];

    if let Some(server) = &cluster.server {
        args.push(String::from("--server"));
        args.push(String::from(server))
    }

    let mut ca_file = NamedTempFile::new()?;

    if let Some(ca_data_encoded) = &cluster.certificate_authority_data {
        let ca_data = base64::decode(&ca_data_encoded)?;
        ca_file.write(&ca_data)?;
    }

    let ca_path = ca_file.into_temp_path();
    if cluster.certificate_authority_data.is_some() {
        let ca_path_name = &ca_path.to_str().unwrap();
        args.push(String::from("--embed-certs"));
        args.push(String::from("--certificate-authority"));
        args.push(ca_path_name.to_string());
    }

    kubectl::run_print(&args)?;
    ca_path.close()?;
    Ok(())
}
