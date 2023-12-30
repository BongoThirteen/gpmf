/// The data type of the sensor data
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumString, Display, TryFromPrimitive)]
#[repr(u8)]
pub enum Type {
    /// | **b** | single byte signed integer | int8\_t | -128 to 127 |
    I8 = b'b',

    /// | **B** | single byte unsigned integer | uint8\_t | 0 to 255 |
    U8 = b'B',

    /// | **c** | single byte &#39;c&#39; style ASCII character string | char | Optionally NULL terminated - size/repeat sets the length |
    Char = b'c',

    /// | **d** | 64-bit double precision (IEEE 754) | double |   |
    F64 = b'd',

    /// | **f** | 32-bit float (IEEE 754) | float |   |
    F32 = b'f',

    /// | **F** | 32-bit four character key -- FourCC | char fourcc\[4\] |   |
    FourCC = b'F',

    /// | **G** | 128-bit ID (like UUID) | uint8\_t guid\[16\] |   |
    U128 = b'G',

    /// | **j** | 64-bit signed unsigned number | int64\_t |   |
    I64 = b'j',

    /// | **J** | 64-bit unsigned unsigned number | uint64\_t |   |
    U64 = b'J',

    /// | **l** | 32-bit signed integer | int32\_t |   |
    I32 = b'l',

    /// | **L** | 32-bit unsigned integer | uint32\_t |   |
    U32 = b'L',

    /// | **q** | 32-bit Q Number Q15.16 | uint32\_t | 16-bit integer (A) with 16-bit fixed point (B) for A.B value (range -32768.0 to 32767.99998) |
    Fixed32 = b'q',

    /// | **Q** | 64-bit Q Number Q31.32 | uint64\_t | 32-bit integer (A) with 32-bit fixed point (B) for A.B value. |
    Fixed64 = b'Q',

    /// | **s** | 16-bit signed integer | int16\_t | -32768 to 32768 |
    I16 = b's',

    /// | **S** | 16-bit unsigned integer | uint16\_t | 0 to 65536 |
    U16 = b'S',

    /// | **U** | UTC Date and Time string | char utcdate\[16\] | Date + UTC Time format yymmddhhmmss.sss - (years 20xx covered) |
    Date = b'U',

    /// | **?** | data structure is complex | TYPE | Structure is defined with a preceding TYPE |
    Complex = b'?',

    /// | **null** | Nested metadata | uint32\_t | The data within is GPMF structured KLV data |
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
