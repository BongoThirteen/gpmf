//! ## Parser and Writer for GoPro Metadata Format (GPMF)
//!
//! WIP: Currently successfully parses all raw test data and logs the results.
//!
//! ## Design Goals
//!
//! * Linux Philosophy, each tool does one thing and does it well
//! * Can be integrated in other tools, and in other languages than Rust
//! * Focus on clean, easy to understand code (no macros)
//! * Performant (but not at the expense of the previous item)
//! * Easy to read detailed log in order to be able to debug problems
//! * Memory safe parser
//! * Zero security vulnerabilities. Avoid problems found in other tools e.g.: [GoPro GPMF-parser Vulnerabilities](https://blog.inhq.net/posts/gopro-gpmf-parser-vuln-1/)
//! * Never generate exceptions, i.e.: Should never panic.
//! * Should pass fuzz tests i.e.: handle junk data
//! * Should avoid DOS attacks. Possibly Add max buffer lengths.
//! * Gracefully recover from errors
//! * Handle unknown tags
//!
//! ## Reporting Issues
//!
//! If you have a file that is not handled please submit an issue, attaching the raw metadata file
//!
//! ## Feature Roadmap
//!
//! * [x] Parser (WIP) at present just prints out data
//! * [ ] Create a structure to hold data
//! * [ ] Handle Scale
//! * [ ] Handle multiple sensor data 'mp4 boxes/atoms', as contained in mp4 file
//! * [ ] Return data in chronological order using Iterator and Tournament Tree
//! * [ ] Stream data
//! * [ ] Handle image exif data
//! * [ ] Writer

#![feature(cursor_remaining)]
#![feature(buf_read_has_data_left)]

mod byteorder_gpmf;

use chrono::{DateTime, Utc};
use fixed::types::{I16F16, I32F32};

use num_enum::TryFromPrimitive;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};
use tracing::{debug, enabled, error, info, span, trace, warn, Level};

const DATE_FORMAT: &str = "%y%m%d%H%M%S%.3f";

/// Key Value struct (not used at present)
#[derive(Debug)]
pub struct KeyValue {
    key: FourCC,
    //typ:Type,
    value: Value,
}

/// The data type of the sensor data
#[derive(Debug, PartialEq, Eq, EnumIter, EnumString, Display, TryFromPrimitive)]
#[repr(u8)]
pub enum Type {
    /// | **b** | single byte signed integer | int8\_t | -128 to 127 |
    // #[strum(serialize="b")]
    I8 = b'b',
    /// | **B** | single byte unsigned integer | uint8\_t | 0 to 255 |
    // #[strum(serialize="B")]
    U8 = b'B',
    /// | **c** | single byte &#39;c&#39; style ASCII character string | char | Optionally NULL terminated - size/repeat sets the length |
    // #[strum(serialize="c")]
    Char = b'c',
    /// | **d** | 64-bit double precision (IEEE 754) | double |   |
    // #[strum(serialize="d")]
    F64 = b'd',
    /// | **f** | 32-bit float (IEEE 754) | float |   |
    // #[strum(serialize="f")]
    F32 = b'f',
    /// | **F** | 32-bit four character key -- FourCC | char fourcc\[4\] |   |
    // #[strum(serialize="F")]
    FourCC = b'F',
    /// | **G** | 128-bit ID (like UUID) | uint8\_t guid\[16\] |   |
    // #[strum(serialize="G")]
    U128 = b'G',
    /// | **j** | 64-bit signed unsigned number | int64\_t |   |
    // #[strum(serialize="j")]
    I64 = b'j',
    /// | **J** | 64-bit unsigned unsigned number | uint64\_t |   |
    // #[strum(serialize="J")]
    U64 = b'J',
    /// | **l** | 32-bit signed integer | int32\_t |   |
    // #[strum(serialize="l")]
    I32 = b'l',
    /// | **L** | 32-bit unsigned integer | uint32\_t |   |
    // #[strum(serialize="L")]
    U32 = b'L',
    /// | **q** | 32-bit Q Number Q15.16 | uint32\_t | 16-bit integer (A) with 16-bit fixed point (B) for A.B value (range -32768.0 to 32767.99998) |
    // #[strum(serialize="q")]
    Fixed32 = b'q',
    /// | **Q** | 64-bit Q Number Q31.32 | uint64\_t | 32-bit integer (A) with 32-bit fixed point (B) for A.B value. |
    // #[strum(serialize="Q")]
    Fixed64 = b'Q',
    /// | **s** | 16-bit signed integer | int16\_t | -32768 to 32768 |
    // #[strum(serialize="s")]
    I16 = b's',
    /// | **S** | 16-bit unsigned integer | uint16\_t | 0 to 65536 |
    // #[strum(serialize="S")]
    U16 = b'S',
    /// | **U** | UTC Date and Time string | char utcdate\[16\] | Date + UTC Time format yymmddhhmmss.sss - (years 20xx covered) |
    // #[strum(serialize="U")]
    Date = b'U',
    /// | **?** | data structure is complex | TYPE | Structure is defined with a preceding TYPE |
    // #[strum(serialize="?")]
    Complex = b'?',
    /// | **null** | Nested metadata | uint32\_t | The data within is GPMF structured KLV data |
    // #[strum(serialize="\0")]
    Nested = b'\0',
}

impl Type {
    /// The size of the sensor data in bytes
    //TODO maybe this should be an option and return None for Nested and Complex
    pub fn size(&self) -> usize {
        match &self {
            Type::I8 | Type::U8 | Type::Char => 1,
            Type::I16 | Type::U16 => 2,
            Type::F32 | Type::FourCC | Type::I32 | Type::U32 | Type::Fixed32 => 4,
            Type::F64 | Type::I64 | Type::U64 | Type::Fixed64 => 8,

            Type::U128 | Type::Date => 16,
            Type::Complex => {
                warn!("COMPLEX SIZE NOT KNOWN actually depends on previous TYPE definition");
                1
            }
            Type::Nested => {
                warn!("NESTED SIZE NOT KNOWN");
                1
            }
        }
    }
}

/// The Camera Model
///
/// This enum is so that the user can intepret fields that change order and or sign
#[derive(Debug, PartialEq, EnumString, EnumIter, Display)]
pub enum Model {
    Hero5,
    //TODO add other common camera models
    Other(String),
}

/// The value of the data,
#[derive(Debug)]
pub enum Value {
    //| **b** | single byte signed integer | int8\_t | -128 to 127 |
    I8(i8),
    // | **B** | single byte unsigned integer | uint8\_t | 0 to 255 |
    U8(u8),
    // | **c** | single byte &#39;c&#39; style ASCII character string | char | Optionally NULL terminated - size/repeat sets the length |
    Char(char),
    String(String),
    // | **d** | 64-bit double precision (IEEE 754) | double |   |
    F64(f64),
    // | **f** | 32-bit float (IEEE 754) | float |   |
    F32(f32),
    // | **F** | 32-bit four character key -- FourCC | char fourcc\[4\] |   |
    FourCC(FourCC),
    // | **G** | 128-bit ID (like UUID) | uint8\_t guid\[16\] |   |
    U128(u128),
    // | **j** | 64-bit signed unsigned number | int64\_t |   |
    I64(i64),
    // | **J** | 64-bit unsigned unsigned number | uint64\_t |   |
    U64(u64),
    // | **l** | 32-bit signed integer | int32\_t |   |
    I32(i32),
    // | **L** | 32-bit unsigned integer | uint32\_t |   |
    U32(u32),
    // | **q** | 32-bit Q Number Q15.16 | uint32\_t | 16-bit integer (A) with 16-bit fixed point (B) for A.B value (range -32768.0 to 32767.99998) |
    Fixed32(I16F16),
    // | **Q** | 64-bit Q Number Q31.32 | uint64\_t | 32-bit integer (A) with 32-bit fixed point (B) for A.B value. |
    Fixed64(I32F32),
    // | **s** | 16-bit signed integer | int16\_t | -32768 to 32768 |
    I16(i16),
    // | **S** | 16-bit unsigned integer | uint16\_t | 0 to 65536 |
    U16(u16),
    // | **U** | UTC Date and Time string | char utcdate\[16\] | Date + UTC Time format yymmddhhmmss.sss - (years 20xx covered) |
    Date(DateTime<Utc>),
    // | **?** | data structure is complex | TYPE | Structure is defined with a preceding TYPE |
    Complex(Vec<Value>),
    // | **null** | Nested metadata | uint32\_t | The data within is GPMF structured KLV data |
    Nested(Vec<KeyValue>),
}

impl Value {
    /// The datatype of the value
    pub fn datatype(&self) -> Type {
        match self {
            Value::I8(_) => Type::I8,
            Value::U8(_) => Type::U8,
            Value::Char(_) => Type::Char,
            Value::String(_) => Type::Char,
            Value::F64(_) => Type::F64,
            Value::F32(_) => Type::F32,
            Value::FourCC(_) => Type::FourCC,
            Value::U128(_) => Type::U128,
            Value::I64(_) => Type::I64,
            Value::U64(_) => Type::U64,
            Value::I32(_) => Type::I32,
            Value::U32(_) => Type::U32,
            Value::Fixed32(_) => Type::Fixed32,
            Value::Fixed64(_) => Type::Fixed64,
            Value::I16(_) => Type::I16,
            Value::U16(_) => Type::U16,
            Value::Date(_) => Type::Date,
            Value::Complex(_) => Type::Complex,
            Value::Nested(_) => Type::Nested,
        }
    }
}

/// The FourCC key of the data
///
/// There are some undocumented tags present in GPMF data.
/// Currently warnings are logged for unsupported tags.
#[derive(Debug, PartialEq, EnumString, EnumIter, Display)]
pub enum FourCC {
    // #[strum(serialize = "blue", serialize = "b")]
    ///unique device source for metadata
    /// Each connected device starts with DEVC. A GoPro camera or Karma drone would have their own DEVC for nested metadata to follow. |
    #[strum(serialize = "DEVC", to_string = "Device")]
    DEVC,

    /// device/track ID
    /// Auto generated unique-ID for managing a large number of connect devices, camera, karma and external BLE devices |
    #[strum(serialize = "DVID", to_string = "Device ID")]
    DVID,

    /// device name
    /// Display name of the device like &quot;Karma 1.0&quot;, this is for communicating to the user the data recorded, so it should be informative. |
    #[strum(serialize = "DVNM", to_string = "Device Name")]
    DVNM,

    ///Nested signal stream of metadata/telemetry
    ///Metadata streams are each nested with STRM
    #[strum(serialize = "STRM", to_string = "Stream")]
    STRM,

    ///Stream name
    /// Display name for a stream like &quot;GPS RAW&quot;, this is for communicating to the user the data recorded, so it should be informative. |
    #[strum(serialize = "STNM", to_string = "Stream Name")]
    STNM,

    ///Comments for any stream
    /// Add more human readable information about the stream |
    #[strum(serialize = "RMRK", to_string = "Stream Comment")]
    RMRK,

    ///Scaling factor (divisor) | Sensor data often needs to be scaled to be presented with the correct units. SCAL is a divisor. |
    #[strum(serialize = "SCAL", to_string = "Scale Factor")]
    SCAL,

    ///Standard Units (like SI) | If the data can be formatted in GPMF&#39;s standard units, this is best. E.g. acceleration as &quot;m/s²&quot;.  SIUN allows for simple format conversions. |
    #[strum(serialize = "SIUN", to_string = "SI Unit")]
    SIUN,

    /// Display units
    /// While SIUN is preferred, not everything communicates well via standard units. E.g. engine speed as &quot;RPM&quot; is more user friendly than &quot;rad/s&quot;. |
    #[strum(serialize = "UNIT", to_string = "Non SI Unit")]
    UNIT,
    ///Typedefs for complex structures
    /// Not everything has a simple repeating type. For complex structure TYPE is used to describe the data packed within each sample. |
    #[strum(serialize = "TYPE", to_string = "Type")]
    TYPE,
    ///Total Samples delivered | Internal field that counts all the sample delivered since record start, and is automatically computed. |
    #[strum(serialize = "TSMP", to_string = "Total Samples")]
    TSMP,
    ///Time Offset | Rare. An internal field that indicates the data is delayed by 'x' seconds. |
    #[strum(serialize = "TIMO", to_string = "Time Offset")]
    TIMO,
    ///Empty payload count
    /// Internal field that reports the number of payloads that contain no new data. TSMP and EMPT simplify the extraction of clock. |
    #[strum(serialize = "EMPT", to_string = "Empty Payload Count")]
    EMPT,

    #[strum(serialize = "TICK", to_string = "Start Timestamp")]
    TICK,
    #[strum(serialize = "TOCK", to_string = "End Timestamp")]
    TOCK,

    // thermal clock drift for temperature sensitive calibrations.
    #[strum(serialize = "TMPC", to_string = "Temp")]
    TMPC,

    //HERO5 Black and Session

    //3-axis accelerometer
    // Hero5: Data order Z,X,Y
    // Fusion: Data order -Y,X,Z
    // Hero 6 Data order Y,-X,Z
    #[strum(serialize = "ACCL", to_string = "Accel")]
    ACCL,
    // 3-axis gyroscope
    // Hero5: Data order Z,X,Y
    // Fusion: Data order -Y,X,Z
    // Hero6 Data order Y,-X,Z
    #[strum(serialize = "GYRO", to_string = "Gyro")]
    GYRO,
    //Image sensor gain
    #[strum(serialize = "ISOG", to_string = "Image Sensor Gain")]
    ISOG,
    //Exposure time
    #[strum(serialize = "SHUT", to_string = "Exposure Time")]
    SHUT,

    //HERO5 Black with GPS Enabled Adds

    //latitude, longitude, altitude (WGS 84), 2D ground speed, and 3D speed
    #[strum(serialize = "GPS5", to_string = "GPS 5")]
    GPS5,

    //UTC time and data from GPS
    #[strum(serialize = "GPSU", to_string = "GPS UTC")]
    GPSU,
    //GPS Fix Within the GPS stream: 0 - no lock, 2 or 3 - 2D or 3D Lock
    #[strum(serialize = "GPSF", to_string = "GPS Fix")]
    GPSF,
    //GPS Precision - Dilution of Precision (DOP x100) Within the GPS stream, under 500 is good
    #[strum(serialize = "GPSP", to_string = "GPS DOP")]
    GPSP,

    //Fusion Adds and Changes
    //magnetometer
    //GoPro MAX  Camera pointing direction x,y,z (valid in v2.0 firmware.)
    #[strum(serialize = "MAGN", to_string = "Magnetometer")]
    MAGN,

    //FUSION
    //microsecond timestamps
    #[strum(serialize = "STMP", to_string = "Timestamp")]
    STMP,

    //HERO6 Black
    //Face detection boundaring boxes
    // Herd6 struct ID,x,y,w,h -- not supported in HEVC modes
    // Hero 7 struct ID,x,y,w,h,unused[17],smile
    // Hero 8 struct ID,x,y,w,h,confidence %,smile %
    // Hero 10 struct ver,confidence %,ID,x,y,w,h,smile %, blink %
    #[strum(serialize = "FACE", to_string = "Face Bbox")]
    FACE,
    // Faces counted per frame
    #[strum(serialize = "FCNM", to_string = "Face count per Frame")]
    FCNM,

    #[strum(serialize = "FSTM", to_string = "UNDOCUMENTED Face something ???")]
    FSTM,

    //Sensor ISO replaces ISOG, has the same function
    #[strum(serialize = "ISOE", to_string = "Image Sensor Gain E")]
    ISOE,

    //Auto Low Light frame Duration
    #[strum(serialize = "ALLD", to_string = "Auto Low Light frame Duration")]
    ALLD,

    //White Balance in Kelvin
    #[strum(serialize = "WBAL", to_string = "White Balance in Kelvin")]
    WBAL,
    //White Balance RGB gains
    #[strum(serialize = "WRGB", to_string = "White Balance RGB gains")]
    WRGB,

    //HERO7 Black (v1.8)
    //Luma (Y) Average over the frame
    #[strum(serialize = "YAVG", to_string = "Luma (Y) Average over the frame")]
    YAVG,

    // Predominant hues over the frame
    // struct ubyte hue, ubyte weight, HSV_Hue = hue x 360/255
    #[strum(serialize = "HUES", to_string = "Predominant hues over the frame")]
    HUES,

    //Image uniformity
    #[strum(serialize = "UNIF", to_string = "Image uniformity")]
    UNIF,

    //Scene classifier in probabilities
    //FourCC scenes: SNOW, URBAn, INDOor, WATR, VEGEtation, BEACh
    #[strum(serialize = "SCEN", to_string = "Scene classifier")]
    SCEN,

    //Sensor Read Out Time
    #[strum(serialize = "SROT", to_string = "Sensor Read Out Time")]
    SROT,

    // HERO8 Black (v2.5)

    //Camera ORIentation
    // Quaternions for the camera orientation since capture start
    #[strum(serialize = "CORI", to_string = "Camera Orientation")]
    CORI,

    #[strum(
        serialize = "ORIO",
        to_string = "UNDOCUMENTED: ORIO Camera Orientation ???"
    )]
    ORIO,
    #[strum(
        serialize = "ORIN",
        to_string = "UNDOCUMENTED: ORIN Camera Orientation ???"
    )]
    ORIN,

    //Image ORIentation
    //Quaternions for the image orientation relative to the camera body
    #[strum(serialize = "IORI", to_string = "Image Orientation")]
    IORI,
    //GRAvity Vector
    //Vector for the direction for gravitiy
    #[strum(serialize = "GRAV", to_string = "Gravity Vector")]
    GRAV,
    //Wind Processing
    //marks whether wind processing is active
    #[strum(serialize = "WNDM", to_string = "Wind Processing")]
    WNDM,
    //Microphone is WET
    //marks whether some of the microphones are wet
    #[strum(serialize = "MWET", to_string = "Microphone is Wet")]
    MWET,

    //Audio Levels
    //RMS and peak audio levels in dBFS
    #[strum(serialize = "AALP", to_string = "Audio Levels (dBFS)")]
    AALP,

    //GoPro MAX (v2.0)
    //1-D depth map for the objects seen by the two lenses
    #[strum(serialize = "DISP", to_string = "Depth Map")]
    DISP,

    //HERO9

    //Main video frame SKiP
    #[strum(serialize = "MSKP", to_string = "Main video frame skip")]
    MSKP,
    //Low res video frame SKiP
    #[strum(serialize = "LSKP", to_string = "Low res video frame skip")]
    LSKP,

    //HERO11
    //lat, long, alt, 2D speed, 3D speed, days since 2000, secs since midnight (ms precision), DOP, fix (0, 2D or 3D)
    //improved precision over GPS5 for time and fix information
    //GPS5 deprecated
    #[strum(serialize = "GPS9", to_string = "GPS 9")]
    GPS9,

    ///  Its data consists of one or more 32-bit integers. The first integer contains the number of available HiLight tags. All subsequent integers resemble an ordered list of HiLight tags. Each HiLight tag is represented as a millisecond value.
    /// <https://superuser.com/questions/881661/how-where-does-a-gopro-camera-store-hilight-tags>
    #[strum(serialize = "HMMT", to_string = "HMMT UNDOCUMENTED HiLights ???")]
    HMMT,

    #[strum(serialize = "HLMT", to_string = "HLMT UNDOCUMENTED HiLights ???")]
    HLMT,

    #[strum(serialize = "MANL", to_string = "MANL UNDOCUMENTED Manual Label ???")]
    MANL,

    #[strum(serialize = "MTRX", to_string = "MTRX UNDOCUMENTED Tracks ???")]
    MTRX,

    #[strum(serialize = "AGST", to_string = "AGST UNDOCUMENTED ???")]
    AGST,

    #[strum(serialize = "KBAT", to_string = "KBAT UNDOCUMENTED Battery Status ???")]
    KBAT,

    #[strum(default)]
    Other(String),
}

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
