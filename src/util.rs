use eyre::Result;
use std::time::Instant;

pub fn timed_scope<R, F: FnOnce() -> Result<R>>(label: &str, fun: F) -> Result<R> {
    let start = Instant::now();

    let res = fun()?;

    let time = Instant::now().duration_since(start);
    println!("{label} took: {time:?}");

    Ok(res)
}
