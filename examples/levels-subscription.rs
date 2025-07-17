use std::io::{BufRead, Write, stdin, stdout};

use tesira_text_protocol::{TesiraSession, proto::Command};

fn inquire(what: &str) -> Option<String> {
    print!("{what}: ");
    stdout().flush().unwrap();
    let value = stdin()
        .lock()
        .lines()
        .next()
        .unwrap()
        .unwrap()
        .trim()
        .to_owned();
    if value.is_empty() { None } else { Some(value) }
}

fn main() {
    let hostname = inquire("Device hostname").expect("Device hostname is mendatory");
    let username = inquire("Username [admin]").unwrap_or_else(|| "admin".to_owned());
    let password = inquire("Password").expect("Password is mendatory");

    let mut session = TesiraSession::new_from_ssh(&hostname, &username, &password)
        .expect("Failed to open Tesira session");

    println!("Session opened");

    let aliases = session.get_aliases().unwrap();
    println!("Available aliases: {aliases:#?}");

    session
        .send_command(
            Command::builder()
                .audio_meter("AudioMeter1")
                .subscribe_level(1, "MySubscription"),
        )
        .unwrap();

    println!("Subscribed to AudioMeter1 level 1");

    loop {
        let token = session.recv_token().unwrap();
        println!("Value received: {:?}", token.value)
    }
}
