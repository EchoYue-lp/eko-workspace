use echo_agent::prelude::{
    Agent, AgentConfig, CircuitBreakerConfig, CancellationToken, ReactAgent,
};
use echo_agent::testing::MockLlmClient;
use futures::StreamExt;
use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant};

fn configured_agent(
    mock: Arc<MockLlmClient>,
    retries: usize,
    retry_delay_ms: u64,
    breaker: Option<CircuitBreakerConfig>,
) -> Result<echo_agent::agent::ReactAgent, Box<dyn Error>> {
    let config = AgentConfig::new("mock-model", "primary-review", "review probe")
        .llm_max_retries(retries)
        .llm_retry_delay_ms(retry_delay_ms);
    let mut agent = ReactAgent::new(config).with_llm_client(mock);
    if let Some(config) = breaker {
        agent.set_circuit_breaker(config);
    }
    Ok(agent)
}

async fn cancellation_during_backoff() -> Result<(), Box<dyn Error>> {
    let mock = Arc::new(
        MockLlmClient::new()
            .with_rate_limit_error()
            .with_response("late response"),
    );
    let agent = configured_agent(mock.clone(), 1, 250, None)?;
    let cancel = CancellationToken::new();
    let started = Instant::now();
    let mut stream = agent
        .chat_stream_with_cancel("cancel while retry sleeps", cancel.clone())
        .await?;

    tokio::time::sleep(Duration::from_millis(20)).await;
    cancel.cancel();

    let mut event_count = 0usize;
    let mut errors = Vec::new();
    while let Some(event) = stream.next().await {
        event_count = event_count.saturating_add(1);
        if let Err(error) = event {
            errors.push(error.to_string());
        }
    }

    println!(
        "scenario=cancel calls={} events={} errors={:?} elapsed_ms={}",
        mock.call_count(),
        event_count,
        errors,
        started.elapsed().as_millis()
    );
    Ok(())
}

async fn open_breaker_admission() -> Result<(), Box<dyn Error>> {
    let mock = Arc::new(
        MockLlmClient::new()
            .with_network_error("first")
            .with_network_error("second"),
    );
    let agent = configured_agent(
        mock.clone(),
        0,
        1,
        Some(CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 1,
            timeout: Duration::from_secs(60),
        }),
    )?;

    let first = agent.chat("first").await;
    let calls_after_first = mock.call_count();
    let second = agent.chat("second").await;
    println!(
        "scenario=breaker calls_after_first={} calls_after_second={} first_err={} second_err={}",
        calls_after_first,
        mock.call_count(),
        first.is_err(),
        second.is_err()
    );
    Ok(())
}

async fn overflowing_backoff() -> Result<(), Box<dyn Error>> {
    let mock = Arc::new(
        MockLlmClient::new()
            .with_rate_limit_error()
            .with_response("unreachable after overflow"),
    );
    let agent = configured_agent(mock.clone(), 1, u64::MAX, None)?;
    let mut stream = agent.chat_stream("overflow backoff").await?;
    let mut event_count = 0usize;
    let mut errors = Vec::new();
    while let Some(event) = stream.next().await {
        event_count = event_count.saturating_add(1);
        if let Err(error) = event {
            errors.push(error.to_string());
        }
    }
    println!(
        "scenario=overflow calls={} events={} errors={:?}",
        mock.call_count(),
        event_count,
        errors
    );
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mode = std::env::args()
        .nth(1)
        .ok_or("expected one of: cancel, breaker, overflow")?;
    match mode.as_str() {
        "cancel" => cancellation_during_backoff().await,
        "breaker" => open_breaker_admission().await,
        "overflow" => overflowing_backoff().await,
        _ => Err(format!("unknown mode: {mode}").into()),
    }
}
