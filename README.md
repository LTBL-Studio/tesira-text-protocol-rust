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

session.send_command(
    Command::builder()
        .standard_mixer("Mixer1")
        .set_outputlevel(1, -10)
).expect("Failed to set level")
```

### Value subscription

```rust
let mut session = TesiraSession::new_from_ssh("192.168.1.14", "admin", "mystrongpassword")
        .expect("Failed to open Tesira session");

let subscription = session.send_command()
    .unwrap();

session.send_command(
    Command::builder()
            .audio_meter("AudioMeter1")
            .subscribe_level(1, "MySubscription")
).unwrap();

loop {
    let token = session.recv_token().unwrap();
    println!("Value received: {:?}", token.value)
}
```

## Development

To update block type list from tesira command generator execute the following code and replace `tesira-blocks.json` with downloaded one.

```javascript
(() => {
    let cleanedBlocks = Object.fromEntries(Object.entries(blocks))
    for(let key in cleanedBlocks){
        for(let attr of cleanedBlocks[key].attributes){
            if(!attr.valuetype){
                attr.valuetype = "none"
            }
        }
    }

    let a = document.createElement("a")
    a.href = `data:application/json,${JSON.stringify(cleanedBlocks)}`
    a.download = "tesira-blocks.json"
    a.click()
})()
```