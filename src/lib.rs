//! # Parser and Writer for GoPro Metadata Format (GPMF)
//!
//! WIP: Currently successfully parses all raw test data and logs the results.
//!
//! # Design Goals
//!
//! * Linux Philosophy, each tool does one thing and does it well
//! * Can be integrated in other tools, and in other languages than Rust
//! * Focus on clean, easy to understand code (no macros)
//! * Performant (but not at the expense of the previous item)
//! * Easy to read detailed log in order to be able to debug problems
//! * Memory safe parser
//! * Zero security vulnerabilities. Avoid problems found in other tools e.g.: [GoPro GPMF-parser Vulnerabilities](https://blog.inhq.net/posts/gopro-gpmf-parser-vuln-1/)
//! * Never generate exceptions, i.e.: Should never panic.
//! * Should pass fuzz tests i.e.: handle corrupt or junk data
//! * Should avoid DOS attacks. Possibly Add max buffer lengths.
//! * Gracefully recover from errors
//! * Handle unknown tags
//! * Roundtrip sensor data (without loss of precision or changing data type)
//!
//! # Reporting Issues
//!
//! If you have a file that is not handled please submit an issue, attaching the raw metadata file
//!
//! # Feature Roadmap
//!
//! * [x] Parser (WIP) at present just prints out data
//! * [ ] Create a structure to hold data
//! * [ ] Handle Scale
//! * [ ] Handle multiple sensor data 'mp4 boxes/atoms', as contained in mp4 file
//! * [ ] Return data in chronological order using Iterator and Tournament Tree
//! * [ ] Extract metadata from Live Stream via WiFi and Rtmp Url in realtime
//! * [ ] Handle exif data in images
//! * [ ] Writer
//! * [ ] Roundtrip sensor data
//!
//! # Example
//!
//! ```
//! use std::path::Path;
//! use gpmf::byteorder_gpmf::parse_gpmf;
//!
//! fn main() -> anyhow::Result<()> {
//!     let path = Path::new("samples/karma.raw");
//!     let text = std::fs::read(path)?;
//!     let res=parse_gpmf(text.as_slice())?;
//!     println!("{:?}",res);
//!     Ok(())
//! }
//! ```
//!
//! # Example with Logging
//!
//! ```
//! use std::path::Path;
//! use gpmf::byteorder_gpmf::parse_gpmf;
//! use tracing::Level;
//! use tracing_subscriber::FmtSubscriber;
//!
//! fn main() -> anyhow::Result<()> {
//!     let subscriber = FmtSubscriber::builder()
//!         .with_max_level(Level::DEBUG)
//!         .finish();
//!     tracing::subscriber::set_global_default(subscriber)?;
//!
//!     let path = Path::new("samples/Fusion.raw");
//!     let text = std::fs::read(path)?;
//!     let res=parse_gpmf(text.as_slice())?;
//!     println!("{:?}",res);
//!     Ok(())
//! }
//! ```

//#![nopanic]
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]
#![feature(cursor_remaining)]
#![feature(buf_read_has_data_left)]

pub mod byteorder_gpmf;
mod entry;
mod key_value;
mod models;
mod tags;
mod types;
mod values;

pub use entry::Entry;
pub use key_value::KeyValue;
pub use models::Model;
pub use models::Model;
pub use tags::Tag;
pub use types::Type;

use chrono::{DateTime, Utc};
use fixed::types::{I16F16, I32F32};

use num_enum::TryFromPrimitive;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};
use tracing::{debug, enabled, error, info, span, trace, warn, Level};

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::Path;

    use std::sync::Once;
    // use chrono::{TimeZone, Utc};
    use chrono::{TimeZone, Utc};

    use fixed::prelude::*;
    use fixed::types::{I16F16, I32F32};
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    static INIT: Once = Once::new();

    // pub(crate) fn setup() {
    //     // INIT.call_once(env_logger::init);
    //     INIT.call_once(||{
    //         SimpleLogger::new().with_level(LevelFilter::Debug).without_timestamps().init().unwrap()
    //     });
    // }

    pub(crate) fn setup() {
        // INIT.call_once(env_logger::init);
        INIT.call_once(|| {
            let subscriber = FmtSubscriber::builder()
                // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
                // will be written to stdout.
                .with_max_level(Level::TRACE)
                //.with_max_level(Level::DEBUG)
                // completes the builder.
                .finish();

            tracing::subscriber::set_global_default(subscriber)
                .expect("setting default subscriber failed");
        });
    }

    #[test]
    fn test_type() {
        setup();
        Type::iter().for_each(|t| {
            info!("{} {:?}", t, t);
        });
    }

    // #[test]
    // fn test_fourcc() {
    //     let (res, f) = fourcc("DEVC".as_bytes()).unwrap();
    //
    //     assert_eq!(f, FourCC::DEVC);
    // }

    #[test]
    fn test_file() -> anyhow::Result<()> {
        let dir = Path::new("samples");
        let path = dir.join("hero5.raw");

        // let input=BufReader::new(file);
        let text = std::fs::read(path).unwrap();

        text.iter().enumerate().for_each(|(i, b)| {
            print!("{}\t{}", i, b);
            if b.is_ascii() {
                println!("\t{}", *b as char);
            } else {
                println!()
            }
        });

        Ok(())
    }

    #[test]
    fn test_take_until() {
        let data = [b'a', b'b', b'c', 0, b'd', 0];
        let until_null: Vec<_> = data.into_iter().take_while(|b| *b != 0).collect();
        let string = String::from_utf8(until_null).unwrap();
        assert_eq!(string, String::from("abc"));
    }

    #[test]
    fn test_take_until_no_null() {
        let data = [b'a', b'b', b'c'];
        let until_null: Vec<_> = data.into_iter().take_while(|b| *b != 0).collect();
        let string = String::from_utf8(until_null).unwrap();
        assert_eq!(string, String::from("abc"));
    }

    //q 	32-bit Q Number Q15.16 	uint32_t 	16-bit integer (A) with 16-bit fixed point (B) for A.B value (range -32768.0 to 32767.99998)
    // Q 	64-bit Q Number Q31.32 	uint64_t 	32-bit integer (A) with 32-bit fixed point (B) for A.B value.

    //use fixed::types::I16F16; //is a 32-bit fixed-point signed number with 20 integer bits and 12 fractional bits

    #[test]
    fn test_q32() {
        //-32768.0 to 32767.99998
        let max = I16F16::from_bits(i32::MAX);

        println!("Max {}", max);
        let min = I16F16::from_bits(i32::MIN);
        println!("Min {}", min);

        let sum = max + min;
        println!("Sum {}", sum);

        // let max_f64:f64=max.into();
        let max_f64 = f64::from_fixed(max);
        println!("Max f64 {}", max_f64);

        let min_f64 = f64::from_fixed(min);
        println!("Min f64 {}", min_f64);

        let max_f32 = f32::from_fixed(max);
        println!("Max f32 {}", max_f32);

        let min_f32 = f32::from_fixed(min);
        println!("Min f32 {}", min_f32);
    }

    #[test]
    fn test_q64() {
        //-32768.0 to 32767.99998
        let max = I32F32::from_bits(i64::MAX);

        println!("Max {}", max);
        let min = I32F32::from_bits(i64::MIN);
        println!("Min {}", min);

        let sum = max + min;
        println!("Sum {}", sum);

        // let max_f64:f64=max.into();
        let max_f64 = f64::from_fixed(max);
        println!("Max f64 {}", max_f64);

        let min_f64 = f64::from_fixed(min);
        println!("Min f64 {}", min_f64);

        let max_f32 = f32::from_fixed(max);
        println!("Max f32 {}", max_f32);

        let min_f32 = f32::from_fixed(min);
        println!("Min f32 {}", min_f32);
    }

    #[test]
    fn test_date() {
        //16 byte
        // yymmddhhmmss.sss
        // yymmddhhmmss.sss

        let date_str = "230323191804.123";
        println!("{}", date_str);
        let no_timezone = Utc.datetime_from_str(date_str, DATE_FORMAT).unwrap();
        println!("{}", no_timezone);

        let roundtrip = no_timezone.format(DATE_FORMAT).to_string();
        println!("{}", roundtrip);
    }
}
