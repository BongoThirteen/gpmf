const DATE_FORMAT: &str = "%y%m%d%H%M%S%.3f";

/// The value of the data,
#[derive(Debug, Clone)]
pub enum Value {
    ///| **b** | single byte signed integer | int8\_t | -128 to 127 |
    I8(i8),
    /// | **B** | single byte unsigned integer | uint8\_t | 0 to 255 |
    U8(u8),
    /// | **c** | single byte &#39;c&#39; style ASCII character string | char | Optionally NULL terminated - size/repeat sets the length |
    Char(char),
    /// | **c** | single byte &#39;c&#39; style ASCII character string | char | Optionally NULL terminated - size/repeat sets the length |
    String(String),
    /// | **d** | 64-bit double precision (IEEE 754) | double |   |
    F64(f64),
    /// | **f** | 32-bit float (IEEE 754) | float |   |
    F32(f32),
    /// | **F** | 32-bit four character key -- FourCC | char fourcc\[4\] |   |
    Tag(Tag),
    /// | **G** | 128-bit ID (like UUID) | uint8\_t guid\[16\] |   |
    U128(u128),
    /// | **j** | 64-bit signed unsigned number | int64\_t |   |
    I64(i64),
    /// | **J** | 64-bit unsigned unsigned number | uint64\_t |   |
    U64(u64),
    /// | **l** | 32-bit signed integer | int32\_t |   |
    I32(i32),
    /// | **L** | 32-bit unsigned integer | uint32\_t |   |
    U32(u32),
    /// | **q** | 32-bit Q Number Q15.16 | uint32\_t | 16-bit integer (A) with 16-bit fixed point (B) for A.B value (range -32768.0 to 32767.99998) |
    Fixed32(I16F16),
    /// | **Q** | 64-bit Q Number Q31.32 | uint64\_t | 32-bit integer (A) with 32-bit fixed point (B) for A.B value. |
    Fixed64(I32F32),
    /// | **s** | 16-bit signed integer | int16\_t | -32768 to 32768 |
    I16(i16),
    /// | **S** | 16-bit unsigned integer | uint16\_t | 0 to 65536 |
    U16(u16),
    /// | **U** | UTC Date and Time string | char utcdate\[16\] | Date + UTC Time format yymmddhhmmss.sss - (years 20xx covered) |
    Date(DateTime<Utc>),

    /// | **?** | data structure is complex | TYPE | Structure is defined with a preceding TYPE |
    Complex(Vec<Vec<Value>>),
    /// | **null** | Nested metadata | uint32\_t | The data within is GPMF structured KLV data |
    Nested(Vec<KeyValue>),
    /// Simple
    Simple(Vec<Vec<Value>>),
    /// Type
    Type(Vec<Type>),
    /// Strings
    Strings(Vec<String>),
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
            Value::Tag(_) => Type::FourCC,
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
            _ => unimplemented!(),
        }
    }
}
