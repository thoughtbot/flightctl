use std::collections::HashMap;
use std::process::{Command, ExitStatus, Output};

pub fn profile_exists(profile: &str) -> anyhow::Result<bool> {
    let result = run_aws_cli(&["configure", "list-profiles"])?;
    let output = String::from_utf8(result.stdout)?;
    Ok(output
        .lines()
        .find(|line| line.trim_end() == profile)
        .is_some())
}

pub fn create_profile(profile: &str, config: &HashMap<String, String>) -> anyhow::Result<()> {
    eprintln!("Creating AWS profile: {}", profile);

    for (key, value) in config.iter() {
        run_aws_cli(&["--profile", profile, "configure", "set", key, value])?;
    }

    Ok(())
}

pub fn verify_auth(profile: &str) -> anyhow::Result<()> {
    run_aws_cli(&["--profile", profile, "sts", "get-caller-identity"]).and(Ok(()))
}

pub fn sso_login(profile: &str) -> anyhow::Result<()> {
    eprintln!("Logging in for AWS profile {}", profile);
    let args = ["--profile", profile, "sso", "login"];
    let mut child = aws_cli(&args).spawn()?;
    let status = child.wait()?;
    verify_exit(&args, status)
}

fn run_aws_cli(args: &[&str]) -> anyhow::Result<Output> {
    let output = aws_cli(args).output()?;
    match verify_exit(&args, output.status) {
        Ok(_) => Ok(output),
        Err(err) => {
            Err(err.context(String::from_utf8(output.stderr).unwrap_or("(binary)".to_string())))
        }
    }
}

fn aws_cli(args: &[&str]) -> Command {
    let mut command = Command::new("aws");
    command.args(args);
    command
}

fn verify_exit(args: &[&str], status: ExitStatus) -> anyhow::Result<()> {
    if status.success() {
        Ok(())
    } else {
        let command: Vec<&str> = args.to_vec();
        Err(anyhow::Error::msg(format!(
            "{}: Command exited unsuccessfully (status code {})",
            command.join(" "),
            status
                .code()
                .map(|code| code.to_string())
                .unwrap_or("unknown".to_string())
        )))
    }
}
