use super::aws;
use super::config::{Auth, AuthConfig, Cluster, ClusterConfig, Config, Context, Release};
use super::kubeconfig;
use base64;
use std::io::Write;
use tempfile::NamedTempFile;

pub fn prepare(config: &Config, release: &Release) -> anyhow::Result<()> {
    let context = config.find_context(&release)?;
    log::debug!("Looking for context {}", context.name);
    let context_exists = kubeconfig::context_exists(&context)?;

    if context_exists {
        log::info!("Using existing context: {}", context.name);
        Ok(())
    } else {
        create_context(&context, &config)
    }
}

fn create_context(context: &Context, config: &Config) -> anyhow::Result<()> {
    ensure_auth(&context, &config)?;
    ensure_cluster(&context, &config)?;
    log::info!("Creating context: {}", context.name);
    kubeconfig::create_context(
        &context.name,
        &context.name,
        &context.cluster,
        &context.namespace,
    )?;
    log::debug!("Context successfully configured");
    Ok(())
}

fn ensure_auth(context: &Context, config: &Config) -> anyhow::Result<()> {
    log::debug!("Checking Kubernetes credentials for {}", context.name);
    let exists = kubeconfig::auth_exists(&context.name)?;

    if exists {
        log::debug!("Using existing Kubernetes credentials");
        Ok(())
    } else {
        log::debug!("Adding credentials to configuration");
        let auth = config.find_auth(context)?;
        let cluster = config.find_cluster(context)?;
        create_auth(context, auth, cluster)
    }
}

fn create_auth(context: &Context, auth: &Auth, cluster: &Cluster) -> anyhow::Result<()> {
    log::debug!("Configuring credentials: {:?}", auth);
    match auth.config {
        AuthConfig::AwsSso { .. } => match &cluster.config {
            ClusterConfig::Eks { name, region } => {
                log::info!(
                    "Setting Kubernetes credentials for EKS cluster: {} as {} in {}",
                    name,
                    context.name,
                    region
                );
                kubeconfig::create_auth(
                    &context.name,
                    &[
                        "--exec-api-version",
                        "client.authentication.k8s.io/v1alpha1",
                        "--exec-arg",
                        "--region",
                        "--exec-arg",
                        region,
                        "--exec-arg",
                        "eks",
                        "--exec-arg",
                        "get-token",
                        "--exec-arg",
                        "--cluster-name",
                        "--exec-arg",
                        name,
                        "--exec-command",
                        "aws",
                        "--exec-env",
                        &format!("AWS_PROFILE={}", auth.name),
                    ],
                )?;
                log::debug!("Credentials configured");
                Ok(())
            }
        },
    }
}

fn ensure_cluster(context: &Context, config: &Config) -> anyhow::Result<()> {
    log::debug!("Checking for Kubernetes cluster");
    let exists = kubeconfig::cluster_exists(&context.cluster)?;

    if exists {
        log::debug!("Cluster already configured");
        Ok(())
    } else {
        log::debug!("Adding cluster to configuration");
        let cluster = config.find_cluster(context)?;
        let auth = config.find_auth(context)?;
        create_cluster(cluster, auth)?;
        log::debug!("Cluster configured");
        Ok(())
    }
}

fn create_cluster(cluster: &Cluster, auth: &Auth) -> anyhow::Result<()> {
    match &cluster.config {
        ClusterConfig::Eks { name, region } => {
            log::info!(
                "Fetching Kubernetes cluster details for EKS cluster: {} as {} in {}",
                name,
                auth.name,
                region
            );
            let eks_cluster = aws::get_eks_cluster(&auth.name, region, name)?;
            let ca_pem = base64::decode(&eks_cluster.cert)?;
            let mut ca_file = NamedTempFile::new()?;
            ca_file.write(&ca_pem)?;
            let ca_path = ca_file.into_temp_path();
            let ca_path_name = ca_path.to_str().unwrap();
            log::info!(
                "Setting Kubernetes cluster details for cluster: {}",
                cluster.name
            );
            kubeconfig::create_cluster(
                &cluster.name,
                &[
                    "--embed-certs",
                    "--server",
                    &eks_cluster.endpoint,
                    "--certificate-authority",
                    ca_path_name,
                ],
            )?;
            ca_path.close()?;
            Ok(())
        }
    }
}
