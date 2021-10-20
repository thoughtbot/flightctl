use super::config::Context;
use super::kubectl;
use kube::config::Kubeconfig;

pub fn context_exists(context: &Context) -> anyhow::Result<bool> {
    let config = Kubeconfig::read()?;
    Ok(config
        .contexts
        .into_iter()
        .find(|named_context| {
            &named_context.name == &context.name
                && &named_context.context.cluster == &context.cluster
                && &named_context.context.user == &context.name
                && named_context.context.namespace.as_deref() == Some(&context.namespace)
        })
        .is_some())
}

pub fn create_context(
    name: &str,
    auth: &str,
    cluster: &str,
    namespace: &str,
) -> anyhow::Result<()> {
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

pub fn auth_exists(name: &str) -> anyhow::Result<bool> {
    let result =
        kubectl::run_get_output(&["config", "view", "--output", "jsonpath={.users[*].name}"])?;
    let output = String::from_utf8(result.stdout)?;
    Ok(output
        .split_whitespace()
        .find(|elem| elem == &name)
        .is_some())
}

pub fn create_auth(name: &str, args: &[&str]) -> anyhow::Result<()> {
    kubectl::run_print(&[&["config", "set-credentials", name], args].concat())
}

pub fn cluster_exists(name: &str) -> anyhow::Result<bool> {
    let result = kubectl::run_get_output(&["config", "get-clusters"])?;
    let output = String::from_utf8(result.stdout)?;
    Ok(output
        .split_whitespace()
        .find(|elem| elem == &name)
        .is_some())
}

pub fn create_cluster(name: &str, args: &[&str]) -> anyhow::Result<()> {
    kubectl::run_print(&[&["config", "set-cluster", name], args].concat())
}
