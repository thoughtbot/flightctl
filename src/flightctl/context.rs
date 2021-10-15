use super::aws;
use super::config::{Auth, AuthConfig, Cluster, ClusterConfig, Config, Context, Release};
use super::kubectl;
use base64;
use std::io::Write;
use tempfile::NamedTempFile;

pub fn prepare(config: &Config, release: &Release) -> anyhow::Result<()> {
    let context_exists = kubectl::context_exists(&release.context)?;

    if context_exists {
        Ok(())
    } else {
        let context = config.find_context(&release)?;
        create_context(&context, &config)
    }
}

fn create_context(context: &Context, config: &Config) -> anyhow::Result<()> {
    ensure_auth(&context, &config)?;
    ensure_cluster(&context, &config)?;
    eprintln!("Creating Kubernetes context: {}", context.name);
    kubectl::create_context(
        &context.name,
        &context.auth,
        &context.cluster,
        &context.namespace,
    )
}

fn ensure_auth(context: &Context, config: &Config) -> anyhow::Result<()> {
    let exists = kubectl::auth_exists(&context.auth)?;

    if exists {
        Ok(())
    } else {
        let auth = config.find_auth(context)?;
        let cluster = config.find_cluster(context)?;
        create_auth(auth, cluster)
    }
}

fn create_auth(auth: &Auth, cluster: &Cluster) -> anyhow::Result<()> {
    match auth.config {
        AuthConfig::AwsSso { .. } => match &cluster.config {
            ClusterConfig::Eks { name, region } => {
                eprintln!(
                    "Setting Kubernetes credentials for EKS cluster: {} as {} in {}",
                    name, auth.name, region
                );
                kubectl::create_auth(
                    &auth.name,
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
                )
            }
        },
    }
}

fn ensure_cluster(context: &Context, config: &Config) -> anyhow::Result<()> {
    let exists = kubectl::cluster_exists(&context.cluster)?;

    if exists {
        Ok(())
    } else {
        let cluster = config.find_cluster(context)?;
        let auth = config.find_auth(context)?;
        create_cluster(cluster, auth)
    }
}

fn create_cluster(cluster: &Cluster, auth: &Auth) -> anyhow::Result<()> {
    match &cluster.config {
        ClusterConfig::Eks { name, region } => {
            eprintln!(
                "Fetching Kubernetes cluster details for EKS cluster: {} as {} in {}",
                name, auth.name, region
            );
            let eks_cluster = aws::get_eks_cluster(&auth.name, region, name)?;
            let ca_pem = base64::decode(&eks_cluster.cert)?;
            let mut ca_file = NamedTempFile::new()?;
            ca_file.write(&ca_pem)?;
            let ca_path = ca_file.into_temp_path();
            let ca_path_name = ca_path.to_str().unwrap();
            eprintln!(
                "Setting Kubernetes cluster details for cluster: {}",
                cluster.name
            );
            kubectl::create_cluster(
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
