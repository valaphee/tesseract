use std::{io::Write, str::from_utf8};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use glam::{DVec3, IVec3, Quat, Vec3};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_value::Value;
use uuid::Uuid;

pub use bit_storage::BitStorage;
pub use chat::{Component, ComponentContents};
pub use entity_data::{EntityData, EntityDataValue};
pub use mojang_session_api::models::{User, UserProperty};
pub use paletted_container::PalettedContainer;

use crate::{Decode, Encode, Error, Result};

mod bit_storage;
mod chat;
mod entity_data;
mod paletted_container;

//=================================================================================== PRIMITIVE ====

impl Encode for bool {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        if *self { 1u8 } else { 0u8 }.encode(output)
    }
}

impl<'a> Decode<'a> for bool {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(match u8::decode(input)? {
            0 => false,
            1 => true,
            variant => return Err(Error::UnknownVariant(variant as i32)),
        })
    }
}

impl Encode for u8 {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        output.write_u8(*self)?;
        Ok(())
    }
}

impl<'a> Decode<'a> for u8 {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(input.read_u8()?)
    }
}

impl Encode for i8 {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        output.write_i8(*self)?;
        Ok(())
    }
}

impl<'a> Decode<'a> for i8 {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(input.read_i8()?)
    }
}

impl Encode for u16 {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        output.write_u16::<BigEndian>(*self)?;
        Ok(())
    }
}

impl<'a> Decode<'a> for u16 {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(input.read_u16::<BigEndian>()?)
    }
}

impl Encode for i16 {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        output.write_i16::<BigEndian>(*self)?;
        Ok(())
    }
}

impl<'a> Decode<'a> for i16 {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(input.read_i16::<BigEndian>()?)
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct VarI21(pub i32);

impl VarI21 {
    pub fn len(&self) -> usize {
        match self.0 {
            0 => 1,
            n => (31 - n.leading_zeros() as usize) / 7 + 1,
        }
    }
}

impl Encode for VarI21 {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        let mut value = self.0 as u32;
        loop {
            if value & !0b01111111 == 0 {
                output.write_u8(value as u8)?;
                return Ok(());
            }
            output.write_u8(value as u8 & 0b01111111 | 0b10000000)?;
            value >>= 7;
        }
    }
}

impl<'a> Decode<'a> for VarI21 {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        let mut value = 0;
        let mut shift = 0;
        while shift <= 21 {
            let head = input.read_u8()?;
            value |= (head as i32 & 0b01111111) << shift;
            if head & 0b10000000 == 0 {
                return Ok(VarI21(value));
            }
            shift += 7;
        }
        Err(Error::VarIntTooWide(21))
    }
}

impl Encode for i32 {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        output.write_i32::<BigEndian>(*self)?;
        Ok(())
    }
}

impl<'a> Decode<'a> for i32 {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(input.read_i32::<BigEndian>()?)
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct VarI32(pub i32);

impl VarI32 {
    pub fn len(&self) -> usize {
        match self.0 {
            0 => 1,
            n => (31 - n.leading_zeros() as usize) / 7 + 1,
        }
    }
}

impl Encode for VarI32 {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        let mut value = self.0 as u32;
        loop {
            if value & !0b01111111 == 0 {
                output.write_u8(value as u8)?;
                return Ok(());
            }
            output.write_u8(value as u8 & 0b01111111 | 0b10000000)?;
            value >>= 7;
        }
    }
}

impl<'a> Decode<'a> for VarI32 {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        let mut value = 0;
        let mut shift = 0;
        while shift <= 35 {
            let head = input.read_u8()?;
            value |= (head as i32 & 0b01111111) << shift;
            if head & 0b10000000 == 0 {
                return Ok(VarI32(value));
            }
            shift += 7;
        }
        Err(Error::VarIntTooWide(35))
    }
}

impl Encode for i64 {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        output.write_i64::<BigEndian>(*self)?;
        Ok(())
    }
}

impl<'a> Decode<'a> for i64 {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(input.read_i64::<BigEndian>()?)
    }
}

impl Encode for u64 {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        output.write_u64::<BigEndian>(*self)?;
        Ok(())
    }
}

impl<'a> Decode<'a> for u64 {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(input.read_u64::<BigEndian>()?)
    }
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct VarI64(pub i64);

impl VarI64 {
    pub fn len(&self) -> usize {
        match self.0 {
            0 => 1,
            n => (63 - n.leading_zeros() as usize) / 7 + 1,
        }
    }
}

impl Encode for VarI64 {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        let mut value = self.0 as u64;
        loop {
            if value & !0b01111111 == 0 {
                output.write_u8(value as u8)?;
                return Ok(());
            }
            output.write_u8(value as u8 & 0b01111111 | 0b10000000)?;
            value >>= 7;
        }
    }
}

impl<'a> Decode<'a> for VarI64 {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        let mut value = 0;
        let mut shift = 0;
        while shift <= 70 {
            let head = input.read_u8()?;
            value |= (head as i64 & 0b01111111) << shift;
            if head & 0b10000000 == 0 {
                return Ok(VarI64(value));
            }
            shift += 7;
        }
        Err(Error::VarIntTooWide(70))
    }
}

impl Encode for f32 {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        output.write_f32::<BigEndian>(*self)?;
        Ok(())
    }
}

impl<'a> Decode<'a> for f32 {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(input.read_f32::<BigEndian>()?)
    }
}

impl Encode for f64 {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        output.write_f64::<BigEndian>(*self)?;
        Ok(())
    }
}

impl<'a> Decode<'a> for f64 {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(input.read_f64::<BigEndian>()?)
    }
}

//======================================================================================= TUPLE ====

macro_rules! tuple {
    ($($ty:ident)*) => {
        #[allow(non_snake_case)]
        impl<$($ty: Encode,)*> Encode for ($($ty,)*) {
            fn encode(&self, output: &mut impl Write) -> Result<()> {
                let ($($ty,)*) = self;
                $(
                    $ty.encode(output)?;
                )*
                Ok(())
            }
        }

        impl<'a, $($ty: Decode<'a>,)*> Decode<'a> for ($($ty,)*) {
            fn decode(input: &mut &'a [u8]) -> Result<Self> {
                Ok(($($ty::decode(input)?,)*))
            }
        }
    }
}

tuple!();
tuple!(A);
tuple!(A B);
tuple!(A B C);
tuple!(A B C D);
tuple!(A B C D E);
tuple!(A B C D E F);
tuple!(A B C D E F G);
tuple!(A B C D E F G H);
tuple!(A B C D E F G H I);
tuple!(A B C D E F G H I J);
tuple!(A B C D E F G H I J K);
tuple!(A B C D E F G H I J K L);

//=============================================================================== ARRAY / SLICE ====

impl<T: Encode, const N: usize> Encode for [T; N] {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        for value in self {
            value.encode(output)?;
        }
        Ok(())
    }
}

impl<'a, T: Decode<'a>, const N: usize> Decode<'a> for [T; N] {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        std::array::try_from_fn(|_| Decode::decode(input))
    }
}

impl Encode for &[u8] {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        VarI32(self.len() as i32).encode(output)?;
        output.write_all(self)?;
        Ok(())
    }
}

impl<'a> Decode<'a> for &'a [u8] {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        let length = VarI32::decode(input)?.0 as usize;
        let (bytes, input_) = input.split_at(length);
        *input = input_;
        Ok(bytes)
    }
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct TrailingBytes<const N: usize>(pub Vec<u8>);

impl<const N: usize> Encode for TrailingBytes<N> {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        assert!(self.0.len() <= N);
        output.write_all(&self.0)?;
        Ok(())
    }
}

impl<'a, const N: usize> Decode<'a> for TrailingBytes<N> {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        assert!(input.len() <= N);
        let value = input.to_vec();
        *input = &input[input.len()..];
        Ok(TrailingBytes(value))
    }
}

//========================================================================================= STD ====

impl<T: Encode> Encode for Option<T> {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        match self {
            None => false.encode(output),
            Some(value) => {
                true.encode(output)?;
                value.encode(output)
            }
        }
    }
}

impl<'a, T: Decode<'a>> Decode<'a> for Option<T> {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(match bool::decode(input)? {
            true => Some(Decode::decode(input)?),
            false => None,
        })
    }
}

impl<T: Encode> Encode for Vec<T> {
    default fn encode(&self, output: &mut impl Write) -> Result<()> {
        VarI32(self.len() as i32).encode(output)?;
        for item in self {
            item.encode(output)?;
        }
        Ok(())
    }
}

impl<'a, T: Decode<'a>> Decode<'a> for Vec<T> {
    default fn decode(input: &mut &'a [u8]) -> Result<Self> {
        let length = VarI32::decode(input)?.0 as usize;
        let mut value = Vec::with_capacity(length);
        for _ in 0..length {
            value.push(Decode::decode(input)?);
        }
        Ok(value)
    }
}

trait VecEncode<T> {
    fn encode_special(value: &[T], output: &mut impl Write) -> Result<()>;
}

trait VecDecode<T> {
    fn decode_special(input: &mut &[u8], length: usize) -> Result<Vec<T>>;
}

impl<T : Encode + VecEncode<T>> Encode for Vec<T> {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        VarI32(self.len() as i32).encode(output)?;
        T::encode_special(self, output)?;
        Ok(())
    }
}

impl<'a, T: Decode<'a> + VecDecode<T>> Decode<'a> for Vec<T> {
    default fn decode(input: &mut &'a [u8]) -> Result<Self> {
        let length = VarI32::decode(input)?.0 as usize;
        T::decode_special(input, length)
    }
}

impl VecEncode<u8> for u8 {
    fn encode_special(value: &[u8], output: &mut impl Write) -> Result<()> {
        output.write_all(value)?;
        Ok(())
    }
}

impl VecDecode<u8> for u8 {
    fn decode_special(input: &mut &[u8], length: usize) -> Result<Vec<u8>> {
        let (value, input_) = input.split_at(length);
        *input = input_;
        Ok(value.to_vec())
    }
}

impl Encode for String {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        let bytes = self.as_bytes();
        VarI32(bytes.len() as i32).encode(output)?;
        output.write_all(bytes)?;
        Ok(())
    }
}

impl<'a> Decode<'a> for String {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        let length = VarI32::decode(input)?.0 as usize;
        let (bytes, input_) = input.split_at(length);
        *input = input_;
        Ok(from_utf8(bytes)?.to_string())
    }
}

//======================================================================================= OTHER ====

impl Encode for Uuid {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        output.write_u128::<BigEndian>(self.as_u128())?;
        Ok(())
    }
}

impl<'a> Decode<'a> for Uuid {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(Uuid::from_u128(input.read_u128::<BigEndian>()?))
    }
}

// interpreted as BlockPos
impl Encode for IVec3 {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        match (self.x, self.y, self.z) {
            (-0x2000000..=0x1FFFFFF, -0x800..=0x7FF, -0x2000000..=0x1FFFFFF) => {
                ((self.x as u64) << 38 | (self.z as u64) << 12 | self.y as u64 & 0xFFF)
                    .encode(output)
            }
            _ => unimplemented!(),
        }
    }
}

impl<'a> Decode<'a> for IVec3 {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        let value = i64::decode(input)?;
        Ok(Self {
            x: (value >> 38) as i32,
            y: (value << 52 >> 52) as i32,
            z: (value << 26 >> 38) as i32,
        })
    }
}

impl Encode for Vec3 {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        self.x.encode(output)?;
        self.y.encode(output)?;
        self.z.encode(output)
    }
}

impl<'a> Decode<'a> for Vec3 {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(Vec3::new(
            Decode::decode(input)?,
            Decode::decode(input)?,
            Decode::decode(input)?,
        ))
    }
}

impl Encode for DVec3 {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        self.x.encode(output)?;
        self.y.encode(output)?;
        self.z.encode(output)
    }
}

impl<'a> Decode<'a> for DVec3 {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(DVec3::new(
            Decode::decode(input)?,
            Decode::decode(input)?,
            Decode::decode(input)?,
        ))
    }
}

impl Encode for Quat {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        self.x.encode(output)?;
        self.y.encode(output)?;
        self.z.encode(output)?;
        self.w.encode(output)
    }
}

impl<'a> Decode<'a> for Quat {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(Quat::from_xyzw(
            Decode::decode(input)?,
            Decode::decode(input)?,
            Decode::decode(input)?,
            Decode::decode(input)?,
        ))
    }
}

#[derive(Clone, Debug)]
pub struct Angle(pub f32);

impl Encode for Angle {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        ((self.0.rem_euclid(360.0) / 360.0 * u8::MAX as f32).round() as u8).encode(output)
    }
}

impl<'a> Decode<'a> for Angle {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(Self(u8::decode(input)? as f32 / u8::MAX as f32 * 360.0))
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct Advancement {
    pub parent_id: Option<String>,
    pub display: Option<AdvancementDisplayInfo>,
    pub criteria: Vec<String>,
    pub requirements: Vec<Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct AdvancementDisplayInfo {
    pub title: Json<Component>,
    pub description: Json<Component>,
    pub icon: Option<ItemStack>,
    pub frame: AdvancementFrameType,
    pub background: Option<String>,
    pub show_toast: bool,
    pub hidden: bool,
    pub x: f32,
    pub y: f32,
}

impl Encode for AdvancementDisplayInfo {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        self.title.encode(output)?;
        self.description.encode(output)?;
        self.icon.encode(output)?;
        self.frame.encode(output)?;
        let mut flags = 0;
        if self.background.is_some() {
            flags |= 1 << 0;
        }
        if self.show_toast {
            flags |= 1 << 1;
        }
        if self.hidden {
            flags |= 1 << 2;
        }
        flags.encode(output)?;
        if let Some(background) = &self.background {
            background.encode(output)?;
        }
        self.x.encode(output)?;
        self.y.encode(output)?;
        Ok(())
    }
}

impl<'a> Decode<'a> for AdvancementDisplayInfo {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        let title = Decode::decode(input)?;
        let description = Decode::decode(input)?;
        let icon = Decode::decode(input)?;
        let frame = Decode::decode(input)?;
        let flags = i32::decode(input)?;
        let background = if flags & (1 << 0) != 0 {
            Some(Decode::decode(input)?)
        } else {
            None
        };
        let x = Decode::decode(input)?;
        let y = Decode::decode(input)?;
        Ok(Self {
            title,
            description,
            icon,
            frame,
            background,
            show_toast: flags & (1 << 1) != 0,
            hidden: flags & (1 << 2) != 0,
            x,
            y,
        })
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum AdvancementFrameType {
    Task,
    Challenge,
    Goal,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum Anchor {
    Feet,
    Eyes,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Biome {
    pub has_precipitation: bool,
    pub temperature: f32,
    pub temperature_modifier: Option<String>,
    pub downfall: f32,
    pub effects: BiomeEffects,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BiomeTemperatureModifier {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "frozen")]
    Frozen,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiomeEffects {
    pub fog_color: u32,
    pub water_color: u32,
    pub water_fog_color: u32,
    pub sky_color: u32,
    pub foliage_color: Option<u32>,
    pub grass_color: Option<u32>,
    pub grass_color_modifier: Option<String>,
    pub ambient_sound: Option<String>,
    pub mood_sound: Option<BiomeEffectsMoodSound>,
    pub additions_sound: Option<BiomeEffectsAdditionsSound>,
    pub music: Option<BiomeEffectsMusic>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiomeEffectsMusic {
    pub sound: String,
    pub min_delay: u32,
    pub max_delay: u32,
    pub replace_current_music: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiomeEffectsAdditionsSound {
    pub sound: String,
    pub tick_chance: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiomeEffectsMoodSound {
    pub sound: String,
    pub tick_delay: u32,
    pub block_search_extent: u32,
    pub offset: f64,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum BossEventColor {
    Pink,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum BossEventOverlay {
    Progress,
    Notched6,
    Notched10,
    Notched12,
    Notched20,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct ChatSession {
    pub session_id: Uuid,
    pub expires_at: i64,
    pub public_key: Vec<u8>,
    pub key_signature: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatType {
    pub chat: ChatTypeDecoration,
    pub narration: ChatTypeDecoration,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct ChatTypeBound {
    pub chat_type: VarI32,
    pub name: Json<Component>,
    pub target_name: Option<Json<Component>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatTypeDecoration {
    pub translation_key: String,
    pub parameters: Vec<String>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum ChatVisibility {
    Full,
    System,
    Hidden,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum ClickType {
    Pickup,
    QuickMove,
    Swap,
    Clone,
    Throw,
    QuickCraft,
    PickupAll,
}

#[derive(Clone, Debug)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl Encode for Difficulty {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        match self {
            Difficulty::Peaceful => 0u8,
            Difficulty::Easy => 1u8,
            Difficulty::Normal => 2u8,
            Difficulty::Hard => 3u8,
        }
        .encode(output)
    }
}

impl<'a> Decode<'a> for Difficulty {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(match u8::decode(input)? % 4 {
            0 => Difficulty::Peaceful,
            1 => Difficulty::Easy,
            2 => Difficulty::Normal,
            3 => Difficulty::Hard,
            _ => unreachable!(),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DimensionType {
    pub fixed_time: Option<u64>,
    pub has_skylight: bool,
    pub has_ceiling: bool,
    pub ultrawarm: bool,
    pub natural: bool,
    pub coordinate_scale: f64,
    pub bed_works: bool,
    pub respawn_anchor_works: bool,
    pub min_y: i32,
    pub height: u32,
    pub logical_height: u32,
    pub infiniburn: String,
    pub effects: String,
    pub ambient_light: f32,
    pub piglin_safe: bool,
    pub has_raids: bool,
    pub monster_spawn_light_level: MonsterSpawnLightLevel,
    pub monster_spawn_block_light_limit: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MonsterSpawnLightLevel {
    Scalar(i32),
    Custom(MonsterSpawnLightLevelCustom),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonsterSpawnLightLevelCustom {
    #[serde(rename = "type")]
    type_: String,
    value: MonsterSpawnLightLevelCustomValue,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonsterSpawnLightLevelCustomValue {
    min_inclusive: i32,
    max_inclusive: i32,
}

#[derive(Clone, Debug)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    East,
    West,
}

impl Encode for Direction {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        match self {
            Direction::Down => 0u8,
            Direction::Up => 1u8,
            Direction::North => 2u8,
            Direction::South => 3u8,
            Direction::East => 4u8,
            Direction::West => 5u8,
        }
        .encode(output)
    }
}

impl<'a> Decode<'a> for Direction {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(match u8::decode(input)? % 6 {
            0 => Direction::Down,
            1 => Direction::Up,
            2 => Direction::North,
            3 => Direction::South,
            4 => Direction::East,
            5 => Direction::West,
            _ => unreachable!(),
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum EquipmentSlot {
    MainHand,
    OffHand,
    Feet,
    Legs,
    Chest,
    Head,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum GameType {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum Hand {
    MainHand,
    OffHand,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum Intention {
    Game,
    Status,
    Login,
}

#[derive(Clone, Debug)]
pub struct ItemStack {
    pub item: VarI32,
    pub count: i8,
    pub tag: Nbt<Value>,
}

impl Encode for ItemStack {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        self.item.encode(output)?;
        self.count.encode(output)?;
        self.tag.encode(output)
    }
}

impl<'a> Decode<'a> for ItemStack {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(ItemStack {
            item: Decode::decode(input)?,
            count: Decode::decode(input)?,
            tag: Decode::decode(input)?,
        })
    }
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Json<T>(pub T);

impl<T: Serialize> Encode for Json<T> {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        serde_json::to_string(&self.0)?.encode(output)
    }
}

impl<'a, T: DeserializeOwned> Decode<'a> for Json<T> {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        let value = String::decode(input)?;
        Ok(Json(serde_json::from_str(value.as_str())?))
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct LastSeenMessages {
    pub offset: VarI32,
    pub acknowledged: [u8; 3],
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum MainHand {
    Left,
    Right,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct MapDecoration {
    pub type_: MapDecorationType,
    pub x: i8,
    pub y: i8,
    pub rot: i8,
    pub name: Option<Json<Component>>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum MapDecorationType {
    Player,
    Frame,
    RedMarker,
    BlueMarker,
    TargetX,
    TargetPoint,
    PlayerOffMap,
    PlayerOffLimits,
    Mansion,
    Monument,
    BannerWhite,
    BannerOrange,
    BannerMagenta,
    BannerLightBlue,
    BannerYellow,
    BannerLime,
    BannerPink,
    BannerGray,
    BannerLightGray,
    BannerCyan,
    BannerPurple,
    BannerBlue,
    BannerBrown,
    BannerGreen,
    BannerRed,
    BannerBlack,
    RedX,
}

#[derive(Clone, Debug)]
pub struct MapPatch {
    pub width: u8,
    pub height: u8,
    pub start_x: u8,
    pub start_y: u8,
    pub map_colors: Vec<u8>,
}

impl Encode for Option<MapPatch> {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        match self {
            None => 0u8.encode(output),
            Some(value) => {
                value.width.encode(output)?;
                value.height.encode(output)?;
                value.start_x.encode(output)?;
                value.start_y.encode(output)?;
                value.map_colors.encode(output)
            }
        }
    }
}

impl<'a> Decode<'a> for Option<MapPatch> {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        let width = Decode::decode(input)?;
        Ok(if width != 0 {
            Some(MapPatch {
                width,
                height: Decode::decode(input)?,
                start_x: Decode::decode(input)?,
                start_y: Decode::decode(input)?,
                map_colors: Decode::decode(input)?,
            })
        } else {
            None
        })
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct MerchantOffer {
    pub base_cost_a: Option<ItemStack>,
    pub result: Option<ItemStack>,
    pub cost_b: Option<ItemStack>,
    pub out_of_stock: bool,
    pub uses: i32,
    pub max_uses: i32,
    pub xp: i32,
    pub special_price_diff: i32,
    pub price_multiplier: f32,
    pub demand: i32,
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Nbt<T>(pub T);

impl<T: Serialize> Encode for Nbt<T> {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        output.write_all(&tesseract_nbt::ser::to_vec(&self.0)?)?;
        Ok(())
    }
}

impl<'a, T: DeserializeOwned> Decode<'a> for Nbt<T> {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(Nbt(tesseract_nbt::de::from_slice(input)?))
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum Pose {
    Standing,
    FallFlying,
    Sleeping,
    Swimming,
    SpinAttack,
    Crouching,
    LongJumping,
    Dying,
    Croaking,
    UsingTongue,
    Roaring,
    Sniffing,
    Emerging,
    Digging,
}

#[derive(Clone, Debug)]
pub enum Recipe {
    Shaped {
        id: String,
        width: VarI32,
        height: VarI32,
        group: String,
        category: VarI32,
        ingredients: Vec<Vec<Option<ItemStack>>>,
        result: Option<ItemStack>,
        show_notification: bool,
    },
    Shapeless {
        id: String,
        group: String,
        category: VarI32,
        ingredients: Vec<Vec<Option<ItemStack>>>,
        result: Option<ItemStack>,
    },
    ArmorDye(SimpleRecipe),
    BookCloning(SimpleRecipe),
    MapCloning(SimpleRecipe),
    MapExtending(SimpleRecipe),
    FireworkRocket(SimpleRecipe),
    FireworkStar(SimpleRecipe),
    FireworkStarFade(SimpleRecipe),
    TippedArrow(SimpleRecipe),
    BannerDuplicate(SimpleRecipe),
    ShieldDecoration(SimpleRecipe),
    ShulkerBoxColoring(SimpleRecipe),
    SuspiciousStew(SimpleRecipe),
    RepairItem(SimpleRecipe),
    Smelting(SimpleCooking),
    Blasting(SimpleCooking),
    Smoking(SimpleCooking),
    CampfireCooking(SimpleCooking),
    Stonecutting {
        id: String,
        group: String,
        ingredient: Vec<Option<ItemStack>>,
        result: Option<ItemStack>,
    },
    Smithing {
        id: String,
        base: Vec<Option<ItemStack>>,
        addition: Vec<Option<ItemStack>>,
        result: Option<ItemStack>,
    },
}

impl Recipe {
    fn type_(&self) -> &'static str {
        match self {
            Recipe::Shaped { .. } => "minecraft:crafting_shaped",
            Recipe::Shapeless { .. } => "minecraft:crafting_shapeless",
            Recipe::ArmorDye(_) => "minecraft:crafting_special_armordye",
            Recipe::BookCloning(_) => "minecraft:crafting_special_bookcloning",
            Recipe::MapCloning(_) => "minecraft:crafting_special_mapcloning",
            Recipe::MapExtending(_) => "minecraft:crafting_special_mapextending",
            Recipe::FireworkRocket(_) => "minecraft:crafting_special_firework_rocket",
            Recipe::FireworkStar(_) => "minecraft:crafting_special_firework_star",
            Recipe::FireworkStarFade(_) => "minecraft:crafting_special_firework_star_fade",
            Recipe::TippedArrow(_) => "minecraft:crafting_special_tippedarrow",
            Recipe::BannerDuplicate(_) => "minecraft:crafting_special_bannerduplicate",
            Recipe::ShieldDecoration(_) => "minecraft:crafting_special_shielddecoration",
            Recipe::ShulkerBoxColoring(_) => "minecraft:crafting_special_shulkerboxcoloring",
            Recipe::SuspiciousStew(_) => "minecraft:crafting_special_suspiciousstew",
            Recipe::RepairItem(_) => "minecraft:crafting_special_repairitem",
            Recipe::Smelting(_) => "minecraft:smelting",
            Recipe::Blasting(_) => "minecraft:blasting",
            Recipe::Smoking(_) => "minecraft:smoking",
            Recipe::CampfireCooking(_) => "minecraft:campfire_cooking",
            Recipe::Stonecutting { .. } => "minecraft:stonecutting",
            Recipe::Smithing { .. } => "minecraft:smithing",
        }
    }
}

impl Encode for Recipe {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        self.type_().to_string().encode(output)?;
        match self {
            Recipe::Shaped {
                id,
                width,
                height,
                group,
                category,
                ingredients,
                result,
                show_notification,
            } => {
                id.encode(output)?;
                width.encode(output)?;
                height.encode(output)?;
                group.encode(output)?;
                category.encode(output)?;
                for ingredient in ingredients {
                    ingredient.encode(output)?;
                }
                result.encode(output)?;
                show_notification.encode(output)
            }
            Recipe::Shapeless {
                id,
                group,
                category,
                ingredients,
                result,
            } => {
                id.encode(output)?;
                group.encode(output)?;
                category.encode(output)?;
                ingredients.encode(output)?;
                result.encode(output)
            }
            Recipe::ArmorDye(recipe)
            | Recipe::BookCloning(recipe)
            | Recipe::MapCloning(recipe)
            | Recipe::MapExtending(recipe)
            | Recipe::FireworkRocket(recipe)
            | Recipe::FireworkStar(recipe)
            | Recipe::FireworkStarFade(recipe)
            | Recipe::TippedArrow(recipe)
            | Recipe::BannerDuplicate(recipe)
            | Recipe::ShieldDecoration(recipe)
            | Recipe::ShulkerBoxColoring(recipe)
            | Recipe::SuspiciousStew(recipe)
            | Recipe::RepairItem(recipe) => recipe.encode(output),
            Recipe::Smelting(recipe)
            | Recipe::Blasting(recipe)
            | Recipe::Smoking(recipe)
            | Recipe::CampfireCooking(recipe) => recipe.encode(output),
            Recipe::Stonecutting {
                id,
                group,
                ingredient,
                result,
            } => {
                id.encode(output)?;
                group.encode(output)?;
                ingredient.encode(output)?;
                result.encode(output)
            }
            Recipe::Smithing {
                id,
                base,
                addition,
                result,
            } => {
                id.encode(output)?;
                base.encode(output)?;
                addition.encode(output)?;
                result.encode(output)
            }
        }
    }
}

impl<'a> Decode<'a> for Recipe {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(match String::decode(input)?.as_str() {
            "minecraft:crafting_shaped" => {
                let id = Decode::decode(input)?;
                let width = VarI32::decode(input)?;
                let height = VarI32::decode(input)?;
                let group = Decode::decode(input)?;
                let category = Decode::decode(input)?;
                let ingredient_count = width.0 * height.0;
                let mut ingredients = Vec::with_capacity(ingredient_count as usize);
                for _ in 0..ingredient_count {
                    ingredients.push(Decode::decode(input)?);
                }
                let result = Decode::decode(input)?;
                let show_notification = Decode::decode(input)?;
                Recipe::Shaped {
                    id,
                    group,
                    category,
                    ingredients,
                    width,
                    height,
                    result,
                    show_notification,
                }
            }
            "minecraft:crafting_shapeless" => Recipe::Shapeless {
                id: Decode::decode(input)?,
                group: Decode::decode(input)?,
                category: Decode::decode(input)?,
                ingredients: Decode::decode(input)?,
                result: Decode::decode(input)?,
            },
            "minecraft:crafting_special_armordye" => Recipe::ArmorDye(Decode::decode(input)?),
            "minecraft:crafting_special_bookcloning" => Recipe::BookCloning(Decode::decode(input)?),
            "minecraft:crafting_special_mapcloning" => Recipe::MapCloning(Decode::decode(input)?),
            "minecraft:crafting_special_mapextending" => {
                Recipe::MapExtending(Decode::decode(input)?)
            }
            "minecraft:crafting_special_firework_rocket" => {
                Recipe::FireworkRocket(Decode::decode(input)?)
            }
            "minecraft:crafting_special_firework_star" => {
                Recipe::FireworkStar(Decode::decode(input)?)
            }
            "minecraft:crafting_special_firework_star_fade" => {
                Recipe::FireworkStarFade(Decode::decode(input)?)
            }
            "minecraft:crafting_special_tippedarrow" => Recipe::TippedArrow(Decode::decode(input)?),
            "minecraft:crafting_special_bannerduplicate" => {
                Recipe::BannerDuplicate(Decode::decode(input)?)
            }
            "minecraft:crafting_special_shielddecoration" => {
                Recipe::ShieldDecoration(Decode::decode(input)?)
            }
            "minecraft:crafting_special_shulkerboxcoloring" => {
                Recipe::ShulkerBoxColoring(Decode::decode(input)?)
            }
            "minecraft:crafting_special_suspiciousstew" => {
                Recipe::SuspiciousStew(Decode::decode(input)?)
            }
            "minecraft:crafting_special_repairitem" => Recipe::RepairItem(Decode::decode(input)?),
            "minecraft:smelting" => Recipe::Smelting(Decode::decode(input)?),
            "minecraft:blasting" => Recipe::Blasting(Decode::decode(input)?),
            "minecraft:smoking" => Recipe::Smoking(Decode::decode(input)?),
            "minecraft:campfire_cooking" => Recipe::CampfireCooking(Decode::decode(input)?),
            "minecraft:stonecutting" => Recipe::Stonecutting {
                id: Decode::decode(input)?,
                group: Decode::decode(input)?,
                ingredient: Decode::decode(input)?,
                result: Decode::decode(input)?,
            },
            "minecraft:smithing" => Recipe::Smithing {
                id: Decode::decode(input)?,
                base: Decode::decode(input)?,
                addition: Decode::decode(input)?,
                result: Decode::decode(input)?,
            },
            _ => return Err(Error::UnknownVariant(0)),
        })
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum RecipeBookType {
    Crafting,
    Furnace,
    BlastFurnace,
    Smoker,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Registries {
    #[serde(rename = "minecraft:dimension_type")]
    pub dimension_type: Registry<DimensionType>,
    #[serde(rename = "minecraft:worldgen/biome")]
    pub biome: Registry<Biome>,
    #[serde(rename = "minecraft:chat_type")]
    pub chat_type: Registry<ChatType>,
    #[serde(rename = "minecraft:damage_type")]
    pub damage_type: Registry<DamageType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DamageType {
    pub message_id: String,
    pub exhaustion: f32,
    pub scaling: String,
    pub effects: Option<String>,
    pub death_message_type: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Registry<T> {
    #[serde(rename = "type")]
    pub type_: String,
    pub value: Vec<RegistryEntry<T>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegistryEntry<T> {
    pub name: String,
    pub id: u32,
    pub element: T,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct SimpleCooking {
    pub id: String,
    pub group: String,
    pub category: VarI32,
    pub ingredient: Vec<Option<ItemStack>>,
    pub result: Option<ItemStack>,
    pub experience: f32,
    pub cooking_time: VarI32,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct SimpleRecipe {
    pub id: String,
    pub category: VarI32,
}

#[derive(Clone, Debug)]
pub enum Sound {
    Id(i32),
    Name(String),
}

impl Encode for Sound {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        match self {
            Sound::Id(id) => VarI32(id + 1).encode(output),
            Sound::Name(name) => {
                VarI32(0).encode(output)?;
                name.encode(output)
            }
        }
    }
}

impl<'a> Decode<'a> for Sound {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        let id = VarI32::decode(input)?;
        Ok(if id.0 == 0 {
            Sound::Name(Decode::decode(input)?)
        } else {
            Sound::Id(id.0 - 1)
        })
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum SoundSource {
    Master,
    Music,
    Records,
    Weather,
    Blocks,
    Hostile,
    Neutral,
    Players,
    Ambient,
    Voice,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Status {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Component>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub players: Option<StatusPlayers>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<StatusVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusVersion {
    pub name: String,
    pub protocol: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusPlayers {
    pub max: i32,
    pub online: i32,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub sample: Vec<StatusPlayersSample>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusPlayersSample {
    pub id: String,
    pub name: String,
}

impl Encode for User {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        self.id.encode(output)?;
        self.name.encode(output)?;
        self.properties.encode(output)
    }
}

impl<'a> Decode<'a> for User {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(User {
            id: Decode::decode(input)?,
            name: Decode::decode(input)?,
            properties: Decode::decode(input)?,
        })
    }
}

impl Encode for UserProperty {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        self.name.encode(output)?;
        self.value.encode(output)?;
        self.signature.encode(output)
    }
}

impl<'a> Decode<'a> for UserProperty {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(UserProperty {
            name: Decode::decode(input)?,
            value: Decode::decode(input)?,
            signature: Decode::decode(input)?,
        })
    }
}
