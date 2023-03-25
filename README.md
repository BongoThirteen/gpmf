# gpmf

## Parser and Writer for GoPro Metadata Format (GPMF)

WIP: Currently successfully parses all raw test data and logs the results.

## Design Goals

* Linux Philosophy, each tool does one thing and does it well
* Can be integrated in other tools, and in other languages than Rust
* Focus on clean, easy to understand code (no macros)
* Performant (but not at the expense of the previous item)
* Easy to read detailed log in order to be able to debug problems
* Memory safe parser
* Zero security vulnerabilities. Avoid problems found in other tools e.g.: [GoPro GPMF-parser Vulnerabilities](https://blog.inhq.net/posts/gopro-gpmf-parser-vuln-1/)
* Never generate exceptions, i.e.: Should never panic.
* Should pass fuzz tests i.e.: handle corrupt or junk data
* Should avoid DOS attacks. Possibly Add max buffer lengths.
* Gracefully recover from errors
* Handle unknown tags
* Roundtrip sensor data (without loss of precision or changing data type)

## Reporting Issues

If you have a file that is not handled please submit an issue, attaching the raw metadata file

## Feature Roadmap

* [x] Parser (WIP) at present just prints out data
* [ ] Create a structure to hold data
* [ ] Handle Scale
* [ ] Handle multiple sensor data 'mp4 boxes/atoms', as contained in mp4 file
* [ ] Return data in chronological order using Iterator and Tournament Tree
* [ ] Extract metadata from Live Stream via WiFi and Rtmp Url in realtime
* [ ] Handle exif data in images
* [ ] Writer
* [ ] Roundtrip sensor data

## Example

```rust
use std::path::Path;
use gpmf::byteorder_gpmf::parse_gpmf;

fn main() -> anyhow::Result<()> {
    let path = Path::new("samples/karma.raw");
    let text = std::fs::read(path)?;
    let res=parse_gpmf(text.as_slice())?;
    println!("{:?}",res);
    Ok(())
}
```

## Example with Logging

```rust
use std::path::Path;
use gpmf::byteorder_gpmf::parse_gpmf;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let path = Path::new("samples/Fusion.raw");
    let text = std::fs::read(path)?;
    let res=parse_gpmf(text.as_slice())?;
    println!("{:?}",res);
    Ok(())
}
```

License: MIT OR Apache-2.0
