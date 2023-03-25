use crate::FourCC;
use crate::{Type, Value, DATE_FORMAT};
use byteorder::{BigEndian, ReadBytesExt};
use chrono::{TimeZone, Utc};
use fixed::types::{I16F16, I32F32};
use std::io;
use std::io::{BufRead, Cursor, Read};
use tracing::{debug, enabled, error, info, span, trace, warn, Level};
// use tracing_error::{InstrumentResult, TracedError};

impl Type {
    /// Implement reading Data Type using the byteorder crate
    fn read(&self, input: &mut Cursor<&[u8]>) -> anyhow::Result<Value> {
        let val = match self {
            Type::I8 => Value::I8(input.read_i8()?),
            Type::U8 => Value::U8(input.read_u8()?),
            Type::Char => Value::Char(input.read_u8()? as char),
            Type::F64 => Value::F64(input.read_f64::<BigEndian>()?),
            Type::F32 => Value::F32(input.read_f32::<BigEndian>()?),
            Type::FourCC => {
                let fourcc = read_fourcc(input)?;
                Value::FourCC(fourcc)
            }
            Type::U128 => Value::U128(input.read_u128::<BigEndian>()?),
            Type::I64 => Value::I64(input.read_i64::<BigEndian>()?),
            Type::U64 => Value::U64(input.read_u64::<BigEndian>()?),
            Type::I32 => Value::I32(input.read_i32::<BigEndian>()?),
            Type::U32 => Value::U32(input.read_u32::<BigEndian>()?),
            Type::Fixed32 => {
                let mut buf = [0u8; 4];
                input.read_exact(&mut buf)?;
                Value::Fixed32(I16F16::from_be_bytes(buf))
            }
            Type::Fixed64 => {
                let mut buf = [0u8; 8];
                input.read_exact(&mut buf)?;
                Value::Fixed64(I32F32::from_be_bytes(buf))
            }
            Type::I16 => Value::I16(input.read_i16::<BigEndian>()?),
            Type::U16 => Value::U16(input.read_u16::<BigEndian>()?),
            Type::Date => {
                let mut buf = [0u8; 16];
                input.read_exact(&mut buf)?;
                let date_str = String::from_utf8_lossy(&buf);
                let utc = Utc
                    .datetime_from_str(date_str.as_ref(), DATE_FORMAT)
                    .unwrap();
                Value::Date(utc)
            }
            _ => {
                unimplemented!("For Type {} please file a bug report", self)
            }
        };
        Ok(val)
    }
}

/// Read the FourCC field using the byteorder crate
fn read_fourcc(input: &mut Cursor<&[u8]>) -> anyhow::Result<FourCC> {
    let mut fourcc = [0u8; 4];
    input.read_exact(fourcc.as_mut_slice())?;
    let fourcc_string: String = fourcc.iter().map(|c| *c as char).collect();
    let fourcc = FourCC::try_from(fourcc_string.as_str())?;
    debug!("FourCC {} ({:?})", fourcc_string, fourcc);
    if let FourCC::Other(other) = &fourcc {
        warn!("Unsupported FourCC key found {}", other);
    }
    Ok(fourcc)
}

/// Parse the GPMF stream using the bytorder crate
/// This function will be called recursively to handle nested data structures
pub fn parse_gpmf(input: &[u8]) -> anyhow::Result<()> {
    //the complex data structure types
    let mut type_def: Option<Vec<Type>> = None;

    //the cursor to handle reading from the slice
    let mut input = Cursor::new(input);

    while input.has_data_left()? {
        let fourcc = read_fourcc(&mut input)?;
        let type_u8 = input.read_u8()?;
        debug!("Type_u8 {}", type_u8);

        let typ = Type::try_from(type_u8)?;
        debug!("Type {}\t{}\t{}", type_u8, type_u8 as char, typ);

        let size = input.read_u8()?;
        let repeat = input.read_u16::<BigEndian>()?;
        debug!("Type Size {} bytes Repeat {}", size, repeat);

        let num_bytes = size as usize * repeat as usize;

        let type_size = if typ == Type::Complex {
            type_def.as_ref().unwrap().iter().map(|t| t.size()).sum()
        } else {
            typ.size()
        };

        let num_elements = if type_size != 0 {
            size as usize / type_size
        } else {
            error!("Type size is Zero - Trying to continue assuming zero elements");
            0
        };
        debug!(
            "Type Calc Size {} bytes Num Elements {}",
            type_size, num_elements
        );

        let mod4 = num_bytes % 4;
        let padding_bytes = if mod4 == 0 { 0 } else { 4 - mod4 };
        trace!(
            "Num Bytes {} Mod4 {} Padding Bytes {}",
            num_bytes,
            mod4,
            padding_bytes
        );

        match typ {
            Type::Char => {
                if num_elements == 1 {
                    // special case for repeat of 1 element
                    let mut vec = Vec::new();
                    let _take = input.by_ref().take(repeat as u64).read_to_end(&mut vec)?;

                    if fourcc != FourCC::TYPE {
                        let v: String = vec
                            .into_iter()
                            .take_while(|b| *b != 0)
                            .map(|b| b as char)
                            .collect();
                        debug!("char/string {:?}", v);
                    } else {
                        let v: Vec<_> = vec
                            .into_iter()
                            .take_while(|b| *b != 0)
                            .map(|type_u8| Type::try_from(type_u8).unwrap())
                            .collect();
                        info!("TYPE def types {:?}", v);
                        type_def = Some(v);
                    }
                } else {
                    for i in 0..repeat {
                        let mut vec = Vec::new();
                        let _take = input
                            .by_ref()
                            .take(num_elements as u64)
                            .read_to_end(&mut vec)?;

                        if enabled!(Level::TRACE) {
                            vec.iter()
                                .enumerate()
                                .for_each(|(i, c)| trace!("{}: {} '{}'", i, c, *c as char));
                        }

                        let v: String = vec
                            .into_iter()
                            .take_while(|b| *b != 0)
                            .map(|b| b as char)
                            .collect();
                        debug!("{}: char/string {:?}", i, v);
                    }
                }
            }
            Type::Complex => {
                let type_def = type_def
                    .as_ref()
                    .ok_or(anyhow::Error::msg("TYPE must be set"))?;
                //TODO assert_eq!(num_elements,type_def.len());
                let mut vec = Vec::new();
                for i in 0..repeat {
                    for t in type_def {
                        let v = t.read(&mut input)?;
                        vec.push(v);
                    }
                    info!("{}: Complex Type {:?}", i, vec);
                }
            }
            Type::Nested => {
                let offset = input.position();
                let len = num_bytes;
                let _span_ =
                    span!(Level::DEBUG, "Type::Nested", offset = offset, len = len).entered();

                let next = &input.remaining_slice()[..num_bytes];

                parse_gpmf(next)?;
            }

            //Handle other types
            t => {
                for i in 0..repeat {
                    let mut vec = Vec::new();
                    for _j in 0..num_elements {
                        let v = t.read(&mut input)?;
                        vec.push(v);
                    }
                    debug!("{}: {:?}", i, vec);
                }
            }
        }
        if padding_bytes > 0 {
            debug!("Skipping {} bytes", padding_bytes);
            io::copy(
                &mut input.by_ref().take(padding_bytes as u64),
                &mut io::sink(),
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::setup;
    use std::path::Path;

    fn read_file(path: &str) -> anyhow::Result<()> {
        setup();
        let dir = Path::new("samples");
        let path = dir.join(path);

        let text = std::fs::read(path)?;

        parse_gpmf(text.as_slice())?;

        Ok(())
    }

    #[test]
    fn test_byteorder_hero5() {
        let _res = read_file("hero5.raw").unwrap();
    }

    #[test]
    fn test_byteorder_hero6() {
        let _res = read_file("hero6.raw").unwrap();
    }

    #[test]
    fn test_byteorder_hero6ble() {
        let _res = read_file("hero6+ble.raw").unwrap();
    }

    #[test]
    fn test_byteorder_fusion() {
        let _res = read_file("Fusion.raw").unwrap();
    }

    #[test]
    fn test_byteorder_karma() {
        let _res = read_file("karma.raw").unwrap();
    }
}
