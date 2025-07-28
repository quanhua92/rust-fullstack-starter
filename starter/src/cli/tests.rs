use super::models::*;

#[test]
fn test_cli_parsing() {
    use clap::Parser;

    // Test server command parsing
    let args = vec!["starter", "server", "--port", "3000"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Commands::Server { port } => {
            assert_eq!(port, 3000);
        }
        _ => panic!("Expected Server command"),
    }
}

#[test]
fn test_admin_commands_parsing() {
    use clap::Parser;

    // Test list-tasks command
    let args = vec![
        "starter",
        "admin",
        "list-tasks",
        "--limit",
        "25",
        "--verbose",
    ];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Commands::Admin { admin_command } => match admin_command {
            AdminCommands::ListTasks {
                status: _,
                task_type: _,
                limit,
                verbose,
            } => {
                assert_eq!(limit, 25);
                assert!(verbose);
            }
            _ => panic!("Expected ListTasks command"),
        },
        _ => panic!("Expected Admin command"),
    }
}

#[test]
fn test_task_stats_command_parsing() {
    use clap::Parser;

    // Test task-stats command with tag filter
    let args = vec!["starter", "admin", "task-stats", "--tag", "test_tag"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Commands::Admin { admin_command } => match admin_command {
            AdminCommands::TaskStats { tag } => {
                assert_eq!(tag, Some("test_tag".to_string()));
            }
            _ => panic!("Expected TaskStats command"),
        },
        _ => panic!("Expected Admin command"),
    }
}

#[test]
fn test_clear_completed_command_parsing() {
    use clap::Parser;

    // Test clear-completed command with dry-run
    let args = vec![
        "starter",
        "admin",
        "clear-completed",
        "--older-than-days",
        "14",
        "--dry-run",
    ];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Commands::Admin { admin_command } => match admin_command {
            AdminCommands::ClearCompleted {
                older_than_days,
                dry_run,
            } => {
                assert_eq!(older_than_days, 14);
                assert!(dry_run);
            }
            _ => panic!("Expected ClearCompleted command"),
        },
        _ => panic!("Expected Admin command"),
    }
}

#[test]
fn test_export_openapi_command_parsing() {
    use clap::Parser;

    // Test export-openapi command with custom output
    let args = vec![
        "starter",
        "export-openapi",
        "--output",
        "custom/path/openapi.json",
    ];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Commands::ExportOpenApi { output } => {
            assert_eq!(output, "custom/path/openapi.json");
        }
        _ => panic!("Expected ExportOpenApi command"),
    }
}

#[test]
fn test_worker_command_parsing() {
    use clap::Parser;

    let args = vec!["starter", "worker"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Commands::Worker => {
            // Worker command has no additional parameters
        }
        _ => panic!("Expected Worker command"),
    }
}

#[test]
fn test_health_check_command_parsing() {
    use clap::Parser;

    let args = vec!["starter", "health-check"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Commands::HealthCheck => {
            // HealthCheck command has no additional parameters
        }
        _ => panic!("Expected HealthCheck command"),
    }
}

#[test]
fn test_admin_config_default() {
    let config = AdminConfig::default();
    assert_eq!(config.default_limit, 50);
    assert_eq!(config.default_days, 7);
}

#[test]
fn test_task_info_serialization() {
    let task_info = TaskInfo {
        id: uuid::Uuid::new_v4(),
        task_type: "email".to_string(),
        status: "pending".to_string(),
        priority: "normal".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        metadata: serde_json::json!({"test": true}),
    };

    // Test that the struct can be serialized and deserialized
    let serialized = serde_json::to_string(&task_info).unwrap();
    let deserialized: TaskInfo = serde_json::from_str(&serialized).unwrap();

    assert_eq!(task_info.id, deserialized.id);
    assert_eq!(task_info.task_type, deserialized.task_type);
    assert_eq!(task_info.status, deserialized.status);
    assert_eq!(task_info.priority, deserialized.priority);
}

#[test]
fn test_task_stats_summary() {
    let stats = vec![
        TaskStats {
            status: "pending".to_string(),
            count: 5,
        },
        TaskStats {
            status: "completed".to_string(),
            count: 10,
        },
    ];

    let summary = TaskStatsSummary {
        stats: stats.clone(),
        total: 15,
        avg_completion_time: Some(2.5),
    };

    assert_eq!(summary.stats.len(), 2);
    assert_eq!(summary.total, 15);
    assert_eq!(summary.avg_completion_time, Some(2.5));

    // Test individual stats
    assert_eq!(summary.stats[0].status, "pending");
    assert_eq!(summary.stats[0].count, 5);
    assert_eq!(summary.stats[1].status, "completed");
    assert_eq!(summary.stats[1].count, 10);
}
