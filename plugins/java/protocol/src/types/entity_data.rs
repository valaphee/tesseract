use std::{collections::HashMap, io::Write};

use glam::{IVec3, Quat, Vec3};
use uuid::Uuid;

use crate::{
    types::{Direction, ItemStack, Nbt, Pose, VarI32, VarI64},
    Decode, Encode, Result,
};

#[derive(Clone, Debug)]
pub struct EntityData(HashMap<u8, EntityDataValue>);

impl Encode for EntityData {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        for (&index, value) in &self.0 {
            index.encode(output)?;
            value.encode(output)?;
        }
        0xFFu8.encode(output)
    }
}

impl Decode<'_> for EntityData {
    fn decode(input: &mut &'_ [u8]) -> Result<Self> {
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

#[derive(Encode, Decode, Clone, Debug)]
pub enum EntityDataValue {
    Byte(u8),
    Int(#[using(VarI32)] i32),
    Long(#[using(VarI64)] i64),
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
    BlockState(#[using(VarI32)] i32),
    OptionalBlockState(#[using(VarI32)] i32),
    CompoundTag(Nbt<serde_value::Value>),
    Particle,
    VillagerData {
        #[using(VarI32)]
        type_: i32,
        #[using(VarI32)]
        profession: i32,
        #[using(VarI32)]
        level: i32,
    },
    OptionalUnsignedInt(#[using(VarI32)] i32),
    Pose(Pose),
    CatVariant(#[using(VarI32)] i32),
    FrogVariant(#[using(VarI32)] i32),
    OptionalGlobalPos(Option<(String, IVec3)>),
    PaintingVariant(#[using(VarI32)] i32),
    SnifferState,
    Vector3(Vec3),
    Quaternion(Quat),
}
