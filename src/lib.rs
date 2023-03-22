use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case,take};
use nom::combinator::{flat_map, map};
use nom::error::context;
use nom::multi::{count, length_count, many0};
use nom::number::complete::{be_u16, be_u8};
use nom::sequence::{preceded, terminated, tuple};
use nom::Offset;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};
use num_enum::TryFromPrimitive;

pub struct KeyValue {
    fourcc:FourCC,
    typ:Type,
    value:Value,
}

#[derive(Debug, PartialEq,Eq, EnumIter, EnumString, Display,TryFromPrimitive)]
#[repr(u8)]
pub enum Type {
    //| **b** | single byte signed integer | int8\_t | -128 to 127 |
    // #[strum(serialize="b")]
    I8=b'b',
    // | **B** | single byte unsigned integer | uint8\_t | 0 to 255 |
    // #[strum(serialize="B")]
    U8=b'B',
    // | **c** | single byte &#39;c&#39; style ASCII character string | char | Optionally NULL terminated - size/repeat sets the length |
    // #[strum(serialize="c")]
    Char=b'c',
    // | **d** | 64-bit double precision (IEEE 754) | double |   |
    // #[strum(serialize="d")]
    F64=b'd',
    // | **f** | 32-bit float (IEEE 754) | float |   |
    // #[strum(serialize="f")]
    F32=b'f',
    // | **F** | 32-bit four character key -- FourCC | char fourcc\[4\] |   |
    // #[strum(serialize="F")]
    FourCC=b'F',
    // | **G** | 128-bit ID (like UUID) | uint8\_t guid\[16\] |   |
    // #[strum(serialize="G")]
    U128=b'G',
    // | **j** | 64-bit signed unsigned number | int64\_t |   |
    // #[strum(serialize="j")]
    I64=b'j',
    // | **J** | 64-bit unsigned unsigned number | uint64\_t |   |
    // #[strum(serialize="J")]
    U64=b'J',
    // | **l** | 32-bit signed integer | int32\_t |   |
    // #[strum(serialize="l")]
    I32=b'l',
    // | **L** | 32-bit unsigned integer | uint32\_t |   |
    // #[strum(serialize="L")]
    U32=b'L',
    // | **q** | 32-bit Q Number Q15.16 | uint32\_t | 16-bit integer (A) with 16-bit fixed point (B) for A.B value (range -32768.0 to 32767.99998) |
    // #[strum(serialize="q")]
    Fixed32=b'q',
    // | **Q** | 64-bit Q Number Q31.32 | uint64\_t | 32-bit integer (A) with 32-bit fixed point (B) for A.B value. |
    // #[strum(serialize="Q")]
    Fixed64=b'Q',
    // | **s** | 16-bit signed integer | int16\_t | -32768 to 32768 |
    // #[strum(serialize="s")]
    I16=b's',
    // | **S** | 16-bit unsigned integer | uint16\_t | 0 to 65536 |
    // #[strum(serialize="S")]
    U16=b'S',
    // | **U** | UTC Date and Time string | char utcdate\[16\] | Date + UTC Time format yymmddhhmmss.sss - (years 20xx covered) |
    // #[strum(serialize="U")]
    Date=b'U',
    // | **?** | data structure is complex | TYPE | Structure is defined with a preceding TYPE |
    // #[strum(serialize="?")]
    Complex=b'?',
    // | **null** | Nested metadata | uint32\_t | The data within is GPMF structured KLV data |
    // #[strum(serialize="\0")]
    Nested=b'\0',
}



pub enum Value {
    //| **b** | single byte signed integer | int8\_t | -128 to 127 |
    I8(i8),
    // | **B** | single byte unsigned integer | uint8\_t | 0 to 255 |
    U8(u8),
    // | **c** | single byte &#39;c&#39; style ASCII character string | char | Optionally NULL terminated - size/repeat sets the length |
    Char(char),
    // | **d** | 64-bit double precision (IEEE 754) | double |   |
    F64(f64),
    // | **f** | 32-bit float (IEEE 754) | float |   |
    F32(f32),
    // | **F** | 32-bit four character key -- FourCC | char fourcc\[4\] |   |
    FourCC([char; 4]),
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
    Fixed32(u16, u16),
    // | **Q** | 64-bit Q Number Q31.32 | uint64\_t | 32-bit integer (A) with 32-bit fixed point (B) for A.B value. |
    Fixed64(u32, u32),
    // | **s** | 16-bit signed integer | int16\_t | -32768 to 32768 |
    I16(i16),
    // | **S** | 16-bit unsigned integer | uint16\_t | 0 to 65536 |
    U16(u16),
    // | **U** | UTC Date and Time string | char utcdate\[16\] | Date + UTC Time format yymmddhhmmss.sss - (years 20xx covered) |
    Date([char; 16]),
    // | **?** | data structure is complex | TYPE | Structure is defined with a preceding TYPE |
    Complex(Vec<u8>),
    // | **null** | Nested metadata | uint32\_t | The data within is GPMF structured KLV data |
    Nested(Vec<u8>),
}

#[derive(Debug, PartialEq, EnumString, EnumIter, Display)]
pub enum FourCC {
    // #[strum(serialize = "blue", serialize = "b")]
    ///unique device source for metadata
    /// Each connected device starts with DEVC. A GoPro camera or Karma drone would have their own DEVC for nested metadata to follow. |
    DEVC,
    /// device/track ID
    /// Auto generated unique-ID for managing a large number of connect devices, camera, karma and external BLE devices |
    DVID,
    /// device name
    /// Display name of the device like &quot;Karma 1.0&quot;, this is for communicating to the user the data recorded, so it should be informative. |
    DVNM,
    ///Nested signal stream of metadata/telemetry
    ///Metadata streams are each nested with STRM
    STRM,
    ///Stream name
    /// Display name for a stream like &quot;GPS RAW&quot;, this is for communicating to the user the data recorded, so it should be informative. |
    STNM,
    ///Comments for any stream
    /// Add more human readable information about the stream |
    RMRK,
    ///Scaling factor (divisor) | Sensor data often needs to be scaled to be presented with the correct units. SCAL is a divisor. |
    SCAL,
    ///Standard Units (like SI) | If the data can be formatted in GPMF&#39;s standard units, this is best. E.g. acceleration as &quot;m/s²&quot;.  SIUN allows for simple format conversions. |
    SIUN,

    /// Display units
    /// While SIUN is preferred, not everything communicates well via standard units. E.g. engine speed as &quot;RPM&quot; is more user friendly than &quot;rad/s&quot;. |
    UNIT,
    ///Typedefs for complex structures
    /// Not everything has a simple repeating type. For complex structure TYPE is used to describe the data packed within each sample. |
    TYPE,
    ///Total Samples delivered | Internal field that counts all the sample delivered since record start, and is automatically computed. |
    TSMP,
    ///Time Offset | Rare. An internal field that indicates the data is delayed by 'x' seconds. |
    TIMO,
    ///Empty payload count
    /// Internal field that reports the number of payloads that contain no new data. TSMP and EMPT simplify the extraction of clock. |
    EMPT,
}

fn key_values(input: &[u8]) -> nom::IResult<&[u8], Vec<Vec<u8>>> {
    context("key value", many0(key_value))(input)
}

fn key_value(input: &[u8]) -> nom::IResult<&[u8], Vec<u8>>
//nom::IResult<&[u8],(FourCC,u8,u8,u16)>
//Result<&str, Scheme>
{
    //method to map a new parser from the output of the first parser, then apply that parser over the rest of the input
    //flat_map()

    //Gets a number from the first parser, then applies the second parser that many times
    //length_count(
    context(
        "key value",
        flat_map(
            tuple((fourcc, be_u8, be_u8, be_u16)),
            move |(four, typ, len, repeat)| {

                //TODO match Ok() Err()
                let typ:Type=typ.try_into().unwrap();

                let bytes = len as usize * repeat as usize;
                let mod4=bytes % 4;
                let padding = if mod4==0 {0} else { 4-mod4 };
                println!("Type {} Bytes {} Padding {}",typ,bytes,padding);
                terminated(count(be_u8, bytes), take(padding))
                // count(be_u8, bytes)
            }
        ),
    )(input)
}

fn fourcc(input: &[u8]) -> nom::IResult<&[u8], FourCC>
//Result<&str, Scheme>
{
    context(
        "fourcc",
        map(
            alt((
                tag("DEVC"),
                tag("DVID"),
                tag("DVNM"),
                tag("STRM"),
                tag("STNM"),
                tag("RMRK"),
                tag("SCAL"),
                tag("SIUN"),
                tag("UNIT"),
                tag("TYPE"),
                tag("TSMP"),
                tag("TIMO"),
                tag("EMPT"),
            )),
            |s: &[u8]| FourCC::try_from(String::from_utf8_lossy(s).as_ref()).unwrap(),
        ),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{
        error::{ErrorKind, VerboseError, VerboseErrorKind},
        AsBytes, Err as NomErr,
    };
    use std::fs::File;
    use std::io::{BufReader, Read};
    use std::path::Path;

    #[test]
    fn test_type() {
        Type::iter().for_each(|t| {
           println!("{} {:?}",t,t);
        });
    }
    #[test]
    fn test_fourcc() {
        let (res, f) = fourcc("DEVC".as_bytes()).unwrap();

        assert_eq!(f, FourCC::DEVC);
    }

    #[test]
    fn test_file() -> anyhow::Result<()> {
        let dir = Path::new("/opt/git/gpmf-parser/samples");
        let path = dir.join("hero5.raw");

        // let input=BufReader::new(file);
        let text = std::fs::read(path).unwrap();

        let (res, f) = fourcc(&text).unwrap();
        assert_eq!(f, FourCC::DEVC);

        Ok(())
    }

    #[test]
    fn test_key_value() -> anyhow::Result<()> {
        let dir = Path::new("/opt/git/gpmf-parser/samples");
        let path = dir.join("hero5.raw");
        let file = File::open(&path)?;
        let len = file.metadata().unwrap().len();
        // let file=File::open(path)?;
        // let input=BufReader::new(file);
        let text = std::fs::read(path).unwrap();

        let (res, v) = key_values(&text).unwrap();

        v.iter().for_each(|e| {
            println!("{:?}", e);
        });
        // let (devc,typ,size,repeat)=&v[0];
        // assert_eq!(devc, &FourCC::DEVC);
        // assert_eq!(*typ,0);
        // assert_eq!(*size,1);
        // assert_eq!(*repeat,len as u16 - 8 );
        //
        // let (fourcc,typ,size,repeat)=&v[1];
        // assert_eq!(fourcc, &FourCC::DVID);
        // assert_eq!(*typ,b'L');
        // assert_eq!(*size,4);
        // assert_eq!(*repeat,1 );

        Ok(())
    }

    // #[test]
    // fn test_enums() {
    //     FourCC::iter().for_each(|x|
    //         println!(r#"tag("{}"),"#,x)
    //     );
    //
    // }
    // #[test]
    // fn test_scheme() {
    //     assert_eq!(scheme("https://yay"), Ok(("yay", Scheme::HTTPS)));
    //     assert_eq!(scheme("http://yay"), Ok(("yay", Scheme::HTTP)));
    //     // assert_eq!(
    //     //     scheme("bla://yay"),
    //     //     Err(NomErr::Error(VerboseError {
    //     //         errors: vec![
    //     //             ("bla://yay", VerboseErrorKind::Nom(ErrorKind::Tag)),
    //     //             ("bla://yay", VerboseErrorKind::Nom(ErrorKind::Alt)),
    //     //             ("bla://yay", VerboseErrorKind::Context("scheme")),
    //     //         ]
    //     //     }))
    //     // );
    // }
}
