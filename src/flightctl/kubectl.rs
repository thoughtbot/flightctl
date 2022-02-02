use log;
use std::process::{Command, ExitStatus, Output};

pub fn run_get_output(args: &[&str]) -> anyhow::Result<Output> {
    let output = run(args).output()?;
    match verify_exit(&args, output.status) {
        Ok(_) => Ok(output),
        Err(err) => {
            Err(err.context(String::from_utf8(output.stderr).unwrap_or("(binary)".to_string())))
        }
    }
}

pub fn run_print(args: &[&str]) -> anyhow::Result<()> {
    let mut child = run(&args).spawn()?;
    let status = child.wait()?;
    verify_exit(&args, status)
}

fn run(args: &[&str]) -> Command {
    log::debug!("Running kubectl with {:?}", args);
    let mut command = Command::new("kubectl");
    command.args(args);
    command
}

fn verify_exit(args: &[&str], status: ExitStatus) -> anyhow::Result<()> {
    log::debug!("kubectl exited with {}", status);
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
