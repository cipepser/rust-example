extern crate rand;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use rand::{thread_rng, Rng};

fn main() {
    let (sender, receiver) = mpsc::channel();

    let names = vec!["Alice", "Bob", "Charlie"];

    let mut handles: Vec<_> = names.into_iter()
        .map(|name| {
            let sender = sender.clone();
            thread::spawn(move || {
                let sleep_time: u64 = thread_rng().gen_range(1, 6);
                thread::sleep(Duration::from_secs(sleep_time));
                println!("send: {:?}", name);
                sender.send(name).unwrap();
            })
        }).collect();

    handles.push(thread::spawn(move || {
        thread::sleep(Duration::from_secs(7));
        loop {
            match receiver.recv() {
                Ok(name) => {
                    println!("receive: {:?}", name);
                }
                _ => {
                    continue;
                }
            }
        }
    }));


    for h in handles {
        h.join().unwrap();
    }
}
