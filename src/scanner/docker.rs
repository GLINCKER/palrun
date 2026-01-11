//! Docker Compose scanner.
//!
//! Scans docker-compose.yml, docker-compose.yaml, or compose.yaml files
//! to discover Docker Compose services and generate related commands.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use super::Scanner;
use crate::core::{Command, CommandSource};

/// Scanner for Docker Compose projects.
pub struct DockerScanner;

impl Scanner for DockerScanner {
    fn name(&self) -> &str {
        "docker"
    }

    fn scan(&self, path: &Path) -> anyhow::Result<Vec<Command>> {
        // Try to find a docker compose file
        let compose_file = find_compose_file(path);
        let Some(compose_path) = compose_file else {
            return Ok(Vec::new());
        };

        let content = std::fs::read_to_string(&compose_path)?;
        let compose: DockerCompose = serde_yaml::from_str(&content)?;

        let mut commands = Vec::new();
        let source = CommandSource::DockerCompose(compose_path.clone());

        // Generate global docker compose commands
        commands.push(
            Command::new("docker compose up -d", "docker compose up -d")
                .with_description("Start all services in detached mode")
                .with_source(source.clone())
                .with_tags(vec!["docker".to_string(), "compose".to_string(), "up".to_string()]),
        );

        commands.push(
            Command::new("docker compose down", "docker compose down")
                .with_description("Stop and remove all services")
                .with_source(source.clone())
                .with_tags(vec!["docker".to_string(), "compose".to_string(), "down".to_string()]),
        );

        commands.push(
            Command::new("docker compose build", "docker compose build")
                .with_description("Build all services")
                .with_source(source.clone())
                .with_tags(vec!["docker".to_string(), "compose".to_string(), "build".to_string()]),
        );

        commands.push(
            Command::new("docker compose ps", "docker compose ps")
                .with_description("List running containers")
                .with_source(source.clone())
                .with_tags(vec!["docker".to_string(), "compose".to_string(), "ps".to_string()]),
        );

        // Generate per-service commands
        if let Some(services) = compose.services {
            for (service_name, service_config) in services {
                // Get description from labels if available
                let description = service_config
                    .as_ref()
                    .and_then(|s| s.labels.as_ref())
                    .and_then(|labels| labels.get("description").cloned())
                    .or_else(|| {
                        service_config.as_ref().and_then(|s| s.labels.as_ref()).and_then(|labels| {
                            labels.get("com.docker.compose.description").cloned()
                        })
                    });

                // docker compose up <service>
                let up_cmd = format!("docker compose up {service_name}");
                let up_desc = description
                    .clone()
                    .map(|d| format!("Start {service_name}: {d}"))
                    .unwrap_or_else(|| format!("Start {service_name} service"));
                commands.push(
                    Command::new(&up_cmd, &up_cmd)
                        .with_description(up_desc)
                        .with_source(source.clone())
                        .with_tags(vec![
                            "docker".to_string(),
                            "compose".to_string(),
                            "up".to_string(),
                            service_name.clone(),
                        ]),
                );

                // docker compose logs <service>
                let logs_cmd = format!("docker compose logs {service_name}");
                commands.push(
                    Command::new(&logs_cmd, &logs_cmd)
                        .with_description(format!("View logs for {service_name}"))
                        .with_source(source.clone())
                        .with_tags(vec![
                            "docker".to_string(),
                            "compose".to_string(),
                            "logs".to_string(),
                            service_name.clone(),
                        ]),
                );

                // docker compose restart <service>
                let restart_cmd = format!("docker compose restart {service_name}");
                commands.push(
                    Command::new(&restart_cmd, &restart_cmd)
                        .with_description(format!("Restart {service_name} service"))
                        .with_source(source.clone())
                        .with_tags(vec![
                            "docker".to_string(),
                            "compose".to_string(),
                            "restart".to_string(),
                            service_name.clone(),
                        ]),
                );
            }
        }

        Ok(commands)
    }
}

/// Find the docker compose file in the given directory.
/// Checks for docker-compose.yml, docker-compose.yaml, and compose.yaml in that order.
fn find_compose_file(path: &Path) -> Option<std::path::PathBuf> {
    let candidates = ["docker-compose.yml", "docker-compose.yaml", "compose.yaml"];

    for candidate in candidates {
        let file_path = path.join(candidate);
        if file_path.exists() {
            return Some(file_path);
        }
    }

    None
}

/// Parsed docker-compose.yml structure.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DockerCompose {
    /// Version of the compose file format (optional in v3+)
    version: Option<String>,

    /// Services defined in the compose file
    services: Option<HashMap<String, Option<ServiceConfig>>>,
}

/// Configuration for a single service.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ServiceConfig {
    /// Image to use for this service
    image: Option<String>,

    /// Build configuration
    build: Option<BuildConfig>,

    /// Labels for the service (can contain description)
    labels: Option<HashMap<String, String>>,

    /// Ports mapping
    ports: Option<Vec<PortMapping>>,

    /// Environment variables
    environment: Option<EnvironmentConfig>,

    /// Container name
    container_name: Option<String>,
}

/// Build configuration for a service.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
enum BuildConfig {
    /// Simple string path to build context
    Simple(String),

    /// Detailed build configuration
    Detailed {
        /// Build context path
        context: Option<String>,
        /// Dockerfile path
        dockerfile: Option<String>,
        /// Build arguments
        args: Option<HashMap<String, String>>,
    },
}

/// Port mapping configuration.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
enum PortMapping {
    /// Simple string format (e.g., "8080:80")
    Simple(String),

    /// Numeric port
    Numeric(u16),

    /// Detailed port configuration
    Detailed { target: u16, published: Option<u16>, protocol: Option<String> },
}

/// Environment variable configuration.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
enum EnvironmentConfig {
    /// List of KEY=VALUE strings
    List(Vec<String>),

    /// Map of key-value pairs
    Map(HashMap<String, Option<String>>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_docker_scanner_name() {
        let scanner = DockerScanner;
        assert_eq!(scanner.name(), "docker");
    }

    #[test]
    fn test_find_compose_file_docker_compose_yml() {
        let temp_dir = TempDir::new().unwrap();
        let compose_path = temp_dir.path().join("docker-compose.yml");
        fs::write(&compose_path, "version: '3'").unwrap();

        let found = find_compose_file(temp_dir.path());
        assert!(found.is_some());
        assert_eq!(found.unwrap(), compose_path);
    }

    #[test]
    fn test_find_compose_file_docker_compose_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let compose_path = temp_dir.path().join("docker-compose.yaml");
        fs::write(&compose_path, "version: '3'").unwrap();

        let found = find_compose_file(temp_dir.path());
        assert!(found.is_some());
        assert_eq!(found.unwrap(), compose_path);
    }

    #[test]
    fn test_find_compose_file_compose_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let compose_path = temp_dir.path().join("compose.yaml");
        fs::write(&compose_path, "version: '3'").unwrap();

        let found = find_compose_file(temp_dir.path());
        assert!(found.is_some());
        assert_eq!(found.unwrap(), compose_path);
    }

    #[test]
    fn test_find_compose_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let found = find_compose_file(temp_dir.path());
        assert!(found.is_none());
    }

    #[test]
    fn test_find_compose_file_priority() {
        let temp_dir = TempDir::new().unwrap();

        // Create both files - docker-compose.yml should be preferred
        fs::write(temp_dir.path().join("docker-compose.yml"), "version: '3'").unwrap();
        fs::write(temp_dir.path().join("compose.yaml"), "version: '3'").unwrap();

        let found = find_compose_file(temp_dir.path());
        assert!(found.is_some());
        assert!(found.unwrap().ends_with("docker-compose.yml"));
    }

    #[test]
    fn test_scan_no_compose_file() {
        let temp_dir = TempDir::new().unwrap();
        let scanner = DockerScanner;

        let commands = scanner.scan(temp_dir.path()).unwrap();
        assert!(commands.is_empty());
    }

    #[test]
    fn test_scan_simple_compose_file() {
        let temp_dir = TempDir::new().unwrap();
        let compose_content = r"
version: '3'
services:
  web:
    image: nginx
  db:
    image: postgres
";
        fs::write(temp_dir.path().join("docker-compose.yml"), compose_content).unwrap();

        let scanner = DockerScanner;
        let commands = scanner.scan(temp_dir.path()).unwrap();

        // Should have global commands (4) + per-service commands (3 per service * 2 services = 6)
        assert_eq!(commands.len(), 10);

        // Check for global commands
        let command_names: Vec<&str> = commands.iter().map(|c| c.name.as_str()).collect();
        assert!(command_names.contains(&"docker compose up -d"));
        assert!(command_names.contains(&"docker compose down"));
        assert!(command_names.contains(&"docker compose build"));
        assert!(command_names.contains(&"docker compose ps"));

        // Check for service-specific commands
        assert!(command_names.contains(&"docker compose up web"));
        assert!(command_names.contains(&"docker compose logs web"));
        assert!(command_names.contains(&"docker compose restart web"));
        assert!(command_names.contains(&"docker compose up db"));
        assert!(command_names.contains(&"docker compose logs db"));
        assert!(command_names.contains(&"docker compose restart db"));
    }

    #[test]
    fn test_scan_compose_with_labels() {
        let temp_dir = TempDir::new().unwrap();
        let compose_content = r#"
version: '3'
services:
  api:
    image: node:18
    labels:
      description: "API server for the application"
"#;
        fs::write(temp_dir.path().join("docker-compose.yml"), compose_content).unwrap();

        let scanner = DockerScanner;
        let commands = scanner.scan(temp_dir.path()).unwrap();

        // Find the "docker compose up api" command and check its description
        let up_cmd = commands.iter().find(|c| c.name == "docker compose up api");
        assert!(up_cmd.is_some());
        let up_cmd = up_cmd.unwrap();
        assert!(up_cmd.description.as_ref().unwrap().contains("API server for the application"));
    }

    #[test]
    fn test_scan_compose_without_version() {
        let temp_dir = TempDir::new().unwrap();
        // Modern compose files don't require version
        let compose_content = r"
services:
  app:
    image: alpine
";
        fs::write(temp_dir.path().join("compose.yaml"), compose_content).unwrap();

        let scanner = DockerScanner;
        let commands = scanner.scan(temp_dir.path()).unwrap();

        // Should have global commands (4) + per-service commands (3)
        assert_eq!(commands.len(), 7);
    }

    #[test]
    fn test_parse_compose_with_build() {
        let yaml = r#"
version: '3.8'
services:
  app:
    build:
      context: .
      dockerfile: Dockerfile.prod
    ports:
      - "3000:3000"
"#;
        let compose: DockerCompose = serde_yaml::from_str(yaml).unwrap();
        assert!(compose.services.is_some());
        let services = compose.services.unwrap();
        assert!(services.contains_key("app"));
    }

    #[test]
    fn test_parse_compose_with_environment() {
        let yaml = r"
version: '3'
services:
  db:
    image: postgres
    environment:
      - POSTGRES_USER=admin
      - POSTGRES_PASSWORD=secret
";
        let compose: DockerCompose = serde_yaml::from_str(yaml).unwrap();
        assert!(compose.services.is_some());
    }

    #[test]
    fn test_parse_compose_environment_map() {
        let yaml = r"
version: '3'
services:
  db:
    image: postgres
    environment:
      POSTGRES_USER: admin
      POSTGRES_PASSWORD: secret
";
        let compose: DockerCompose = serde_yaml::from_str(yaml).unwrap();
        assert!(compose.services.is_some());
    }

    #[test]
    fn test_command_source_is_docker_compose() {
        let temp_dir = TempDir::new().unwrap();
        let compose_content = r"
services:
  app:
    image: alpine
";
        fs::write(temp_dir.path().join("docker-compose.yml"), compose_content).unwrap();

        let scanner = DockerScanner;
        let commands = scanner.scan(temp_dir.path()).unwrap();

        for cmd in &commands {
            assert!(matches!(cmd.source, CommandSource::DockerCompose(_)));
        }
    }

    #[test]
    fn test_command_tags() {
        let temp_dir = TempDir::new().unwrap();
        let compose_content = r"
services:
  web:
    image: nginx
";
        fs::write(temp_dir.path().join("docker-compose.yml"), compose_content).unwrap();

        let scanner = DockerScanner;
        let commands = scanner.scan(temp_dir.path()).unwrap();

        // All commands should have "docker" and "compose" tags
        for cmd in &commands {
            assert!(cmd.tags.contains(&"docker".to_string()));
            assert!(cmd.tags.contains(&"compose".to_string()));
        }

        // Service-specific commands should have the service name as a tag
        let web_up = commands.iter().find(|c| c.name == "docker compose up web");
        assert!(web_up.is_some());
        assert!(web_up.unwrap().tags.contains(&"web".to_string()));
    }
}
