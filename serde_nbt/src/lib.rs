use num_enum::{IntoPrimitive, TryFromPrimitive};

pub mod de;
pub mod error;
pub mod ser;

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(i8)]
enum TagType {
    #[default]
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
    IntArray,
    LongArray,
}
