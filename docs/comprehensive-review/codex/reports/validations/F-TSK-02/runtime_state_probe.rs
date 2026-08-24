use echo_orchestration::tasks::{
    DagExecutionState, ManagedTask, PlanValidator, Task, TaskExecution, TaskKind, TaskManager,
    TaskSpec, TaskStatus,
};

fn runtime_task(id: &str, status: TaskStatus, dependencies: &[&str]) -> Task {
    Task {
        spec: TaskSpec {
            id: id.to_string(),
            title: id.to_string(),
            description: format!("execute {id}"),
            kind: TaskKind::Investigation,
            agent_role: "review-probe".to_string(),
            depends_on: dependencies.iter().map(|item| (*item).to_string()).collect(),
            files: Vec::new(),
            allowed_tools: Vec::new(),
            required_artifacts: Vec::new(),
            execution_checks: Vec::new(),
            acceptance_criteria: Vec::new(),
            max_retries: 1,
            metadata: Default::default(),
        },
        execution: TaskExecution {
            task_id: id.to_string(),
            status,
            retry_count: 0,
            failure_fingerprint: None,
            claim: None,
        },
    }
}

fn pause_run_probe() -> Result<(), String> {
    let manager = TaskManager::new();
    let mut pending = ManagedTask::new("pending", "pending task");
    pending.run_id = Some("run".to_string());
    manager.add_task(pending);
    let mut running = ManagedTask::new("running", "running task");
    running.run_id = Some("run".to_string());
    manager.add_task(running);
    manager.update_task("running", TaskStatus::Running)?;

    manager.pause_run("run", "probe");
    let pending_status = manager
        .get_task("pending")
        .map(|task| task.status)
        .ok_or_else(|| "pending task disappeared".to_string())?;
    let running_status = manager
        .get_task("running")
        .map(|task| task.status)
        .ok_or_else(|| "running task disappeared".to_string())?;
    if matches!(pending_status, TaskStatus::Paused(_))
        && matches!(running_status, TaskStatus::Paused(_))
    {
        Ok(())
    } else {
        Err(format!(
            "pause_run returned with statuses pending={pending_status:?}, running={running_status:?}"
        ))
    }
}

fn skipped_dependency_probe() -> Result<(), String> {
    let tasks = vec![
        runtime_task("upstream", TaskStatus::Skipped, &[]),
        runtime_task("downstream", TaskStatus::Pending, &["upstream"]),
    ];
    let state = DagExecutionState::from_tasks(&tasks);
    let ready = state.ready_task_ids(&tasks);
    if ready == ["downstream".to_string()] {
        Ok(())
    } else {
        Err(format!(
            "skipped upstream is counted as resolved for graph completion but leaves frontier {ready:?}"
        ))
    }
}

fn transitive_failure_probe() -> Result<(), String> {
    let tasks = vec![
        runtime_task("root", TaskStatus::Failed("probe".to_string()), &[]),
        runtime_task("child", TaskStatus::Pending, &["root"]),
        runtime_task("grandchild", TaskStatus::Pending, &["child"]),
    ];
    let state = DagExecutionState::from_tasks(&tasks);
    let blocked = state.blocked_by_failures(&tasks);
    if blocked == ["child".to_string(), "grandchild".to_string()] {
        Ok(())
    } else {
        Err(format!("blocked descendants were {blocked:?}"))
    }
}

fn paused_snapshot_probe() -> Result<(), String> {
    let tasks = vec![runtime_task(
        "paused",
        TaskStatus::Paused("persisted".to_string()),
        &[],
    )];
    let state = DagExecutionState::from_tasks(&tasks);
    let runnable = !state.ready_task_ids(&tasks).is_empty();
    let complete = state.all_completed(&tasks);
    let failed = !state.failed.is_empty();
    let cancelled = !state.cancelled.is_empty();
    let in_flight = !state.in_flight.is_empty();
    if runnable || complete || failed || cancelled || in_flight {
        Ok(())
    } else {
        Err("paused snapshot is classified only as an unfinished stall".to_string())
    }
}

fn retrying_snapshot_probe() -> Result<(), String> {
    let tasks = vec![runtime_task(
        "retrying",
        TaskStatus::Retrying {
            attempt: 2,
            last_error: "persisted".to_string(),
        },
        &[],
    )];
    let state = DagExecutionState::from_tasks(&tasks);
    if state.in_flight.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "retrying snapshot is treated as externally in-flight: {:?}",
            state.in_flight
        ))
    }
}

fn frontier_order_probe() -> Result<(), String> {
    let tasks = vec![
        runtime_task("z", TaskStatus::Pending, &[]),
        runtime_task("a", TaskStatus::Pending, &[]),
        runtime_task("m", TaskStatus::Pending, &[]),
    ];
    let state = DagExecutionState::from_tasks(&tasks);
    let first = state.ready_task_ids(&tasks);
    let second = state.ready_task_ids(&tasks);
    let expected = vec!["z".to_string(), "a".to_string(), "m".to_string()];
    if first == expected && second == expected {
        Ok(())
    } else {
        Err(format!("frontier runs returned {first:?} and {second:?}"))
    }
}

fn status_independent_validation_probe() -> Result<(), String> {
    let statuses = [
        TaskStatus::Pending,
        TaskStatus::Failed("probe".to_string()),
        TaskStatus::Skipped,
    ];
    for status in statuses {
        let tasks = vec![runtime_task("valid", status.clone(), &[])];
        PlanValidator::default()
            .validate_task_snapshot(&tasks)
            .map_err(|errors| format!("status {status:?} changed structural validity: {errors:?}"))?;
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let scenario = std::env::args()
        .nth(1)
        .ok_or_else(|| "missing scenario".to_string())?;
    match scenario.as_str() {
        "pause-run" => pause_run_probe(),
        "skipped-dependency" => skipped_dependency_probe(),
        "transitive-failure" => transitive_failure_probe(),
        "paused-snapshot" => paused_snapshot_probe(),
        "retrying-snapshot" => retrying_snapshot_probe(),
        "frontier-order" => frontier_order_probe(),
        "status-independent" => status_independent_validation_probe(),
        _ => Err(format!("unknown scenario: {scenario}")),
    }
}
