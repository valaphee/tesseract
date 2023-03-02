use std::io::Write;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use glam::{DVec3, IVec3};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use serde_value::Value;
use uuid::Uuid;

use crate::{Decode, Encode, Error, Result};

//=================================================================================== PRIMITIVE ====

impl Encode for bool {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        if *self { 1u8 } else { 0u8 }.encode(output)
    }
}

impl<'a> Decode<'a> for bool {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(match u8::decode(input)? {
            0 => false,
            1 => true,
            _ => todo!(),
        })
    }
}

impl Encode for u8 {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
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
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
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
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
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
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i16::<BigEndian>(*self)?;
        Ok(())
    }
}

impl<'a> Decode<'a> for i16 {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(input.read_i16::<BigEndian>()?)
    }
}

impl Encode for i32 {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_i32::<BigEndian>(*self)?;
        Ok(())
    }
}

impl<'a> Decode<'a> for i32 {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(input.read_i32::<BigEndian>()?)
    }
}

#[derive(Clone, Debug)]
pub struct VarInt(pub i32);

impl VarInt {
    pub fn len(&self) -> usize {
        match self.0 {
            0 => 1,
            n => (31 - n.leading_zeros() as usize) / 7 + 1,
        }
    }
}

impl Encode for VarInt {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
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

impl<'a> Decode<'a> for VarInt {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        let mut value = 0;
        let mut shift = 0;
        while shift <= 35 {
            let head = input.read_u8()?;
            value |= (head as i32 & 0b01111111) << shift;
            if head & 0b10000000 == 0 {
                return Ok(VarInt(value));
            }
            shift += 7;
        }
        Err(Error::VarIntTooWide(35))
    }
}

impl Encode for i64 {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
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
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
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
pub struct VarLong(pub i64);

impl VarLong {
    pub fn len(&self) -> usize {
        match self.0 {
            0 => 1,
            n => (63 - n.leading_zeros() as usize) / 7 + 1,
        }
    }
}

impl Encode for VarLong {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
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

impl<'a> Decode<'a> for VarLong {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        let mut value = 0;
        let mut shift = 0;
        while shift <= 70 {
            let head = input.read_u8()?;
            value |= (head as i64 & 0b01111111) << shift;
            if head & 0b10000000 == 0 {
                return Ok(VarLong(value));
            }
            shift += 7;
        }
        Err(Error::VarIntTooWide(70))
    }
}

impl Encode for f32 {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
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
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
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
            fn encode<W: Write>(&self, _output: &mut W) -> Result<()> {
                let ($($ty,)*) = self;
                $(
                    $ty.encode(_output)?;
                )*
                Ok(())
            }
        }

        impl<'a, $($ty: Decode<'a>,)*> Decode<'a> for ($($ty,)*) {
            fn decode(_input: &mut &'a [u8]) -> Result<Self> {
                Ok(($($ty::decode(_input)?,)*))
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

//======================================================================================= ARRAY ====

impl<T, const N: usize> Encode for [T; N]
where
    T: Encode,
{
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        for value in self {
            value.encode(output)?;
        }
        Ok(())
    }
}

impl<'a, T, const N: usize> Decode<'a> for [T; N]
where
    T: Decode<'a>,
{
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(std::array::try_from_fn(|_| T::decode(input))?)
    }
}

#[derive(Clone, Debug)]
pub struct TrailingBytes(pub Vec<u8>);

impl Encode for TrailingBytes {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_all(&self.0)?;
        Ok(())
    }
}

impl<'a> Decode<'a> for TrailingBytes {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(TrailingBytes(input.to_vec()))
    }
}

//========================================================================================= STD ====

impl<T> Encode for Option<T>
where
    T: Encode,
{
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        match self {
            None => false.encode(output),
            Some(value) => {
                true.encode(output)?;
                value.encode(output)
            }
        }
    }
}

impl<'a, T> Decode<'a> for Option<T>
where
    T: Decode<'a>,
{
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(match bool::decode(input)? {
            true => Some(T::decode(input)?),
            false => None,
        })
    }
}

impl<T> Encode for Vec<T>
where
    T: Encode,
{
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        VarInt(self.len() as i32).encode(output)?;
        for item in self.iter() {
            item.encode(output)?;
        }
        Ok(())
    }
}

impl<'a, T> Decode<'a> for Vec<T>
where
    T: Decode<'a>,
{
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        let length = VarInt::decode(input)?.0 as usize;
        let mut value = Vec::with_capacity(length);
        for _ in 0..length {
            value.push(T::decode(input)?);
        }
        Ok(value)
    }
}

impl Encode for str {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        VarInt(self.len() as i32).encode(output)?;
        output.write_all(self.as_bytes())?;
        Ok(())
    }
}

impl<'a> Decode<'a> for &'a str {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        let length = VarInt::decode(input)?.0 as usize;
        let (bytes, input_) = input.split_at(length);
        *input = input_;
        Ok(std::str::from_utf8(bytes)?)
    }
}

impl Encode for String {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        self.as_bytes().to_vec().encode(output)?;
        Ok(())
    }
}

impl<'a> Decode<'a> for String {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(String::from_utf8(Vec::<u8>::decode(input)?)?)
    }
}

impl Encode for Uuid {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
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
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        match (self.x, self.y, self.z) {
            (-0x2000000..=0x1ffffff, -0x800..=0x7ff, -0x2000000..=0x1ffffff) => {
                ((self.x as u64) << 38 | (self.z as u64) << 38 >> 26 | (self.y as u64) & 0xFFF)
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
            y: (value << 26 >> 38) as i32,
            z: (value << 52 >> 52) as i32,
        })
    }
}

impl Encode for DVec3 {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
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

#[derive(Clone, Debug)]
pub struct Angle(pub f32);

impl Encode for Angle {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_u8((self.0.rem_euclid(360.0) / 360.0 * u8::MAX as f32).round() as u8)?;
        Ok(())
    }
}

impl<'a> Decode<'a> for Angle {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(Self(u8::decode(input)? as f32 / u8::MAX as f32 * 360.0))
    }
}

//======================================================================================== GAME ====

#[derive(Clone, Debug, Encode, Decode)]
pub struct Advancement {
    pub parent_id: Option<String>,
    pub display: Option<AdvancementDisplayInfo>,
    pub criteria: Vec<String>,
    pub requirements: Vec<Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct AdvancementDisplayInfo {
    pub title: String,
    pub description: String,
    pub icon: ItemStack,
    pub frame: AdvancementFrameType,
    pub background: Option<String>,
    pub show_toast: bool,
    pub hidden: bool,
    pub x: f32,
    pub y: f32,
}

impl Encode for AdvancementDisplayInfo {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
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
        let show_toast = flags & (1 << 1) != 0;
        let hidden = flags & (1 << 2) != 0;
        let x = Decode::decode(input)?;
        let y = Decode::decode(input)?;
        Ok(Self {
            title,
            description,
            icon,
            frame,
            background,
            show_toast,
            hidden,
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
    pub precipitation: BiomePrecipitation,
    pub temperature: f32,
    pub temperature_modifier: Option<BiomeTemperatureModifier>,
    pub downfall: f32,
    pub effects: BiomeEffects,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BiomePrecipitation {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "rain")]
    Rain,
    #[serde(rename = "snow")]
    Snow,
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

#[derive(Clone, Debug, Encode, Decode)]
pub struct ChatType {
    pub chat_type: VarInt,
    pub name: String,
    pub target_name: String,
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
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        output.read_u8(match self {
            Difficulty::Peaceful => 0,
            Difficulty::Easy => 1,
            Difficulty::Normal => 2,
            Difficulty::Hard => 3,
        })?;
        Ok(())
    }
}

impl<'a> Decode<'a> for Difficulty {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(match input.read_u8()? {
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
    pub monster_spawn_light_level: i32,
    pub monster_spawn_block_light_limit: i32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
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
pub struct GameProfile {
    pub id: Uuid,
    pub name: String,
    pub properties: Vec<GameProfileProperty>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct GameProfileProperty {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
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

#[derive(Clone, Debug, Encode, Decode)]
pub struct ItemStack {
    pub item: VarInt,
    pub count: i8,
    pub tag: Nbt<Value>,
}

#[derive(Clone, Debug)]
pub struct Json<T>(pub T);

impl<T> Encode for Json<T>
where
    T: Serialize,
{
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        serde_json::to_string(&self.0)?.encode(output)
    }
}

impl<'a, T> Decode<'a> for Json<T>
where
    T: Deserialize<'a>,
{
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(Json(serde_json::from_str(Decode::decode(input)?)?))
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct LastSeenMessages {
    pub offset: VarInt,
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
    pub name: Option<String>,
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
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
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
pub struct Nbt<T>(pub T);

impl<T> Encode for Nbt<T>
where
    T: Serialize,
{
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        output.write_all(&tesseract_serde_nbt::ser::to_vec(&self.0)?)?;
        Ok(())
    }
}

impl<'a, T> Decode<'a> for Nbt<T>
where
    T: Deserialize<'a>,
{
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(Nbt(tesseract_serde_nbt::de::from_slice(input)?))
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
    #[serde(rename = "minecraft:worldgen/biome")]
    pub biome_registry: Registry<Biome>,
    #[serde(rename = "minecraft:dimension_type")]
    pub dimension_type_registry: Registry<DimensionType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Registry<T> {
    #[serde(rename = "type")]
    pub _type: String,
    pub value: Vec<RegistryEntry<T>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegistryEntry<T> {
    pub name: String,
    pub id: u32,
    pub element: T,
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
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub players: Option<StatusPlayers>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<StatusVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    pub previews_chat: bool,
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
    pub sample: Vec<StatusPlayersSample>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusPlayersSample {
    pub id: String,
    pub name: String,
}
