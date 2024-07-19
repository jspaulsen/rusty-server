mod server;

use std::thread;
use std::time::{
    Duration, 
    Instant,
};

use server::{
    Message, 
    WebsocketServer,
};

const TICKS_PER_SECOND: u32 = 30;


// spawn thread
// run runtime
fn main() {
    let server = WebsocketServer::new("localhost", 3000)
        .run();

    let mut connections: Vec<uuid::Uuid> = vec![];

    println!("Starting server");

    loop {
        let messages = server.recv_next();

        for message in messages {
            println!("Received message in main: {:?}", message);

            match &message {
                Message::Connection(uid) => {
                    connections.push(*uid);
                },
                Message::Disconnection(uid) => {
                    connections.retain(|&x| x != *uid);
                },
                Message::Message(_, text) => {
                    for connection in &connections {
                        let nmessage = Message::Message(
                            *connection,
                            text.clone()
                        );

                        server.send(nmessage)
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(1));
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
