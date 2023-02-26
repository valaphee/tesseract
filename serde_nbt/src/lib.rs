use num_enum::{IntoPrimitive, TryFromPrimitive};

pub mod de;
pub mod error;
pub mod ser;

#[derive(Copy, Clone, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(i8)]
enum TagType {
    End,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    ByteArray,
    String,
    List,
    Compound,
}
