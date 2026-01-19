//! Integration tests for Docker sandbox functionality
//!
//! These tests validate the sandbox container lifecycle:
//! - Container creation when starting a sandboxed session
//! - Container cleanup when deleting a sandboxed session
//! - Docker availability validation
//! - Tool availability in the sandbox image

use agent_of_empires::docker::{is_daemon_running, is_docker_available, DockerContainer};
use agent_of_empires::session::{Instance, SandboxInfo, Storage};
use std::path::PathBuf;

/// Tools that should be available in the sandbox image.
/// When adding a new tool to AvailableTools, add it here with its:
/// - name: The tool name (must match AvailableTools field)
/// - dockerfile_pattern: A pattern that should appear in the Dockerfile install section
/// - binary: The binary name to check for in the container
const SANDBOX_TOOLS: &[SandboxTool] = &[
    SandboxTool {
        name: "claude",
        dockerfile_pattern: "claude.ai/install",
        binary: "claude",
    },
    SandboxTool {
        name: "opencode",
        dockerfile_pattern: "opencode.ai/install",
        binary: "opencode",
    },
    SandboxTool {
        name: "codex",
        dockerfile_pattern: "@openai/codex",
        binary: "codex",
    },
];

struct SandboxTool {
    name: &'static str,
    dockerfile_pattern: &'static str,
    binary: &'static str,
}

fn docker_available() -> bool {
    is_docker_available() && is_daemon_running()
}

fn dockerfile_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docker")
        .join("Dockerfile")
}

/// Static test: Verify Dockerfile contains install commands for all supported tools.
/// This test doesn't require Docker and should always run.
#[test]
fn test_dockerfile_installs_all_tools() {
    let dockerfile = std::fs::read_to_string(dockerfile_path())
        .expect("Failed to read Dockerfile - is docker/Dockerfile missing?");

    let mut missing_tools = Vec::new();

    for tool in SANDBOX_TOOLS {
        if !dockerfile.contains(tool.dockerfile_pattern) {
            missing_tools.push(format!(
                "Tool '{}' is missing from Dockerfile (expected pattern: '{}')",
                tool.name, tool.dockerfile_pattern
            ));
        }
    }

    assert!(
        missing_tools.is_empty(),
        "Dockerfile is missing install commands for sandbox tools:\n  - {}\n\n\
         When adding a new tool to AvailableTools:\n\
         1. Add the install command to docker/Dockerfile\n\
         2. Add the tool to SANDBOX_TOOLS in tests/sandbox_integration.rs\n\
         3. Rebuild the Docker image: docker build -t aoe-sandbox:latest docker/",
        missing_tools.join("\n  - ")
    );
}

/// Static test: Verify all tools in AvailableTools have a corresponding SANDBOX_TOOLS entry.
/// This ensures we don't forget to add new tools to the test coverage.
#[test]
fn test_all_available_tools_have_sandbox_entries() {
    // These must match the fields in AvailableTools struct
    let available_tool_names = ["claude", "opencode", "codex"];

    let sandbox_tool_names: Vec<&str> = SANDBOX_TOOLS.iter().map(|t| t.name).collect();

    for tool_name in &available_tool_names {
        assert!(
            sandbox_tool_names.contains(tool_name),
            "Tool '{}' is in AvailableTools but not in SANDBOX_TOOLS constant.\n\
             Add it to SANDBOX_TOOLS in tests/sandbox_integration.rs to ensure \
             Docker image tests cover this tool.",
            tool_name
        );
    }
}

/// Runtime test: Verify all tools are actually executable in the sandbox container.
/// Requires Docker daemon and the aoe-sandbox image to be built.
#[test]
#[ignore = "requires Docker daemon and aoe-sandbox image"]
fn test_sandbox_image_has_all_tools() {
    if !docker_available() {
        eprintln!("Skipping: Docker not available");
        return;
    }

    let output = std::process::Command::new("docker")
        .args(["images", "-q", "aoe-sandbox:latest"])
        .output()
        .expect("Failed to run docker images command");

    if output.stdout.is_empty() {
        panic!(
            "aoe-sandbox:latest image not found. Build it first:\n\
             docker build -t aoe-sandbox:latest docker/"
        );
    }

    let mut failed_tools = Vec::new();

    for tool in SANDBOX_TOOLS {
        let result = std::process::Command::new("docker")
            .args(["run", "--rm", "aoe-sandbox:latest", "which", tool.binary])
            .output();

        match result {
            Ok(output) if output.status.success() => {
                // Tool found
            }
            _ => {
                failed_tools.push(tool.name);
            }
        }
    }

    assert!(
        failed_tools.is_empty(),
        "The following tools are not available in aoe-sandbox image: {:?}\n\n\
         This means the Dockerfile install commands may have failed or are incorrect.\n\
         1. Check docker/Dockerfile for proper install commands\n\
         2. Rebuild the image: docker build -t aoe-sandbox:latest docker/\n\
         3. Test manually: docker run --rm aoe-sandbox:latest which <tool>",
        failed_tools
    );
}

#[test]
fn test_sandbox_info_serialization() {
    let sandbox_info = SandboxInfo {
        enabled: true,
        container_id: Some("abc123".to_string()),
        image: Some("ubuntu:latest".to_string()),
        container_name: "aoe-sandbox-test1234".to_string(),
        created_at: Some(chrono::Utc::now()),
        yolo_mode: None,
    };

    let json = serde_json::to_string(&sandbox_info).unwrap();
    let deserialized: SandboxInfo = serde_json::from_str(&json).unwrap();

    assert!(deserialized.enabled);
    assert_eq!(deserialized.container_id, Some("abc123".to_string()));
    assert_eq!(deserialized.container_name, "aoe-sandbox-test1234");
}

#[test]
fn test_instance_is_sandboxed() {
    let mut inst = Instance::new("test", "/tmp/test");
    assert!(!inst.is_sandboxed());

    inst.sandbox_info = Some(SandboxInfo {
        enabled: true,
        container_id: None,
        image: None,
        container_name: "aoe-sandbox-test".to_string(),
        created_at: None,
        yolo_mode: None,
    });
    assert!(inst.is_sandboxed());

    inst.sandbox_info = Some(SandboxInfo {
        enabled: false,
        container_id: None,
        image: None,
        container_name: "aoe-sandbox-test".to_string(),
        created_at: None,
        yolo_mode: None,
    });
    assert!(!inst.is_sandboxed());
}

#[test]
fn test_sandbox_info_persists_across_save_load() {
    let temp = tempfile::TempDir::new().unwrap();
    std::env::set_var("HOME", temp.path());

    let storage = Storage::new("sandbox_test").unwrap();

    let mut inst = Instance::new("sandbox-session", "/tmp/project");
    inst.sandbox_info = Some(SandboxInfo {
        enabled: true,
        container_id: Some("container123".to_string()),
        image: Some("custom:image".to_string()),
        container_name: "aoe-sandbox-abcd1234".to_string(),
        created_at: Some(chrono::Utc::now()),
        yolo_mode: Some(true),
    });

    storage.save(&[inst.clone()]).unwrap();

    let loaded = storage.load().unwrap();
    assert_eq!(loaded.len(), 1);

    let loaded_inst = &loaded[0];
    assert!(loaded_inst.sandbox_info.is_some());

    let sandbox = loaded_inst.sandbox_info.as_ref().unwrap();
    assert!(sandbox.enabled);
    assert_eq!(sandbox.container_id, Some("container123".to_string()));
    assert_eq!(sandbox.image, Some("custom:image".to_string()));
    assert_eq!(sandbox.container_name, "aoe-sandbox-abcd1234");
}

#[test]
fn test_container_name_generation() {
    let name1 = DockerContainer::generate_name("abcd1234");
    assert_eq!(name1, "aoe-sandbox-abcd1234");

    let name2 = DockerContainer::generate_name("abcdefghijklmnop");
    assert_eq!(name2, "aoe-sandbox-abcdefgh");

    let name3 = DockerContainer::generate_name("abc");
    assert_eq!(name3, "aoe-sandbox-abc");
}

#[test]
#[ignore = "requires Docker daemon"]
fn test_container_lifecycle() {
    if !docker_available() {
        eprintln!("Skipping: Docker not available");
        return;
    }

    let session_id = format!(
        "test{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let container = DockerContainer::new(&session_id, "alpine:latest");

    assert!(!container.exists().unwrap());

    let config = agent_of_empires::docker::ContainerConfig {
        working_dir: "/workspace".to_string(),
        volumes: vec![],
        named_volumes: vec![],
        environment: vec![],
        cpu_limit: None,
        memory_limit: None,
    };

    let container_id = container.create(&config).unwrap();
    assert!(!container_id.is_empty());
    assert!(container.exists().unwrap());
    assert!(container.is_running().unwrap());

    container.stop().unwrap();
    assert!(container.exists().unwrap());
    assert!(!container.is_running().unwrap());

    container.remove(false).unwrap();
    assert!(!container.exists().unwrap());
}

#[test]
#[ignore = "requires Docker daemon"]
fn test_container_force_remove() {
    if !docker_available() {
        eprintln!("Skipping: Docker not available");
        return;
    }

    let session_id = format!(
        "testforce{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let container = DockerContainer::new(&session_id, "alpine:latest");

    let config = agent_of_empires::docker::ContainerConfig {
        working_dir: "/workspace".to_string(),
        volumes: vec![],
        named_volumes: vec![],
        environment: vec![],
        cpu_limit: None,
        memory_limit: None,
    };

    container.create(&config).unwrap();
    assert!(container.is_running().unwrap());

    // Force remove while running
    container.remove(true).unwrap();
    assert!(!container.exists().unwrap());
}
