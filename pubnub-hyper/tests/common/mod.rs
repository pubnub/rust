use std::future::Future;

pub fn init() {
    pubnub_test_util::init_log();
}

pub fn current_thread_block_on<F: Future>(future: F) -> F::Output {
    let mut rt = tokio::runtime::Builder::new()
        .enable_all()
        .basic_scheduler()
        .build()
        .expect("unable to build tokio runtime");
    rt.block_on(future)
}
