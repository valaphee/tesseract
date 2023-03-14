use std::{collections::HashMap, io::Write};

use glam::{IVec3, Vec3};
use uuid::Uuid;

use crate::{
    types::{Direction, ItemStack, Nbt, Pose, VarI32, VarI64},
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
    Int(VarI32),
    Long(VarI64),
    Float(f32),
    String(String),
    Component(String),
    OptionalComponent(Option<String>),
    ItemStack(Option<ItemStack>),
    Boolean(bool),
    Rotations(Vec3),
    BlockPos(IVec3),
    OptionalBlockPos(Option<IVec3>),
    Direction(Direction),
    OptionalUuid(Option<Uuid>),
    BlockState(VarI32),
    CompoundTag(Nbt<serde_value::Value>),
    Particle,
    VillagerData {
        type_: VarI32,
        profession: VarI32,
        level: VarI32,
    },
    OptionalUnsignedInt(VarI32),
    Pose(Pose),
    CatVariant(VarI32),
    FrogVariant(VarI32),
    OptionalGlobalPos(Option<(String, IVec3)>),
    PaintingVariant(VarI32),
}
