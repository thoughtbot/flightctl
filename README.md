# flightctl

Client for interacting with Mission Control workspaces.

# How to Use

In order to use `flightctl` commands to access the workspace, you will need the files in the `/templates` directory: the setup binary (`setup.sh`) and the configurations yaml (`flightctl.yaml`).

## Setup script

Copy `setup.sh` from `/templates` into your application codebase. We recommend storing it in a `bin/` directory.

The first time the `flightctl` command is used, the script will download the compiled asset for your operating system into your project's `/tmp` directory. Every other time going forward, the command will execute the downloaded binary.

## Configurations yaml

Copy the configurations template `flightctl.yaml` into your application root directory. Be sure to replace each variable interpolation with real values for your workspace.

## User Commands

```
flightctl config     Fetch configuration variables for a release
flightctl console    Run a console for a release
flightctl help       Prints this message or the help of the given subcommand(s)
flightctl kubectl    Run a kubectl command for a release
flightctl ps         List processes running for a release
flightctl run        Run a container command for a release
flightctl view       View information about this workspace
```
