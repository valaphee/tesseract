use std::{collections::HashMap, io::Write};

use glam::{IVec3, Vec3};
use serde_value::Value;
use uuid::Uuid;

use crate::{
    types::{ItemStack, Nbt, Pose, VarInt32},
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

impl Decode for EntityData {
    fn decode(input: &mut &[u8]) -> Result<Self> {
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
    Int(VarInt32),
    Float(f32),
    String(String),
    Component(String),
    OptionalComponent(Option<String>),
    ItemStack(Option<ItemStack>),
    Boolean(bool),
    Rotations(Vec3),
    BlockPos(IVec3),
    OptionalBlockPos(Option<IVec3>),
    Direction(VarInt32),
    OptionalUuid(Option<Uuid>),
    BlockState(VarInt32),
    CompoundTag(Nbt<Value>),
    Particle,
    VillagerData {
        type_: VarInt32,
        profession: VarInt32,
        level: VarInt32,
    },
    OptionalUnsignedInt(VarInt32),
    Pose(Pose),
    CatVariant(VarInt32),
    FrogVariant(VarInt32),
    OptionalGlobalPos(Option<(String, IVec3)>),
    PaintingVariant(VarInt32),
}
