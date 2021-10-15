use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Auth {
    pub name: String,

    #[serde(flatten)]
    pub config: AuthConfig,
}

impl fmt::Display for Auth {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "provider", content = "params", rename_all = "kebab-case")]
pub enum AuthConfig {
    AwsSso {
        #[serde(flatten)]
        config: HashMap<String, String>,
    },
}

#[derive(Debug, Deserialize)]
pub struct Cluster {
    pub name: String,

    pub auth: String,

    #[serde(flatten)]
    pub config: ClusterConfig,
}

impl fmt::Display for Cluster {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "provider", content = "params", rename_all = "kebab-case")]
pub enum ClusterConfig {
    Eks { name: String, region: String },
}

#[derive(Debug, Deserialize)]
pub struct Application {
    pub manifests: ApplicationManifests,

    pub name: String,

    #[serde(flatten)]
    pub config: ApplicationConfig,
}

impl fmt::Display for Application {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "provider", content = "params", rename_all = "kebab-case")]
pub enum ApplicationConfig {
    Kubectl {
        #[serde(default)]
        console: Option<Console>,

        #[serde(default)]
        selector: HashMap<String, String>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "provider", content = "params", rename_all = "kebab-case")]
pub enum Console {
    Exec {
        command: Vec<String>,
        selector: HashMap<String, String>,
    },
}

#[derive(Debug, Deserialize)]
pub struct ApplicationManifests {
    provider: ManifestsProvider,
    repo: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ManifestsProvider {
    Kustomize,
}

#[derive(Debug, Deserialize)]
pub struct Release {
    pub application: String,

    pub context: String,

    pub environment: String,

    pub manifests: ManifestConfig,

    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", untagged)]
pub enum ManifestConfig {
    Kustomize { path: String },
}

impl fmt::Display for Release {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Deserialize)]
pub struct Context {
    pub auth: String,
    pub cluster: String,
    pub name: String,
    pub namespace: String,
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Config {
    pub api_version: String,
    pub applications: Vec<Application>,
    pub auth: Vec<Auth>,
    pub clusters: Vec<Cluster>,
    pub contexts: Vec<Context>,
    pub kind: String,
    pub releases: Vec<Release>,
}

impl Config {
    pub fn find_auth(&self, context: &Context) -> anyhow::Result<&Auth> {
        self.auth
            .iter()
            .find(|&auth| &auth.name == &context.auth)
            .ok_or(anyhow::Error::msg(format!(
                "Context {} uses auth {}, which isn't defined",
                context.name, context.auth
            )))
    }

    pub fn find_cluster(&self, context: &Context) -> anyhow::Result<&Cluster> {
        self.clusters
            .iter()
            .find(|&cluster| &cluster.name == &context.cluster)
            .ok_or(anyhow::Error::msg(format!(
                "Context {} uses cluster {}, which isn't defined",
                context.name, context.cluster
            )))
    }

    pub fn find_context(&self, release: &Release) -> anyhow::Result<&Context> {
        self.contexts
            .iter()
            .find(|&context| &context.name == &release.context)
            .ok_or(anyhow::Error::msg(format!(
                "Release {} uses context {}, which isn't defined",
                release.name, release.context
            )))
    }
}

#[derive(Debug)]
pub struct ConfigFile {
    pub config: Config,
    pub path: PathBuf,
}

impl ConfigFile {
    pub fn find() -> anyhow::Result<ConfigFile> {
        let current_dir = std::env::current_dir()?;
        match find_config(current_dir) {
            Some(path) => {
                let file = std::fs::File::open(&path)?;
                let reader = std::io::BufReader::new(file);
                let config = serde_yaml::from_reader(reader)?;
                Ok(ConfigFile {
                    config: config,
                    path: path,
                })
            }
            None => Err(anyhow::Error::msg("No configuration file found")),
        }
    }
}

fn find_config(mut dir: PathBuf) -> Option<PathBuf> {
    let mut path = dir.clone();
    path.push("flightctl.yaml");

    if path.exists() {
        Some(path)
    } else {
        if dir.pop() {
            find_config(dir)
        } else {
            None
        }
    }
}
