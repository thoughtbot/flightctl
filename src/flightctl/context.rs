use super::aws;
use super::config::{Auth, AuthConfig, Cluster, ClusterConfig, Config, Context, Release};
use super::kubeconfig_writer;
use kube::config::{
    AuthInfo, ExecConfig, Kubeconfig, KubeconfigError, NamedAuthInfo, NamedCluster,
};
use std::collections::HashMap;

pub fn prepare(config: &Config, release: &Release) -> anyhow::Result<()> {
    let context = config.find_context(&release)?;
    let auth = config.find_auth(context)?;
    let cluster = config.find_cluster(context)?;
    let kubeauth = build_auth(context, auth, cluster);
    let kubecluster = build_cluster(cluster, auth)?;

    let kubeconfig = read_kubeconfig()?;
    log::debug!(
        "Loaded Kubernetes configuration successfully: {:?}",
        kubeconfig
    );
    ensure_auth(&kubeconfig, kubeauth)?;
    ensure_cluster(&kubeconfig, kubecluster)?;
    ensure_context(&kubeconfig, &context)
}

fn read_kubeconfig() -> anyhow::Result<Kubeconfig> {
    log::debug!("Loading Kubernetes configuration");
    match Kubeconfig::read() {
        Ok(config) => anyhow::Ok(config),
        Err(KubeconfigError::ReadConfig { .. }) => anyhow::Ok(empty_config()),
        Err(err) => Err(anyhow::anyhow!(err)),
    }
}

fn empty_config() -> Kubeconfig {
    Kubeconfig {
        auth_infos: vec![],
        kind: None,
        api_version: None,
        clusters: vec![],
        contexts: vec![],
        current_context: None,
        extensions: None,
        preferences: None,
    }
}

fn ensure_context(config: &Kubeconfig, expected: &Context) -> anyhow::Result<()> {
    log::debug!("Checking Kubernetes context {}", &expected.name);

    let exists = config
        .contexts
        .iter()
        .find(|actual| {
            &actual.name == &expected.name
                && &actual.context.cluster == &expected.cluster
                && &actual.context.user == &expected.name
                && actual.context.namespace.as_deref() == Some(&expected.namespace)
        })
        .is_some();

    if exists {
        log::debug!("Using existing Kubenetes context");
        Ok(())
    } else {
        log::info!("Writing Kubernetes context: {}", expected.name);
        kubeconfig_writer::write_context(
            &expected.name,
            &expected.name,
            &expected.cluster,
            &expected.namespace,
        )
    }
}

fn ensure_auth(config: &Kubeconfig, expected: NamedAuthInfo) -> anyhow::Result<()> {
    log::debug!("Checking Kubernetes credentials for {}", &expected.name);

    let exists = config
        .auth_infos
        .iter()
        .find(|actual| {
            expected.name == actual.name
                && expected.auth_info.client_certificate_data
                    == actual.auth_info.client_certificate_data
                && expected
                    .auth_info
                    .exec
                    .as_ref()
                    .map(|exec| (&exec.api_version, &exec.args, &exec.command, &exec.args))
                    == actual
                        .auth_info
                        .exec
                        .as_ref()
                        .map(|exec| (&exec.api_version, &exec.args, &exec.command, &exec.args))
        })
        .is_some();

    if exists {
        log::debug!("Using existing Kubernetes credentials");
        Ok(())
    } else {
        log::info!("Writing Kubernetes credentials for {}", &expected.name);
        kubeconfig_writer::write_auth(expected)
    }
}

fn build_auth(context: &Context, auth: &Auth, cluster: &Cluster) -> NamedAuthInfo {
    let mut result = AuthInfo {
        auth_provider: None,
        client_certificate: None,
        client_certificate_data: None,
        client_key: None,
        client_key_data: None,
        exec: None,
        impersonate: None,
        impersonate_groups: None,
        password: None,
        token: None,
        token_file: None,
        username: None,
    };

    log::debug!("Configuring credentials: {:?}", auth);
    match auth.config {
        AuthConfig::AwsSso { .. } => match &cluster.config {
            ClusterConfig::Eks { name, region } => {
                log::info!(
                    "Using AWS profile {} for EKS cluster {} in region {}",
                    auth.name,
                    name,
                    region
                );

                let mut env = HashMap::new();
                env.insert(String::from("name"), String::from("AWS_PROFILE"));
                env.insert(String::from("value"), auth.name.clone());

                result.exec = Some(ExecConfig {
                    api_version: Some(String::from("client.authentication.k8s.io/v1alpha1")),
                    args: Some(vec![
                        String::from("--region"),
                        region.to_string(),
                        String::from("eks"),
                        String::from("get-token"),
                        String::from("--cluster-name"),
                        name.to_string(),
                    ]),
                    command: String::from("aws"),
                    env: Some(vec![env]),
                })
            }
        },
    }

    NamedAuthInfo {
        name: context.name.clone(),
        auth_info: result,
    }
}

fn ensure_cluster(config: &Kubeconfig, expected: NamedCluster) -> anyhow::Result<()> {
    log::debug!("Checking for Kubernetes cluster");

    let exists = config
        .clusters
        .iter()
        .find(|actual| {
            &actual.name == &expected.name
                && &actual.cluster.server == &expected.cluster.server
                && &actual.cluster.certificate_authority_data
                    == &expected.cluster.certificate_authority_data
        })
        .is_some();

    if exists {
        log::debug!("Cluster already configured");
        Ok(())
    } else {
        log::info!("Writing cluster {}", &expected.name);
        kubeconfig_writer::write_cluster(expected)
    }
}

fn build_cluster(cluster: &Cluster, auth: &Auth) -> anyhow::Result<NamedCluster> {
    match &cluster.config {
        ClusterConfig::Eks { name, region } => {
            log::debug!(
                "Fetching Kubernetes cluster details for EKS cluster: {} as {} in {}",
                name,
                auth.name,
                region
            );
            let eks_cluster = aws::get_eks_cluster(&auth.name, region, name)?;
            Ok(NamedCluster {
                name: cluster.name.clone(),
                cluster: kube::config::Cluster {
                    certificate_authority: None,
                    certificate_authority_data: Some(eks_cluster.cert),
                    extensions: None,
                    insecure_skip_tls_verify: None,
                    proxy_url: None,
                    server: eks_cluster.endpoint,
                },
            })
        }
    }
}
