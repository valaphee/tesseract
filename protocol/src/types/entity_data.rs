use std::{collections::HashMap, io::Write};

use glam::{IVec3, Vec3};
use uuid::Uuid;

use crate::{
    types::{ItemStack, Pose, VarInt},
    Decode, Encode, Result,
};

#[derive(Clone, Debug)]
pub struct EntityData(HashMap<u8, EntityDataValue>);

impl Encode for EntityData {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        for (&index, value) in &self.0 {
            index.encode(output)?;
            value.encode(output)?;
        }
        0xFFu8.encode(output)
    }
}

impl<'a> Decode<'a> for EntityData {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        let mut fields = HashMap::new();
        loop {
            let index = u8::decode(input)?;
            if index == 0xFF {
                break;
            }
            fields.insert(index, EntityDataValue::decode(input)?);
        }
        Ok(EntityData(fields))
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum EntityDataValue {
    Byte(u8),
    Int(VarInt),
    Float(f32),
    String(String),
    Component(String),
    OptionalComponent(Option<String>),
    ItemStack(Option<ItemStack>),
    Boolean(bool),
    Rotations(Vec3),
    BlockPos(IVec3),
    OptionalBlockPos(Option<IVec3>),
    Direction(VarInt),
    OptionalUuid(Option<Uuid>),
    BlockState(VarInt),
    CompoundTag,
    Particle,
    VillagerData {
        type_: VarInt,
        profession: VarInt,
        level: VarInt,
    },
    OptionalUnsignedInt(VarInt),
    Pose(Pose),
    CatVariant(VarInt),
    FrogVariant(VarInt),
    OptionalGlobalPos(Option<(String, IVec3)>),
    PaintingVariant(VarInt),
}
