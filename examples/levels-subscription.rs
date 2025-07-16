use std::{io::{stdin, stdout, BufRead, Write}, thread, time::Duration};

use tesira_text_protocol::TesiraSession;

fn inquire(what: &str) -> Option<String> {
    print!("{}: ", what);
    stdout().flush().unwrap();
    let value = stdin().lock().lines().next().unwrap().unwrap().trim().to_owned();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn main() {
    let hostname = inquire("Device hostname").expect("Device hostname is mendatory");
    let username = inquire("Username [admin]").unwrap_or_else(|| "admin".to_owned());
    let password = inquire("Password").expect("Password is mendatory");

    let mut session = TesiraSession::new_from_ssh(hostname, username, password)
        .expect("Failed to open Tesira session");

    println!("Session opened");

    let subscription = session.subscribe_with_rate("AudioMeter1", "level", Some(1), Duration::from_secs(1))
        .unwrap();

    println!("Subscribed to AudioMeter1 level 1");

    thread::spawn(move || {
        loop {
            match subscription.recv() {
                Err(e) => {
                    println!("Channel closed: {e}");
                    break;
                },
                Ok(t) => println!("Value received: {:?}", t.value)
            }
        }
    });

    loop {
        session.dispatch_next_token()
            .unwrap()
    }

}