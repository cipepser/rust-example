use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    let (sender, receiver) = mpsc::channel();

    let numbers = vec![1, 10, 4, 7, 3, 6, 9, 2, 5, 8, 3];

    let mut handles: Vec<_> = numbers.into_iter()
        .map(|n| {
            let sender = sender.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(n));
                sender.send(n).unwrap();
            })
        }).collect();

    handles.push(thread::spawn(move || {
        loop {
            match receiver.recv() {
                Ok(n) => {
                    println!("{:?} ", n);
                }
                Err(_) => {
                    break;
                }
            }
        }
    }));

    for h in handles {
        h.join().unwrap();
    }
}
