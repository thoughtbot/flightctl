use log;
use std::ffi::OsStr;
use std::fmt::Debug;
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

pub fn run_print<T: AsRef<OsStr> + Clone + Debug>(args: &[T]) -> anyhow::Result<()> {
    let mut child = run(args.as_ref()).spawn()?;
    let status = child.wait()?;
    verify_exit(args.as_ref(), status)
}

fn run<T: AsRef<OsStr> + Clone + Debug>(args: &[T]) -> Command {
    log::debug!("Running kubectl with {:?}", &args.to_vec());
    let mut command = Command::new("kubectl");
    command.args(args);
    command
}

fn verify_exit<T: AsRef<OsStr> + Clone + Debug>(
    args: &[T],
    status: ExitStatus,
) -> anyhow::Result<()> {
    log::debug!("kubectl exited with {}", status);
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::Error::msg(format!(
            "kubectl {:?}: Command exited unsuccessfully (status code {})",
            &args.to_vec(),
            status
                .code()
                .map(|code| code.to_string())
                .unwrap_or("unknown".to_string())
        )))
    }
}
