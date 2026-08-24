use echo_agent::{error::Result, guard, guard::{GuardDirection, GuardResult}};

#[guard(name = "1-invalid")]
async fn invalid_name(_content: &str, _direction: GuardDirection) -> Result<GuardResult> {
    Ok(GuardResult::Pass)
}
