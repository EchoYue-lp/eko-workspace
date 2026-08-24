use echo_agent::callback;

struct Callback<T>(T);

#[callback]
impl<T> Callback<T>
where
    T: Send + Sync,
{
    async fn on_final_answer(&self, _agent: &str, _answer: &str) {}
}
