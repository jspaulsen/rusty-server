mod server;
mod queue;

use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::Builder;
use server::WebsocketServer;

const TICKS_PER_SECOND: u32 = 30;


// spawn thread
// run runtime
fn main() {
    let server = WebsocketServer::new();
    let mut outgoing = server.outgoing_queue();
    let mut incoming = server.incoming_queue();

    // let 
    // either decompose the server into an incoming and outgoing queue while it runs
    // and/or have the server run in a separate thread
    // ALSO: It could decompose into an object, one with a flag for stopping the server
    // the input/output queues and the handle so we can join on it.
    // TODO: Maybe run spawns the thread and returns a handle
    thread::spawn(move || {
        server.run();
    });

    loop {
        thread::sleep(Duration::from_millis(1));
        // println!("tick")
        // perform some work; if the time between now and the last tick is less than the duration per second, sleep for the difference
        // but don't actually sleep; we should sleep 1ms and check again
    }
}


// fn main() {
//     // let duration_per_second: Duration = Duration::from_secs(1) / TICKS_PER_SECOND;
//     // let ticks: u128 = 0;

//     // let mut last_tick = Instant::now();
//     // let mut do_work = true;

//     // perform some work; if the time between now and the last tick is less than the duration per second, sleep for the difference
//     // but don't actually sleep; we should sleep 1ms and check again

//     loop {
//         thread::sleep(Duration::from_millis(1));
//         // println!("tick")
//         // perform some work; if the time between now and the last tick is less than the duration per second, sleep for the difference
//         // but don't actually sleep; we should sleep 1ms and check again

//         // TODO: Do work
//         // println!("tick");
//         // do_work = false;

//         // let now = Instant::now();
//         // let elapsed = now.duration_since(last_tick);

//         // if elapsed < duration_per_second {
//         //     thread::sleep(duration_per_second - elapsed);
//         // }

        


//         // println!("tick");
//         // let now = Instant::now();
//         // let elapsed = now.duration_since(last_tick);

//         // if elapsed < duration_per_second {
//         //     thread::sleep(duration_per_second - elapsed);
//         // }

//         // last_tick = Instant::now();
//     }
// }
