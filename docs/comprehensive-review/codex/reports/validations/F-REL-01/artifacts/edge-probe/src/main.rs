use chrono::{DateTime, Utc};
use echo_agent::agent::{Agent, AgentConfig, CancellationToken, ReactAgentBuilder};
use echo_agent::agent::react::ReactAgent;
use echo_agent::testing::{MockLlmClient, MockTool};
use echo_agent::tools::ToolExecutionConfig;
use futures::{FutureExt, StreamExt};
use echo_core::budget::TokenBudget;
use echo_core::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use echo_core::retry::{RetryPolicy, with_retry};
use echo_core::utils::{hash::fnv1a_64, json_parse, time};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};

fn budget_probe() {
    let budget = TokenBudget::new(100);
    let overflow = std::panic::catch_unwind(|| budget.allocate(usize::MAX, 1, 1));
    println!("allocation_overflow_panicked={}", overflow.is_err());

    let invalid = TokenBudget::new(100).with_allocations(-0.5, 0.0, 0.0, 0.0);
    println!(
        "negative_system_budget={} conversation_budget={}",
        invalid.system_prompt_budget(),
        invalid.conversation_budget()
    );

    assert!(overflow.is_err(), "unchecked total must panic in debug mode");
    assert_eq!(invalid.system_prompt_budget(), 0);
    assert_eq!(invalid.conversation_budget(), 150);
}

async fn retry_cancel_probe() {
    let cancelled = Arc::new(AtomicBool::new(false));
    let attempts = Arc::new(AtomicU32::new(0));
    let policy = RetryPolicy::new(1, Duration::from_millis(250)).jitter(false);

    let cancel_writer = Arc::clone(&cancelled);
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(20)).await;
        cancel_writer.store(true, Ordering::SeqCst);
    });

    let started = Instant::now();
    let result = with_retry(&policy, || {
        let cancelled = Arc::clone(&cancelled);
        let attempts = Arc::clone(&attempts);
        async move {
            attempts.fetch_add(1, Ordering::SeqCst);
            if cancelled.load(Ordering::SeqCst) {
                Err::<(), _>("cancelled")
            } else {
                Err::<(), _>("retryable")
            }
        }
    })
    .await;
    let elapsed_ms = started.elapsed().as_millis();
    println!(
        "cancel_flag={} attempts={} elapsed_ms={} result={:?}",
        cancelled.load(Ordering::SeqCst),
        attempts.load(Ordering::SeqCst),
        elapsed_ms,
        result
    );

    assert!(cancelled.load(Ordering::SeqCst));
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
    assert!(elapsed_ms >= 200, "backoff did not wake for cancellation");
}

fn circuit_probe() {
    let breaker = CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 1,
        success_threshold: 2,
        timeout: Duration::ZERO,
    });
    breaker.record_failure();
    let first_probe_rejected = breaker.try_advance();
    // Simulate cancellation/drop: no record_success or record_failure follows.
    let next_probe_rejected = breaker.try_advance();
    println!(
        "state={} first_probe_rejected={} next_probe_rejected={}",
        breaker.state_name(),
        first_probe_rejected,
        next_probe_rejected
    );
    assert!(!first_probe_rejected);
    assert!(next_probe_rejected);
    assert_eq!(breaker.state_name(), "half_open");
}

async fn live_circuit_probe() {
    let mock = Arc::new(
        MockLlmClient::new()
            .with_network_error("first failure")
            .with_network_error("second failure"),
    );
    let config = AgentConfig::minimal("mock", "probe")
        .llm_max_retries(0)
        .llm_retry_delay_ms(0);
    let mut agent = ReactAgent::new(config).with_llm_client(mock.clone());
    agent.set_circuit_breaker(CircuitBreakerConfig {
        failure_threshold: 1,
        success_threshold: 1,
        timeout: Duration::from_secs(60),
    });

    let first = agent.chat("first").await;
    let calls_after_first = mock.call_count();
    let second = agent.chat("second").await;
    let calls_after_second = mock.call_count();
    println!(
        "first_err={} second_err={} calls_after_first={} calls_after_second={}",
        first.is_err(),
        second.is_err(),
        calls_after_first,
        calls_after_second
    );
    assert!(first.is_err());
    assert!(second.is_err());
    assert_eq!(calls_after_first, 1);
    assert_eq!(calls_after_second, 2, "open breaker did not reject second call");
}

async fn live_retry_cancel_probe() {
    let mock = Arc::new(
        MockLlmClient::new()
            .with_rate_limit_error()
            .with_response("late success"),
    );
    let config = AgentConfig::minimal("mock", "probe")
        .llm_max_retries(1)
        .llm_retry_delay_ms(250);
    let agent = ReactAgent::new(config).with_llm_client(mock.clone());
    let cancel = CancellationToken::new();
    let cancel_writer = cancel.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(20)).await;
        cancel_writer.cancel();
    });

    let started = Instant::now();
    let stream = agent
        .chat_stream_with_cancel("cancel retry", cancel)
        .await
        .unwrap_or_else(|error| panic!("stream setup failed: {error}"));
    let events: Vec<_> = stream.collect().await;
    println!(
        "calls={} elapsed_ms={} events={}",
        mock.call_count(),
        started.elapsed().as_millis(),
        events.len()
    );
    assert_eq!(mock.call_count(), 2, "retry proceeded after cancellation");
    assert!(started.elapsed() >= Duration::from_millis(200));
}

async fn llm_backoff_overflow_probe() {
    let mock = Arc::new(
        MockLlmClient::new()
            .with_network_error("first failure")
            .with_network_error("second failure"),
    );
    let config = AgentConfig::minimal("mock", "probe")
        .llm_max_retries(1)
        .llm_retry_delay_ms(u64::MAX);
    let agent = ReactAgent::new(config).with_llm_client(mock);
    let outcome = std::panic::AssertUnwindSafe(agent.chat("overflow"))
        .catch_unwind()
        .await;
    match outcome {
        Err(_) => println!("llm_backoff_panic_escaped=true"),
        Ok(Err(error)) => println!("llm_backoff_panic_escaped=false returned_error={error}"),
        Ok(Ok(value)) => {
            println!(
                "llm_backoff_panic_escaped=false returned_success_len={}",
                value.len()
            );
            assert!(value.is_empty(), "current stream boundary returned unexpected content");
        }
    }
}

async fn tool_timeout_overflow_probe() {
    let mock = Arc::new(
        MockLlmClient::new()
            .then_tool_call("call-1", "probe_tool", "{}")
            .with_response("done"),
    );
    let agent = ReactAgentBuilder::new()
        .llm_client(mock)
        .tool(Box::new(MockTool::new("probe_tool").with_response("ok")))
        .tool_execution(ToolExecutionConfig {
            timeout_ms: 1,
            retry_on_fail: true,
            max_retries: u32::MAX,
            retry_delay_ms: u64::MAX,
            max_concurrency: Some(1),
            max_read_concurrency: Some(1),
        })
        .max_iterations(2)
        .build()
        .unwrap_or_else(|error| panic!("agent fixture build failed: {error}"));

    let outcome = std::panic::AssertUnwindSafe(agent.chat("run tool"))
        .catch_unwind()
        .await;
    match outcome {
        Err(_) => println!("tool_batch_timeout_panic_escaped=true"),
        Ok(Err(error)) => println!(
            "tool_batch_timeout_panic_escaped=false returned_error={error}"
        ),
        Ok(Ok(value)) => {
            println!(
                "tool_batch_timeout_panic_escaped=false returned_success_len={}",
                value.len()
            );
            assert!(value.is_empty(), "current stream boundary returned unexpected content");
        }
    }
}

fn json_probe() {
    let malformed = r#"{"score":1,"passed":false,"feedback":"keep ,} literal",}"#;
    let cleaned = json_parse::clean_json(malformed);
    let parsed: serde_json::Value = serde_json::from_str(&cleaned)
        .unwrap_or_else(|error| panic!("cleaned fixture should parse: {error}"));
    println!("cleaned={cleaned}");
    assert_eq!(parsed["feedback"], "keep } literal");
    assert_ne!(parsed["feedback"], "keep ,} literal");

    let apostrophe = "{'score': 1, 'feedback': 'don't'}";
    let apostrophe_cleaned = json_parse::clean_json(apostrophe);
    println!(
        "apostrophe_cleaned={} parse_ok={}",
        apostrophe_cleaned,
        serde_json::from_str::<serde_json::Value>(&apostrophe_cleaned).is_ok()
    );
    assert!(serde_json::from_str::<serde_json::Value>(&apostrophe_cleaned).is_err());
}

fn time_hash_probe() {
    let winter: DateTime<Utc> = "2026-01-15T12:00:00Z"
        .parse()
        .unwrap_or_else(|error| panic!("winter fixture parse failed: {error}"));
    let summer: DateTime<Utc> = "2026-07-15T12:00:00Z"
        .parse()
        .unwrap_or_else(|error| panic!("summer fixture parse failed: {error}"));
    let winter_local = time::to_local(winter);
    let summer_local = time::to_local(summer);
    println!(
        "winter_offset={} summer_offset={} hash_hello={:016x}",
        winter_local.offset(),
        summer_local.offset(),
        fnv1a_64(b"hello")
    );
    assert_eq!(winter_local.offset().local_minus_utc(), -5 * 3600);
    assert_eq!(summer_local.offset().local_minus_utc(), -4 * 3600);
    assert_eq!(fnv1a_64(b"hello"), 0xa430_d846_80aa_bd0b);
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    match mode.as_str() {
        "budget" => budget_probe(),
        "retry-cancel" => retry_cancel_probe().await,
        "circuit" => circuit_probe(),
        "live-circuit" => live_circuit_probe().await,
        "live-retry-cancel" => live_retry_cancel_probe().await,
        "llm-backoff-overflow" => llm_backoff_overflow_probe().await,
        "tool-timeout-overflow" => tool_timeout_overflow_probe().await,
        "json" => json_probe(),
        "time-hash" => time_hash_probe(),
        other => panic!("unknown probe mode: {other}"),
    }
}
