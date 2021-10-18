use super::kubeclient::KubeClient;
use k8s_openapi::api::core::v1 as k8s;
use std::collections::HashMap;
use std::rc::Rc;

pub struct ResolvedVar {
    pub name: Rc<String>,
    pub value: ResolvedValue,
}

pub enum ResolvedValue {
    Pod {
        value: Option<String>,
    },
    ConfigMapKeyRef {
        config_map: Rc<String>,
        key: Rc<String>,
        value: Option<Rc<String>>,
    },
    SecretKeyRef {
        secret: Rc<String>,
        key: Rc<String>,
    },
    FieldRef {
        path: String,
    },
}

pub struct Resolver<'c> {
    cache: Cache<'c>,
}

impl<'c> Resolver<'c> {
    pub fn new(client: &'c KubeClient) -> Resolver<'c> {
        Resolver {
            cache: Cache::new(client),
        }
    }

    pub fn resolve(&mut self, container: k8s::Container) -> Vec<ResolvedVar> {
        let mut env: Vec<ResolvedVar> = Vec::new();
        let container_envs = container.env.unwrap_or(vec![]);
        env.append(&mut self.container_env_values(container_envs));
        let container_env_sources = container.env_from.unwrap_or(vec![]);
        env.append(&mut self.container_env_from_values(container_env_sources));
        env.sort_by(|a, b| a.name.cmp(&b.name));
        env
    }

    fn container_env_values(&mut self, container_envs: Vec<k8s::EnvVar>) -> Vec<ResolvedVar> {
        let mut vars = Vec::new();
        for env_var in container_envs {
            let name_ref = Rc::new(env_var.name);
            vars.push(match (env_var.value, env_var.value_from) {
                (None, Some(value_from)) => self.env_from_value(&name_ref, value_from),
                (Some(value), _) => ResolvedVar {
                    name: name_ref,
                    value: ResolvedValue::Pod { value: Some(value) },
                },
                (None, None) => ResolvedVar {
                    name: name_ref,
                    value: ResolvedValue::Pod { value: None },
                },
            })
        }
        vars
    }

    fn env_from_value(&mut self, name: &Rc<String>, value_from: k8s::EnvVarSource) -> ResolvedVar {
        let value = match (
            value_from.config_map_key_ref,
            value_from.field_ref,
            value_from.resource_field_ref,
            value_from.secret_key_ref,
        ) {
            (Some(config_map_key_ref), _, _, _) => match config_map_key_ref.name {
                Some(config_map_name) => {
                    let config_map_ref = Rc::new(config_map_name);
                    self.cache
                        .reference_config_map_key(&config_map_ref, &config_map_key_ref.key)
                }
                None => ResolvedValue::ConfigMapKeyRef {
                    config_map: Rc::new(String::from("null")),
                    key: Rc::new(config_map_key_ref.key),
                    value: None,
                },
            },
            (_, Some(field_ref), _, _) => ResolvedValue::FieldRef {
                path: field_ref.field_path,
            },
            (_, _, _, _) => ResolvedValue::Pod { value: None },
        };
        ResolvedVar {
            name: Rc::clone(name),
            value: value,
        }
    }

    fn container_env_from_values(
        &mut self,
        env_sources: Vec<k8s::EnvFromSource>,
    ) -> Vec<ResolvedVar> {
        let mut vars = Vec::new();
        for env_source in env_sources {
            match (env_source.config_map_ref, env_source.secret_ref) {
                (
                    Some(k8s::ConfigMapEnvSource {
                        name: Some(name), ..
                    }),
                    _,
                ) => {
                    vars.append(&mut self.cache.import_config_map(&name));
                }
                (
                    None,
                    Some(k8s::SecretEnvSource {
                        name: Some(name), ..
                    }),
                ) => {
                    vars.append(&mut self.cache.import_secret(&name));
                }
                _ => (),
            }
        }
        vars
    }
}

struct SharedMap {
    data: HashMap<Rc<String>, Rc<String>>,
}

impl SharedMap {
    fn from_config_map(config_map: k8s::ConfigMap) -> SharedMap {
        SharedMap {
            data: config_map
                .data
                .map(|values| {
                    values
                        .into_iter()
                        .map(|(key, value)| (Rc::new(key), Rc::new(value)))
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    fn from_secret(secret: k8s::Secret) -> SharedMap {
        SharedMap {
            data: secret
                .data
                .map(|values| {
                    values
                        .into_iter()
                        .map(|(key, k8s_openapi::ByteString(value))| {
                            (
                                Rc::new(key),
                                Rc::new(
                                    String::from_utf8(value).unwrap_or(String::from("(binary)")),
                                ),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

struct Cache<'c> {
    client: &'c KubeClient,
    config_maps: HashMap<Rc<String>, SharedMap>,
    secrets: HashMap<Rc<String>, SharedMap>,
}

impl<'c> Cache<'c> {
    fn new(client: &'c KubeClient) -> Cache<'c> {
        Cache {
            client: client,
            config_maps: HashMap::new(),
            secrets: HashMap::new(),
        }
    }

    fn reference_config_map_key(&mut self, name: &str, key: &str) -> ResolvedValue {
        let name_ref = Rc::new(String::from(name));
        let key_ref = Rc::new(String::from(key));
        self.fetch_config_map(&name_ref);
        let value = self.lookup_config_map_key(&name_ref, &key_ref);
        ResolvedValue::ConfigMapKeyRef {
            config_map: name_ref,
            key: key_ref,
            value: value,
        }
    }

    fn lookup_config_map_key(&self, name: &Rc<String>, key: &Rc<String>) -> Option<Rc<String>> {
        let config_map = self.config_maps.get(name)?;
        let value = config_map.data.get(key)?;
        Some(Rc::clone(value))
    }

    fn import_config_map(&mut self, name: &str) -> Vec<ResolvedVar> {
        let name_ref = Rc::new(String::from(name));
        self.fetch_config_map(&name_ref);
        match self.config_maps.get(&name_ref) {
            Some(config_map) => config_map
                .data
                .iter()
                .map(|(key, value)| ResolvedVar {
                    name: Rc::clone(key),
                    value: ResolvedValue::ConfigMapKeyRef {
                        config_map: Rc::clone(&name_ref),
                        key: Rc::clone(key),
                        value: Some(Rc::clone(value)),
                    },
                })
                .collect(),
            None => Vec::new(),
        }
    }

    fn fetch_config_map(&mut self, name: &Rc<String>) {
        if !self.config_maps.contains_key(name) {
            match self
                .client
                .fetch_resource::<k8s::ConfigMap>(&format!("configmap/{}", &name))
            {
                Ok(config_map) => {
                    self.config_maps
                        .insert(Rc::clone(name), SharedMap::from_config_map(config_map));
                }
                Err(_) => (),
            }
        }
    }

    fn import_secret(&mut self, name: &str) -> Vec<ResolvedVar> {
        let name_ref = Rc::new(String::from(name));
        self.fetch_secret(&name_ref);
        match self.secrets.get(&name_ref) {
            Some(secret) => secret
                .data
                .iter()
                .map(|(key, _)| ResolvedVar {
                    name: Rc::clone(key),
                    value: ResolvedValue::SecretKeyRef {
                        secret: Rc::clone(&name_ref),
                        key: Rc::clone(key),
                    },
                })
                .collect(),
            None => Vec::new(),
        }
    }

    fn fetch_secret(&mut self, name: &Rc<String>) {
        if !self.secrets.contains_key(name) {
            match self
                .client
                .fetch_resource::<k8s::Secret>(&format!("secret/{}", &name))
            {
                Ok(secret) => {
                    self.secrets
                        .insert(Rc::clone(name), SharedMap::from_secret(secret));
                }
                Err(_) => (),
            }
        }
    }
}
