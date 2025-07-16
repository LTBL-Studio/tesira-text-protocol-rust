# Rust Tesira Text Protocol

> Implementation of [Tesira Text Protocol](https://support.biamp.com/Tesira/Control/Tesira_Text_Protocol) in Rust

**Cargo features**

* **ssh** use ssh2 to connect to tesira devices with ssh [default]

## Quick Start

When connected, you can get, set, toggle, incremenet and decrement values.

```rust
let mut session = TesiraSession::new_from_ssh("192.168.1.14", "admin", "mystrongpassword")
        .expect("Failed to open Tesira session");

let aliases = session.get_aliases().unwrap();
println!("Available aliases: {:#?}", aliases);

session.set("Mixer1", "outputLevel", ["1", "-10"])
    .expect("Failed to set level")
```

### Value subscription

TesiraSession is designed to allow multithreaded application to monitor values from a single connection.
Call to `subscribe` returns a channel that dispatch values updates.

```rust
let mut session = TesiraSession::new_from_ssh("192.168.1.14", "admin", "mystrongpassword")
        .expect("Failed to open Tesira session");

let subscription = session.subscribe("AudioMeter1", "level", Some(1))
    .unwrap();

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
    // You still need to call "dispatch_next_token" regularly
    session.dispatch_next_token()
        .unwrap()
}
```