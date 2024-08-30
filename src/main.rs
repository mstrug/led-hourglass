/*fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");
}*/


use edge_executor::LocalExecutor;
use edge_executor::Executor;
use esp_idf_svc::hal::task::block_on;
use async_channel::unbounded;
use async_channel::{Receiver, Sender};
//use gloo_timers::future::TimeoutFuture
use std::time::Duration;
use futures_timer::Delay;


async fn task1(r: Receiver<u8>) {
    loop {
	if let Ok(v) = r.recv().await {
	    log::info!("task1 {}", v);
	} else {
	    break;
	}
    }
}

async fn task2(s: Sender<u8>) {
    for i in 1..=3 {
        log::info!("task2 send: {}", i);
        s.send(i as u8).await;
        Delay::new(Duration::from_secs(1)).await;
    }
}

async fn app(rt: &Executor<'_>) {
//    let rt: Executor = Default::default();
    //let rt: LocalExecutor = Default::default();
    log::info!("app started");

    let (s, r) = async_channel::unbounded::<u8>();

    rt.spawn(task1(r)).detach();

    log::info!("spawned task1");

    let task2 = rt.spawn(task2(s));

    log::info!("spawned task2");

    //edge_executor::block_on(task2.await);
    task2.await;

    log::info!("app end");
}


fn main() {
    esp_idf_svc::sys::link_patches();
    
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Started");

    let ex: Executor = Executor::new();

    let future = app(&ex);

    log::info!("Blocking on app.");
    
//    future.await.detach();

//    edge_executor::block_on(ex.run(core::future::pending::<()>()));
    edge_executor::block_on(ex.run(future));

    // Borrowed by `&mut` inside the future spawned on the executor
/*    let mut data = 3;

    let data = &mut data;

    let task = local_ex.spawn(async move {
        *data += 1;

        *data
    });

    let res = block_on(local_ex.run(async { task.await * 2 }));

    assert_eq!(res, 8);*/
}
