use std::process::{Command, ExitStatus, Output};

pub fn context_exists(name: &str) -> anyhow::Result<bool> {
    let result = run_kubectl(&["config", "get-contexts", "--output", "name"])?;
    let output = String::from_utf8(result.stdout)?;
    Ok(output
        .lines()
        .find(|line| line.trim_end() == name)
        .is_some())
}

pub fn create_context(
    name: &str,
    auth: &str,
    cluster: &str,
    namespace: &str,
) -> anyhow::Result<()> {
    run_kubectl_print(&[
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
    let result = run_kubectl(&["config", "view", "--output", "jsonpath={.users[*].name}"])?;
    let output = String::from_utf8(result.stdout)?;
    Ok(output
        .split_whitespace()
        .find(|elem| elem == &name)
        .is_some())
}

pub fn create_auth(name: &str, args: &[&str]) -> anyhow::Result<()> {
    run_kubectl_print(&[&["config", "set-credentials", name], args].concat())
}

pub fn cluster_exists(name: &str) -> anyhow::Result<bool> {
    let result = run_kubectl(&["config", "get-clusters"])?;
    let output = String::from_utf8(result.stdout)?;
    Ok(output
        .split_whitespace()
        .find(|elem| elem == &name)
        .is_some())
}

pub fn create_cluster(name: &str, args: &[&str]) -> anyhow::Result<()> {
    run_kubectl_print(&[&["config", "set-cluster", name], args].concat())
}

fn run_kubectl(args: &[&str]) -> anyhow::Result<Output> {
    let output = kubectl(args).output()?;
    match verify_exit(&args, output.status) {
        Ok(_) => Ok(output),
        Err(err) => {
            Err(err.context(String::from_utf8(output.stderr).unwrap_or("(binary)".to_string())))
        }
    }
}

fn run_kubectl_print(args: &[&str]) -> anyhow::Result<()> {
    let mut child = kubectl(&args).spawn()?;
    let status = child.wait()?;
    verify_exit(&args, status)
}

fn kubectl(args: &[&str]) -> Command {
    let mut command = Command::new("kubectl");
    command.args(args);
    command
}

fn verify_exit(args: &[&str], status: ExitStatus) -> anyhow::Result<()> {
    if status.success() {
        Ok(())
    } else {
        let command: Vec<&str> = args.to_vec();
        Err(anyhow::Error::msg(format!(
            "kubectl {}: Command exited unsuccessfully (status code {})",
            command.join(" "),
            status
                .code()
                .map(|code| code.to_string())
                .unwrap_or("unknown".to_string())
        )))
    }
}
