use std::future::Future;

pub fn init() {
    let env = env_logger::Env::default().default_filter_or("pubnub=trace");
    let _ = env_logger::Builder::from_env(env).is_test(true).try_init();
}

pub fn current_thread_block_on<F: Future>(future: F) -> F::Output {
    let mut rt = tokio::runtime::Builder::new()
        .enable_all()
        .basic_scheduler()
        .build()
        .expect("unable to build tokio runtime");
    rt.block_on(future)
}
