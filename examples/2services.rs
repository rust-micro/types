use std::thread;
use std::thread::sleep;
use types::String;

fn main() {
    thread::scope(|s| {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let client2 = client.clone();

        let t1 = s.spawn(move || {
            let mut string = String::with_value("Hello".to_string(), "test", client);
            println!("Thread1: {}", string.cached().unwrap());
            assert_eq!(string, "Hello");
            sleep(std::time::Duration::from_secs(1));
            string.store("World".to_string());
            println!("Thread1: {}", string.cached().unwrap());
            assert_eq!(string, "World");
        });

        let t2 = s.spawn(move || {
            sleep(std::time::Duration::from_micros(100));
            let mut string = String::with_load("test", client2);
            println!("Thread2: {}", string.cached().unwrap());
            assert_eq!(string, "Hello");
            sleep(std::time::Duration::from_secs(2));
            string.acquire();
            println!("Thread2: {}", string.cached().unwrap());
            assert_eq!(string, "World");
        });
        t1.join().expect("Failed to join thread1");
        t2.join().expect("Failed to join thread2");
    });
}
