use std::{collections::HashMap, io::Write};

use glam::{DVec3, IVec3};
use uuid::Uuid;

use mojang_session_api::models::User;

use crate::{
    types::{
        Advancement, Anchor, Angle, BossEventColor, BossEventOverlay, ChatSession, ChatTypeBound,
        Component, Difficulty, EntityData, EquipmentSlot, GameType, Hand, ItemStack, Json,
        MapDecoration, MapPatch, MerchantOffer, Nbt, Recipe, Registries, Sound, SoundSource,
        TrailingBytes, VarI32, VarI64,
    },
    Decode, Encode, Error, Result,
};

#[derive(Clone, Debug, Encode, Decode)]
pub enum GamePacket {
    BundleDelimiter,
    AddEntity {
        id: VarI32,
        uuid: Uuid,
        type_: VarI32,
        pos: DVec3,
        pitch: Angle,
        yaw: Angle,
        head_yaw: Angle,
        data: VarI32,
        xa: i16,
        ya: i16,
        za: i16,
    },
    AddExperienceOrb {
        id: VarI32,
        pos: DVec3,
        value: i16,
    },
    AddPlayer {
        entity_id: VarI32,
        player_id: Uuid,
        pos: DVec3,
        yaw: Angle,
        pitch: Angle,
    },
    Animate {
        id: VarI32,
        action: AnimatePacketAction,
    },
    AwardStats {
        stats: Vec<(VarI32, VarI32, VarI32)>,
    },
    BlockChangedAck {
        sequence: VarI32,
    },
    BlockDestruction {
        id: VarI32,
        pos: IVec3,
        progress: u8,
    },
    BlockEntityData {
        pos: IVec3,
        type_: VarI32,
        tag: Nbt<serde_value::Value>,
    },
    BlockEvent {
        pos: IVec3,
        b0: u8,
        b1: u8,
        block: VarI32,
    },
    BlockUpdate {
        pos: IVec3,
        block_state: VarI32,
    },
    BossEvent {
        id: Uuid,
        operation: BossEventPacketOperation,
    },
    ChangeDifficulty {
        difficulty: Difficulty,
        locked: bool,
    },
    ChunksBiomes {
        x: VarI32,
        z: VarI32,
        buffer: Vec<u8>,
    },
    ClearTitles {
        reset_times: bool,
    },
    CommandSuggestions {
        id: VarI32,
        suggestions_start: VarI32,
        suggestions_length: VarI32,
        suggestions: Vec<(String, Option<Json<Component>>)>,
    },
    Commands {
        entries: Vec<CommandsPacketEntry>,
        root_index: VarI32,
    },
    ContainerClose {
        container_id: u8,
    },
    ContainerSetContent {
        container_id: u8,
        state_id: VarI32,
        items: Vec<Option<ItemStack>>,
        carried_item: Option<ItemStack>,
    },
    ContainerSetData {
        container_id: u8,
        id: i16,
        value: i16,
    },
    ContainerSetSlot {
        container_id: i8,
        state_id: VarI32,
        slot: i16,
        item_stack: Option<ItemStack>,
    },
    Cooldown {
        item: VarI32,
        duration: VarI32,
    },
    CustomChatCompletions {
        action: CustomChatCompletionsPacketAction,
        entries: Vec<String>,
    },
    CustomPayload {
        identifier: String,
        data: TrailingBytes<{ 1 << 20 }>,
    },
    DamageEvent {
        entity_id: VarI32,
        source_type_id: VarI32,
        source_cause_id: VarI32,
        source_direct_id: VarI32,
        source_position: Option<DVec3>,
    },
    DeleteChat {
        message_signature: Vec<u8>,
    },
    Disconnect {
        reason: String,
    },
    DisguisedChatPacket {
        message: Json<Component>,
        chat_type: ChatTypeBound,
    },
    EntityEvent {
        entity_id: i32,
        event_id: i8,
    },
    Explode {
        pos: DVec3,
        power: f32,
        to_blow: Vec<i8>,
        knockback_x: f32,
        knockback_y: f32,
        knockback_z: f32,
    },
    ForgetLevelChunk {
        x: i32,
        z: i32,
    },
    GameEvent {
        event: GameEventPacketEvent,
        param: f32,
    },
    HorseScreenOpen {
        container_id: u8,
        size: VarI32,
        entity_id: i32,
    },
    HurtAnimation {
        id: VarI32,
        yaw: f32,
    },
    InitializeBorder {
        new_center_x: f64,
        new_center_z: f64,
        old_size: f64,
        new_size: f64,
        lerp_time: VarI64,
        new_absolute_max_size: VarI32,
        warning_blocks: VarI32,
        warning_time: VarI32,
    },
    KeepAlive {
        id: i64,
    },
    LevelChunkWithLight {
        x: i32,
        z: i32,
        chunk_data: LevelChunkPacketData,
        light_data: LightUpdatePacketData,
    },
    LevelEvent {
        type_: i32,
        pos: IVec3,
        data: i32,
        global_event: bool,
    },
    LevelParticles {
        particle_type: VarI32,
        override_limiter: bool,
        pos: DVec3,
        x_dist: f32,
        y_dist: f32,
        z_dist: f32,
        max_speed: f32,
        count: i32,
        particle: (),
    },
    LightUpdate {
        x: VarI32,
        z: VarI32,
        light_data: LightUpdatePacketData,
    },
    Login {
        player_id: i32,
        hardcore: bool,
        game_type: GameType,
        previous_game_type: i8,
        levels: Vec<String>,
        registry_holder: Nbt<Registries>,
        dimension_type: String,
        dimension: String,
        seed: i64,
        max_players: VarI32,
        chunk_radius: VarI32,
        simulation_distance: VarI32,
        reduced_debug_info: bool,
        show_death_screen: bool,
        is_debug: bool,
        is_flat: bool,
        last_death_location: Option<(String, IVec3)>,
    },
    MapItemData {
        map_id: VarI32,
        scale: i8,
        locked: bool,
        decorations: Option<Vec<MapDecoration>>,
        color_patch: Option<MapPatch>,
    },
    MerchantOffers {
        container_id: VarI32,
        offers: Vec<MerchantOffer>,
        villager_level: VarI32,
        villager_xp: VarI32,
        show_progress: bool,
        can_restock: bool,
    },
    MoveEntityPos {
        entity_id: VarI32,
        xa: i16,
        ya: i16,
        za: i16,
        on_ground: bool,
    },
    MoveEntityPosRot {
        entity_id: VarI32,
        xa: i16,
        ya: i16,
        za: i16,
        yaw: Angle,
        pitch: Angle,
        on_ground: bool,
    },
    MoveEntityRot {
        entity_id: VarI32,
        yaw: Angle,
        pitch: Angle,
        on_ground: bool,
    },
    MoveVehicle {
        pos: DVec3,
        yaw: f32,
        pitch: f32,
    },
    OpenBook {
        hand: Hand,
    },
    OpenScreen {
        container_id: VarI32,
        type_: VarI32,
        title: Json<Component>,
    },
    OpenSignEditor {
        pos: IVec3,
    },
    Ping {
        id: i32,
    },
    PlaceGhostRecipe {
        container_id: i8,
        recipe: String,
    },
    PlayerAbilities(PlayerAbilitiesPacket),
    PlayerChat {
        sender: Uuid,
        index: VarI32,
        signature: Option<[u8; 256]>,
        message: String,
        timestamp: i64,
        salt: i64,
        unsigned_content: Option<Json<Component>>,
        chat_type: ChatTypeBound,
    },
    PlayerCombatEnd {
        duration: VarI32,
        killer_id: i32,
    },
    PlayerCombatEnter,
    PlayerCombatKill {
        player_id: VarI32,
        killer_id: i32,
        message: Json<Component>,
    },
    PlayerInfoRemove {
        profile_ids: Vec<Uuid>,
    },
    PlayerInfoUpdate(PlayerInfoUpdatePacket),
    PlayerLookAt {
        from_anchor: Anchor,
        pos: DVec3,
        at_entity: Option<PlayerLookAtPacketAtEntity>,
    },
    PlayerPosition {
        pos: DVec3,
        yaw: f32,
        pitch: f32,
        relative_arguments: u8,
        id: VarI32,
    },
    Recipe(RecipePacket),
    RemoveEntities {
        entity_ids: Vec<VarI32>,
    },
    RemoveMobEffect {
        entity_id: VarI32,
        effect: VarI32,
    },
    ResourcePack {
        url: String,
        hash: String,
        required: bool,
        prompt: Option<Json<Component>>,
    },
    Respawn {
        dimension_type: String,
        dimension: String,
        seed: i64,
        player_game_type: GameType,
        previous_player_game_type: i8,
        is_debug: bool,
        is_flat: bool,
        keep_all_player_data: bool,
        last_death_location: Option<(String, IVec3)>,
    },
    RotateHead {
        entity_id: VarI32,
        head_yaw: Angle,
    },
    SectionBlocksUpdate(SectionBlocksUpdatePacket),
    SelectAdvancementsTab {
        tab: Option<String>,
    },
    ServerData {
        motd: Json<Component>,
        icon_base64: Option<String>,
        previews_chat: bool,
    },
    SetActionBarText {
        text: Json<Component>,
    },
    SetBorderCenter {
        new_center_x: f64,
        new_center_z: f64,
    },
    SetBorderLerpSize {
        old_size: f64,
        new_size: f64,
        lerp_time: VarI64,
    },
    SetBorderSize {
        size: f64,
    },
    SetBorderWarningDelay {
        warning_delay: VarI32,
    },
    SetBorderWarningDistance {
        warning_blocks: VarI32,
    },
    SetCamera {
        camera_id: VarI32,
    },
    SetCarriedItem {
        slot: i8,
    },
    SetChunkCacheCenter {
        x: VarI32,
        z: VarI32,
    },
    SetChunkCacheRadius {
        radius: VarI32,
    },
    SetDefaultSpawnPosition {
        pos: IVec3,
        yaw: f32,
    },
    SetDisplayObjective {
        slot: i8,
        objective_name: String,
    },
    SetEntityData {
        id: VarI32,
        packed_items: EntityData,
    },
    SetEntityLink {
        source_id: i32,
        dest_id: i32,
    },
    SetEntityMotion {
        id: VarI32,
        xa: i16,
        ya: i16,
        za: i16,
    },
    SetEquipment {
        entity: VarI32,
        slots: SetEquipmentPacketSlots,
    },
    SetExperience {
        experience_progress: f32,
        experience_level: VarI32,
        total_experience: VarI32,
    },
    SetHealth {
        health: f32,
        food: VarI32,
        saturation: f32,
    },
    SetObjective {
        objective_name: String,
        method: SetObjectivePacketMethod,
    },
    SetPassengers {
        vehicle: VarI32,
        passengers: Vec<VarI32>,
    },
    SetPlayerTeam {
        name: String,
        method: SetPlayerTeamPacketMethod,
    },
    SetScore {
        owner: String,
        method: SetScorePacketMethod,
    },
    SetSimulationDistance {
        simulation_distance: VarI32,
    },
    SetSubtitleText {
        text: Json<Component>,
    },
    SetTime {
        game_time: i64,
        day_time: i64,
    },
    SetTitleText {
        text: Json<Component>,
    },
    SetTitlesAnimation {
        fade_in: i32,
        stay: i32,
        fade_out: i32,
    },
    SoundEntity {
        sound: Sound,
        range: Option<f32>,
        source: SoundSource,
        id: VarI32,
        volume: f32,
        pitch: f32,
        seed: i64,
    },
    Sound {
        sound: Sound,
        range: Option<f32>,
        source: SoundSource,
        x: i32,
        y: i32,
        z: i32,
        volume: f32,
        pitch: f32,
        seed: i64,
    },
    StopSound(StopSoundPacket),
    SystemChat {
        content: Json<Component>,
        overlay: bool,
    },
    TabList {
        header: Json<Component>,
        footer: Json<Component>,
    },
    TagQuery {
        transaction_id: VarI32,
        tag: Nbt<serde_value::Value>,
    },
    TakeItemEntity {
        item_id: VarI32,
        player_id: VarI32,
        amount: VarI32,
    },
    TeleportEntity {
        id: VarI32,
        pos: DVec3,
        yaw: Angle,
        pitch: Angle,
        on_ground: bool,
    },
    UpdateAdvancements {
        reset: bool,
        added: Vec<(String, Advancement)>,
        removed: Vec<String>,
        progress: Vec<(String, Vec<(String, Option<i64>)>)>,
    },
    UpdateAttributes {
        entity_id: VarI32,
        attributes: Vec<(String, f64, Vec<(Uuid, f64, i8)>)>,
    },
    UpdateEnabledFeatures {
        features: Vec<String>,
    },
    UpdateMobEffect {
        entity_id: VarI32,
        id: VarI32,
        effect_amplifier: i8,
        effect_duration_ticks: VarI32,
        flags: u8,
        factor_data: Option<Nbt<serde_value::Value>>,
    },
    UpdateRecipes {
        recipes: Vec<Recipe>,
    },
    UpdateTags {
        tags: Vec<(String, Vec<(String, Vec<VarI32>)>)>,
    },
}

#[derive(Clone, Debug)]
pub enum AnimatePacketAction {
    SwingMainHand,
    WakeUp,
    SwingOffHand,
    CriticalHit,
    MagicCriticalHit,
}

impl Encode for AnimatePacketAction {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        match self {
            Self::SwingMainHand => 0u8,
            Self::WakeUp => 1u8,
            Self::SwingOffHand => 2u8,
            Self::CriticalHit => 3u8,
            Self::MagicCriticalHit => 4u8,
        }
        .encode(output)
    }
}

impl Decode for AnimatePacketAction {
    fn decode(input: &mut &[u8]) -> Result<Self> {
        Ok(match u8::decode(input)? {
            0 => Self::SwingMainHand,
            1 => Self::WakeUp,
            2 => Self::SwingOffHand,
            3 => Self::CriticalHit,
            4 => Self::MagicCriticalHit,
            variant => return Err(Error::UnknownVariant(variant as i32)),
        })
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum BossEventPacketOperation {
    Add {
        name: Json<Component>,
        progress: f32,
        color: BossEventColor,
        overlay: BossEventOverlay,
        properties: u8,
    },
    Remove,
    UpdateProgress {
        progress: f32,
    },
    UpdateName {
        name: Json<Component>,
    },
    UpdateStyle {
        color: BossEventColor,
        overlay: BossEventOverlay,
    },
    UpdateProperties {
        properties: u8,
    },
}

#[derive(Clone, Debug)]
pub struct CommandsPacketEntry {
    children: Vec<VarI32>,
    redirect: Option<VarI32>,
    stub: CommandsPacketNodeStub,
}

#[derive(Clone, Debug)]
pub enum CommandsPacketNodeStub {
    Root,
    Literal {
        id: String,
    },
    Argument {
        id: String,
        argument_type: CommandsPacketArgumentType,
        suggestion_id: Option<String>,
    },
}

impl Encode for CommandsPacketEntry {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        let mut flags = match &self.stub {
            CommandsPacketNodeStub::Root => 0i8,
            CommandsPacketNodeStub::Literal { .. } => 1i8,
            CommandsPacketNodeStub::Argument {
                suggestion_id: suggestions_type,
                ..
            } => {
                2i8 | if suggestions_type.is_some() {
                    1 << 4
                } else {
                    0
                }
            }
        };
        if self.redirect.is_some() {
            flags |= 1 << 3;
        }
        flags.encode(output)?;
        self.children.encode(output)?;
        if let Some(redirect_node) = self.redirect {
            redirect_node.encode(output)?;
        }
        match &self.stub {
            CommandsPacketNodeStub::Literal { id: name } => {
                name.encode(output)?;
            }
            CommandsPacketNodeStub::Argument {
                id: name,
                argument_type: parser,
                suggestion_id: suggestions_type,
            } => {
                name.encode(output)?;
                parser.encode(output)?;
                if let Some(suggestions_type) = suggestions_type {
                    suggestions_type.encode(output)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl Decode for CommandsPacketEntry {
    fn decode(input: &mut &[u8]) -> Result<Self> {
        let flags = i8::decode(input)?;
        let children = Decode::decode(input)?;
        let redirect_node = if flags & (1 << 3) != 0 {
            Some(Decode::decode(input)?)
        } else {
            None
        };
        let type_ = match flags & 3 {
            0 => CommandsPacketNodeStub::Root,
            1 => CommandsPacketNodeStub::Literal {
                id: Decode::decode(input)?,
            },
            2 => CommandsPacketNodeStub::Argument {
                id: Decode::decode(input)?,
                argument_type: Decode::decode(input)?,
                suggestion_id: if flags & (1 << 4) != 0 {
                    Some(Decode::decode(input)?)
                } else {
                    None
                },
            },
            _ => unreachable!(),
        };
        Ok(Self {
            children,
            redirect: redirect_node,
            stub: type_,
        })
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum CommandsPacketArgumentType {
    Bool,
    Float(CommandsPacketArgumentTypeNumber<f32>),
    Double(CommandsPacketArgumentTypeNumber<f64>),
    Integer(CommandsPacketArgumentTypeNumber<i32>),
    Long(CommandsPacketArgumentTypeNumber<i64>),
    String(CommandsPacketArgumentTypeString),
    Entity { flags: i8 },
    GameProfile,
    BlockPos,
    ColumnPos,
    Vec3,
    Vec2,
    BlockState,
    BlockPredicate,
    ItemStack,
    ItemPredicate,
    Color,
    Component,
    Message,
    Nbt,
    NbtTag,
    NbtPath,
    Objective,
    ObjectiveCriteria,
    Operation,
    Particle,
    Angle,
    Rotation,
    ScoreboardSlot,
    ScoreHolder { flags: i8 },
    Swizzle,
    Team,
    ItemSlot,
    ResourceLocation,
    Function,
    EntityAnchor,
    IntRange,
    FloatRange,
    Dimension,
    GameMode,
    Time,
    ResourceOrTag { registry: String },
    ResourceOrTagKey { registry: String },
    Resource { registry: String },
    ResourceKey { registry: String },
    TemplateMirror,
    TemplateRotation,
    Uuid,
}

#[derive(Clone, Debug)]
pub struct CommandsPacketArgumentTypeNumber<T> {
    min: Option<T>,
    max: Option<T>,
}

impl<T: Encode> Encode for CommandsPacketArgumentTypeNumber<T> {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        (if self.min.is_some() { 1i8 << 0 } else { 0 }
            | if self.max.is_some() { 1i8 << 1 } else { 0 })
        .encode(output)?;
        if let Some(min) = &self.min {
            min.encode(output)?;
        }
        if let Some(max) = &self.max {
            max.encode(output)?;
        }
        Ok(())
    }
}

impl<T: Decode> Decode for CommandsPacketArgumentTypeNumber<T> {
    fn decode(input: &mut &[u8]) -> Result<Self> {
        let flags = i8::decode(input)?;
        let min = if flags & (1 << 0) != 0 {
            Some(Decode::decode(input)?)
        } else {
            None
        };
        let max = if flags & (1 << 1) != 0 {
            Some(Decode::decode(input)?)
        } else {
            None
        };
        Ok(CommandsPacketArgumentTypeNumber { min, max })
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum CommandsPacketArgumentTypeString {
    SingleWord,
    QuotablePhrase,
    GreedyPhrase,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum CustomChatCompletionsPacketAction {
    Add,
    Remove,
    Set,
}

#[derive(Clone, Debug)]
pub enum GameEventPacketEvent {
    NoRespawnBlockAvailable,
    StartRaining,
    StopRaining,
    ChangeGameMode,
    WinGame,
    DemoEvent,
    ArrowHitPlayer,
    RainLevelChange,
    ThunderLevelChange,
    PufferFishSting,
    GuardianElderEffect,
    ImmediateRespawn,
}

impl Encode for GameEventPacketEvent {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        match self {
            Self::NoRespawnBlockAvailable => 0u8,
            Self::StartRaining => 1u8,
            Self::StopRaining => 2u8,
            Self::ChangeGameMode => 3u8,
            Self::WinGame => 4u8,
            Self::DemoEvent => 5u8,
            Self::ArrowHitPlayer => 6u8,
            Self::RainLevelChange => 7u8,
            Self::ThunderLevelChange => 8u8,
            Self::PufferFishSting => 9u8,
            Self::GuardianElderEffect => 10u8,
            Self::ImmediateRespawn => 11u8,
        }
        .encode(output)
    }
}

impl Decode for GameEventPacketEvent {
    fn decode(input: &mut &[u8]) -> Result<Self> {
        Ok(match u8::decode(input)? {
            0 => Self::NoRespawnBlockAvailable,
            1 => Self::StartRaining,
            2 => Self::StopRaining,
            3 => Self::ChangeGameMode,
            4 => Self::WinGame,
            5 => Self::DemoEvent,
            6 => Self::ArrowHitPlayer,
            7 => Self::RainLevelChange,
            8 => Self::ThunderLevelChange,
            9 => Self::PufferFishSting,
            10 => Self::GuardianElderEffect,
            11 => Self::ImmediateRespawn,
            variant => return Err(Error::UnknownVariant(variant as i32)),
        })
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct LevelChunkPacketData {
    pub heightmaps: Nbt<serde_value::Value>,
    pub buffer: Vec<u8>,
    pub block_entities_data: Vec<LevelChunkPacketDataBlockEntity>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct LevelChunkPacketDataBlockEntity {
    pub xz: i8,
    pub y: i16,
    pub type_: VarI32,
    pub data: Nbt<serde_value::Value>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct LightUpdatePacketData {
    pub trust_edges: bool,
    pub sky_y_mask: Vec<i64>,
    pub block_y_mask: Vec<i64>,
    pub empty_sky_y_mask: Vec<i64>,
    pub empty_block_y_mask: Vec<i64>,
    pub sky_updates: Vec<Vec<u8>>,
    pub block_updates: Vec<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct PlayerAbilitiesPacket {
    pub invulnerable: bool,
    pub is_flying: bool,
    pub can_fly: bool,
    pub instabuild: bool,
    pub flying_speed: f32,
    pub walking_speed: f32,
}

impl Encode for PlayerAbilitiesPacket {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        let mut flags = 0i8;
        if self.invulnerable {
            flags |= 1 << 0;
        }
        if self.is_flying {
            flags |= 1 << 1;
        }
        if self.can_fly {
            flags |= 1 << 2;
        }
        if self.instabuild {
            flags |= 1 << 3;
        }
        flags.encode(output)?;
        self.flying_speed.encode(output)?;
        self.walking_speed.encode(output)
    }
}

impl Decode for PlayerAbilitiesPacket {
    fn decode(input: &mut &[u8]) -> Result<Self> {
        let flags = i8::decode(input)?;
        let flying_speed = Decode::decode(input)?;
        let walking_speed = Decode::decode(input)?;
        Ok(Self {
            invulnerable: flags & (1 << 0) != 0,
            is_flying: flags & (1 << 1) != 0,
            can_fly: flags & (1 << 2) != 0,
            instabuild: flags & (1 << 3) != 0,
            flying_speed,
            walking_speed,
        })
    }
}

#[derive(Clone, Debug)]
pub struct PlayerInfoUpdatePacket {
    pub entries: Vec<PlayerInfoUpdatePacketEntry>,
}

#[derive(Clone, Debug)]
pub struct PlayerInfoUpdatePacketEntry {
    pub profile_id: Uuid,
    pub profile: Option<User>,
    pub chat_session: Option<Option<ChatSession>>,
    pub game_mode: Option<GameType>,
    pub listed: Option<bool>,
    pub latency: Option<VarI32>,
    pub display_name: Option<Option<Json<Component>>>,
}

impl Encode for PlayerInfoUpdatePacket {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        let first_entry = self.entries.first().unwrap();
        let add_player = first_entry.profile.is_some();
        let initialize_chat = first_entry.chat_session.is_some();
        let update_game_mode = first_entry.game_mode.is_some();
        let update_listed = first_entry.listed.is_some();
        let update_latency = first_entry.latency.is_some();
        let update_display_name = first_entry.display_name.is_some();
        let mut actions = 0i8;
        if add_player {
            actions |= 1 << 0;
        }
        if initialize_chat {
            actions |= 1 << 1;
        }
        if update_game_mode {
            actions |= 1 << 2;
        }
        if update_listed {
            actions |= 1 << 3;
        }
        if update_latency {
            actions |= 1 << 4;
        }
        if update_display_name {
            actions |= 1 << 5;
        }
        actions.encode(output)?;
        VarI32(self.entries.len() as i32).encode(output)?;
        for entry in &self.entries {
            if add_player {
                entry.profile.as_ref().unwrap().encode(output)?;
            } else {
                entry.profile_id.encode(output)?;
            }
            if initialize_chat {
                entry.chat_session.as_ref().unwrap().encode(output)?;
            }
            if update_game_mode {
                entry.game_mode.as_ref().unwrap().encode(output)?;
            }
            if update_listed {
                entry.listed.unwrap().encode(output)?;
            }
            if update_latency {
                entry.latency.unwrap().encode(output)?;
            }
            if update_display_name {
                entry.display_name.as_ref().unwrap().encode(output)?;
            }
        }
        Ok(())
    }
}

impl Decode for PlayerInfoUpdatePacket {
    fn decode(input: &mut &[u8]) -> Result<Self> {
        let actions = i8::decode(input)?;
        let add_player = actions & (1 << 0) != 0;
        let initialize_chat = actions & (1 << 1) != 0;
        let update_game_mode = actions & (1 << 2) != 0;
        let update_listed = actions & (1 << 3) != 0;
        let update_latency = actions & (1 << 4) != 0;
        let update_display_name = actions & (1 << 5) != 0;
        let entry_count = VarI32::decode(input)?.0;
        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            let (profile_id, profile) = if add_player {
                let profile = User::decode(input)?;
                (profile.id, Some(profile))
            } else {
                (Decode::decode(input)?, None)
            };
            let chat_session = if initialize_chat {
                Some(Decode::decode(input)?)
            } else {
                None
            };
            let game_mode = if update_game_mode {
                Some(Decode::decode(input)?)
            } else {
                None
            };
            let listed = if update_listed {
                Some(Decode::decode(input)?)
            } else {
                None
            };
            let latency = if update_latency {
                Some(Decode::decode(input)?)
            } else {
                None
            };
            let display_name = if update_display_name {
                Some(Decode::decode(input)?)
            } else {
                None
            };
            entries.push(PlayerInfoUpdatePacketEntry {
                profile_id,
                profile,
                chat_session,
                game_mode,
                listed,
                latency,
                display_name,
            });
        }
        Ok(Self { entries })
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct PlayerLookAtPacketAtEntity {
    pub entity: VarI32,
    pub to_anchor: Anchor,
}

#[derive(Clone, Debug)]
pub struct SectionBlocksUpdatePacket {
    section_pos: IVec3,
    suppress_light_updates: bool,
    position_and_states: Vec<SectionBlocksUpdatePacketPositionAndState>,
}

impl Encode for SectionBlocksUpdatePacket {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        match (self.section_pos.x, self.section_pos.y, self.section_pos.z) {
            (-0x200000..=0x1FFFFF, -0x80000..=0x7FFFF, -0x200000..=0x1FFFFF) => {
                ((self.section_pos.x as i64) << 42
                    | (self.section_pos.z as i64) << 20
                    | (self.section_pos.y as i64))
                    .encode(output)?
            }
            _ => unimplemented!(),
        }
        self.suppress_light_updates.encode(output)?;
        self.position_and_states.encode(output)?;
        Ok(())
    }
}

impl Decode for SectionBlocksUpdatePacket {
    fn decode(input: &mut &[u8]) -> Result<Self> {
        Ok(Self {
            section_pos: {
                let value = i64::decode(input)?;
                IVec3::new(
                    (value >> 42) as i32,
                    (value << 44 >> 44) as i32,
                    (value << 22 >> 42) as i32,
                )
            },
            suppress_light_updates: Decode::decode(input)?,
            position_and_states: Decode::decode(input)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct SectionBlocksUpdatePacketPositionAndState {
    x: u8,
    y: u8,
    z: u8,
    block_state: i64,
}

impl Encode for SectionBlocksUpdatePacketPositionAndState {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        match (self.block_state, self.x, self.y, self.z) {
            (0x0..=0x1FFFFFFFFFFFF, 0x0..=0xF, 0x0..=0xF, 0x0..=0xF) => VarI64(
                (self.block_state) << 12 | (((self.x as i64) << 8) | (self.z << 4 | self.y) as i64),
            )
            .encode(output),
            _ => unimplemented!(),
        }
    }
}

impl Decode for SectionBlocksUpdatePacketPositionAndState {
    fn decode(input: &mut &[u8]) -> Result<Self> {
        let value = VarI64::decode(input)?.0;
        Ok(Self {
            x: (value >> 8) as u8 & 0xF,
            y: value as u8 & 0xF,
            z: (value >> 4) as u8 & 0xF,
            block_state: value >> 12,
        })
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum RecipePacket {
    Init {
        crafting_recipe_book_open: bool,
        crafting_recipe_book_filter_active: bool,
        smelting_recipe_book_open: bool,
        smelting_recipe_book_filter_active: bool,
        blast_furnace_recipe_book_open: bool,
        blast_furnace_recipe_book_filter_active: bool,
        smoker_recipe_book_open: bool,
        smoker_recipe_book_filter_active: bool,
        recipes: Vec<String>,
        to_highlight: Vec<String>,
    },
    Add {
        crafting_recipe_book_open: bool,
        crafting_recipe_book_filter_active: bool,
        smelting_recipe_book_open: bool,
        smelting_recipe_book_filter_active: bool,
        blast_furnace_recipe_book_open: bool,
        blast_furnace_recipe_book_filter_active: bool,
        smoker_recipe_book_open: bool,
        smoker_recipe_book_filter_active: bool,
        recipes: Vec<String>,
    },
    Remove {
        crafting_recipe_book_open: bool,
        crafting_recipe_book_filter_active: bool,
        smelting_recipe_book_open: bool,
        smelting_recipe_book_filter_active: bool,
        blast_furnace_recipe_book_open: bool,
        blast_furnace_recipe_book_filter_active: bool,
        smoker_recipe_book_open: bool,
        smoker_recipe_book_filter_active: bool,
        recipes: Vec<String>,
    },
}

#[derive(Clone, Debug)]
pub struct SetEquipmentPacketSlots(HashMap<EquipmentSlot, Option<ItemStack>>);

impl Encode for SetEquipmentPacketSlots {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        if !self.0.is_empty() {
            for (&equipment_slot, item) in self.0.iter().take(self.0.len() - 1) {
                (u8::from(equipment_slot) | 0x80).encode(output)?;
                item.encode(output)?;
            }
            let (&equipment_slot, item) = self.0.iter().clone().last().unwrap();
            u8::from(equipment_slot).encode(output)?;
            item.encode(output)?;
        }
        Ok(())
    }
}

impl Decode for SetEquipmentPacketSlots {
    fn decode(input: &mut &[u8]) -> Result<Self> {
        let mut slots = HashMap::new();
        while {
            let equipment_slot_and_next_bit = u8::decode(input)?;
            slots.insert(
                EquipmentSlot::try_from(equipment_slot_and_next_bit & 0x7F).unwrap(),
                Decode::decode(input)?,
            );
            equipment_slot_and_next_bit & 0x80 != 0
        } {}
        Ok(Self(slots))
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum SetObjectivePacketMethod {
    Add {
        display_name: Json<Component>,
        render_type: VarI32,
    },
    Remove,
    Change {
        display_name: Json<Component>,
        render_type: VarI32,
    },
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum SetPlayerTeamPacketMethod {
    Add {
        display_name: Json<Component>,
        options: i8,
        nametag_visibility: String,
        collision_rule: String,
        color: VarI32,
        prefix: Json<Component>,
        suffix: Json<Component>,
        players: Vec<String>,
    },
    Remove,
    Change {
        display_name: Json<Component>,
        options: i8,
        nametag_visibility: String,
        collision_rule: String,
        color: VarI32,
        prefix: Json<Component>,
        suffix: Json<Component>,
    },
    Join {
        players: Vec<String>,
    },
    Leave {
        players: Vec<String>,
    },
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum SetScorePacketMethod {
    Change {
        objective_name: String,
        score: VarI32,
    },
    Remove {
        objective_name: String,
    },
}

#[derive(Clone, Debug)]
pub struct StopSoundPacket {
    pub source: Option<SoundSource>,
    pub name: Option<String>,
}

impl Encode for StopSoundPacket {
    fn encode<W: Write>(&self, output: &mut W) -> Result<()> {
        let mut flags = 0i8;
        if self.source.is_some() {
            flags |= 1 << 0;
        }
        if self.name.is_some() {
            flags |= 1 << 1;
        }
        flags.encode(output)?;
        if let Some(source) = &self.source {
            source.encode(output)?;
        }
        if let Some(name) = &self.name {
            name.encode(output)?;
        }
        Ok(())
    }
}

impl Decode for StopSoundPacket {
    fn decode(input: &mut &[u8]) -> Result<Self> {
        let flags = i8::decode(input)?;
        Ok(Self {
            source: if flags & 1 << 0 != 0 {
                Some(Decode::decode(input)?)
            } else {
                None
            },
            name: if flags & 1 << 1 != 0 {
                Some(Decode::decode(input)?)
            } else {
                None
            },
        })
    }
}
