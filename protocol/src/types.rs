use std::io::Write;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use glam::IVec3;
use serde::{Deserialize, Serialize};
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

#[derive(Debug)]
pub struct VarInt(pub i32);

impl Encode for VarInt {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        let encoded_without_trailing_bits =
            unsafe { std::arch::x86_64::_pdep_u64(self.0 as u64, 0x0000000000037F7F) };
        let encoded_length = 8 - ((encoded_without_trailing_bits.leading_zeros() - 1) >> 3);
        let encoded = encoded_without_trailing_bits
            | (0x8080808080808080 & (0xFFFFFFFFFFFFFFFF >> (((8 - encoded_length + 1) << 3) - 1)));
        output.write_all(unsafe {
            encoded
                .to_le_bytes()
                .get_unchecked(..encoded_length as usize)
        })?;
        Ok(())
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

#[derive(Debug)]
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

//======================================================================================== GAME ====

#[derive(Debug, Encode, Decode)]
pub enum Anchor {
    Feet,
    Eyes,
}

#[derive(Debug, Encode, Decode)]
pub enum BossEventColor {
    Pink,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White,
}

#[derive(Debug, Encode, Decode)]
pub enum BossEventOverlay {
    Progress,
    Notched6,
    Notched10,
    Notched12,
    Notched20,
}

#[derive(Debug, Encode, Decode)]
pub struct ChatSession {
    pub session_id: Uuid,
    pub expires_at: i64,
    pub public_key: Vec<u8>,
    pub key_signature: Vec<u8>,
}

#[derive(Debug, Encode, Decode)]
pub struct ChatType {
    pub chat_type: VarInt,
    pub name: String,
    pub target_name: String,
}

#[derive(Debug, Encode, Decode)]
pub enum ChatVisibility {
    Full,
    System,
    Hidden,
}

#[derive(Debug, Encode, Decode)]
pub enum ClickType {
    Pickup,
    QuickMove,
    Swap,
    Clone,
    Throw,
    QuickCraft,
    PickupAll,
}

#[derive(Debug)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

impl Encode for Difficulty {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        match self {
            Difficulty::Peaceful => 0i8,
            Difficulty::Easy => 1i8,
            Difficulty::Normal => 2i8,
            Difficulty::Hard => 3i8,
        }
        .encode(output)
    }
}

impl<'a> Decode<'a> for Difficulty {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
        Ok(match input.read_i8()? {
            0 => Difficulty::Peaceful,
            1 => Difficulty::Easy,
            2 => Difficulty::Normal,
            3 => Difficulty::Hard,
            _ => unreachable!(),
        })
    }
}

#[derive(Debug, Encode, Decode)]
pub struct GameProfile {
    pub id: Uuid,
    pub name: String,
    pub properties: Vec<GameProfileProperty>,
}

#[derive(Debug, Encode, Decode)]
pub struct GameProfileProperty {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}

#[derive(Debug, Encode, Decode)]
pub enum GameType {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Debug, Encode, Decode)]
pub enum Hand {
    MainHand,
    OffHand,
}

#[derive(Debug, Encode, Decode)]
pub enum Intention {
    Game,
    Status,
    Login,
}

#[derive(Debug, Encode, Decode)]
pub struct ItemStack {
    pub item: VarInt,
    pub count: i8,
    pub tag: (),
}

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

#[derive(Debug, Encode, Decode)]
pub struct LastSeenMessages {
    pub offset: VarInt,
    pub acknowledged: [u8; 3],
}

#[derive(Debug, Encode, Decode)]
pub enum MainHand {
    Left,
    Right,
}

#[derive(Debug, Encode, Decode)]
pub struct MapDecoration {
    pub type_: MapDecorationType,
    pub x: i8,
    pub y: i8,
    pub rot: i8,
    pub name: Option<String>,
}

#[derive(Debug, Encode, Decode)]
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

#[derive(Debug)]
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

#[derive(Debug, Encode, Decode)]
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

#[derive(Debug, Encode, Decode)]
pub enum RecipeBookType {
    Crafting,
    Furnace,
    BlastFurnace,
    Smoker,
}

#[derive(Debug, Encode, Decode)]
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

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct StatusVersion {
    pub name: String,
    pub protocol: i32,
}

#[derive(Serialize, Deserialize)]
pub struct StatusPlayers {
    pub max: i32,
    pub online: i32,
    pub sample: Vec<StatusPlayersSample>,
}

#[derive(Serialize, Deserialize)]
pub struct StatusPlayersSample {
    pub id: String,
    pub name: String,
}
