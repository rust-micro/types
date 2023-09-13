use std::thread;
use types::String;

fn main() {
    thread::scope(|s| {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let client2 = client.clone();

        s.spawn(move || {
            let string = String::with_value("Hello".to_string(), "test", client);
            // prints out "Hello"
            println!("string: {}", string.cached().unwrap());
            assert_eq!(string, "Hello");
        })
        .join()
        .expect("Failed to join thread1");

        s.spawn(move || {
            let string = String::with_value_load("test", client2);
            // prints out "Hello"
            println!("string: {}", string.cached().unwrap());
            assert_eq!(string, "Hello");
        })
        .join()
        .expect("Failed to join thread2");
    });
}
