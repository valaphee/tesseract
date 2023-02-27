use std::io::Write;

use anyhow::{bail, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use glam::IVec3;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Decode, Encode};

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
        todo!()
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
            _ => bail!(""),
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

#[derive(Encode, Decode)]
pub enum Anchor {
    Feet,
    Eyes,
}

#[derive(Encode, Decode)]
pub enum BossEventColor {
    Pink,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White,
}

#[derive(Encode, Decode)]
pub enum BossEventOverlay {
    Progress,
    Notched6,
    Notched10,
    Notched12,
    Notched20,
}

#[derive(Encode, Decode)]
pub struct ChatSession {
    session_id: Uuid,
    expires_at: i64,
    public_key: Vec<u8>,
    key_signature: Vec<u8>,
}

#[derive(Encode, Decode)]
pub struct ChatType {
    chat_type: VarInt,
    name: String,
    target_name: String,
}

#[derive(Encode, Decode)]
pub enum ChatVisibility {
    Full,
    System,
    Hidden,
}

#[derive(Encode, Decode)]
pub enum ClickType {
    Pickup,
    QuickMove,
    Swap,
    Clone,
    Throw,
    QuickCraft,
    PickupAll,
}

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

#[derive(Encode, Decode)]
pub struct GameProfile {
    id: Uuid,
    name: String,
    properties: Vec<GameProfileProperty>,
}

#[derive(Encode, Decode)]
pub struct GameProfileProperty {
    name: String,
    value: String,
    signature: Option<String>,
}

#[derive(Encode, Decode)]
pub enum GameType {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Encode, Decode)]
pub enum Hand {
    MainHand,
    OffHand,
}

#[derive(Encode, Decode)]
pub struct ItemStack {
    item: VarInt,
    count: i8,
    tag: (),
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

#[derive(Encode, Decode)]
pub struct LastSeenMessages {
    offset: VarInt,
    acknowledged: [u8; 3],
}

#[derive(Encode, Decode)]
pub enum MainHand {
    Left,
    Right,
}

#[derive(Encode, Decode)]
pub struct MapDecoration {
    type_: MapDecorationType,
    x: i8,
    y: i8,
    rot: i8,
    name: Option<String>,
}

#[derive(Encode, Decode)]
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

pub struct MapPatch {
    width: u8,
    height: u8,
    start_x: u8,
    start_y: u8,
    map_colors: Vec<u8>,
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

#[derive(Encode, Decode)]
pub struct MerchantOffer {
    base_cost_a: Option<ItemStack>,
    result: Option<ItemStack>,
    cost_b: Option<ItemStack>,
    out_of_stock: bool,
    uses: i32,
    max_uses: i32,
    xp: i32,
    special_price_diff: i32,
    price_multiplier: f32,
    demand: i32,
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

#[derive(Encode, Decode)]
pub enum RecipeBookType {
    Crafting,
    Furnace,
    BlastFurnace,
    Smoker,
}

#[derive(Encode, Decode)]
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
