use std::{collections::HashMap, io::Write};

use glam::{DVec3, IVec3};
use uuid::Uuid;

use mojang_session_api::models::User;

use crate::{
    types::{
        Advancement, Anchor, Angle, BossEventColor, BossEventOverlay, ChatSession, ChatTypeBound,
        Component, Difficulty, EntityData, EntityDataValue, EquipmentSlot, GameType, Hand,
        ItemStack, Json, MapDecoration, MapPatch, MerchantOffer, Nbt, Recipe, Registries, Sound,
        SoundSource, TrailingBytes, VarI32, VarI64,
    },
    Decode, Encode, Result,
};

#[derive(Encode, Decode, Clone, Debug)]
pub enum GamePacket<'a> {
    BundleDelimiter,
    AddEntity {
        #[using(VarI32)]
        id: i32,
        uuid: Uuid,
        #[using(VarI32)]
        type_: i32,
        pos: DVec3,
        #[using(Angle)]
        pitch: f32,
        #[using(Angle)]
        yaw: f32,
        #[using(Angle)]
        head_yaw: f32,
        #[using(VarI32)]
        data: i32,
        xa: i16,
        ya: i16,
        za: i16,
    },
    AddExperienceOrb {
        #[using(VarI32)]
        id: i32,
        pos: DVec3,
        value: i16,
    },
    AddPlayer {
        #[using(VarI32)]
        entity_id: i32,
        player_id: Uuid,
        pos: DVec3,
        #[using(Angle)]
        yaw: f32,
        #[using(Angle)]
        pitch: f32,
    },
    Animate {
        #[using(VarI32)]
        id: i32,
        action: AnimatePacketAction,
    },
    AwardStats {
        stats: Vec<(VarI32, VarI32, VarI32)>,
    },
    BlockChangedAck {
        #[using(VarI32)]
        sequence: i32,
    },
    BlockDestruction {
        #[using(VarI32)]
        id: i32,
        pos: IVec3,
        progress: u8,
    },
    BlockEntityData {
        pos: IVec3,
        #[using(VarI32)]
        type_: i32,
        tag: Nbt<serde_value::Value>,
    },
    BlockEvent {
        pos: IVec3,
        b0: u8,
        b1: u8,
        #[using(VarI32)]
        block: i32,
    },
    BlockUpdate {
        pos: IVec3,
        #[using(VarI32)]
        block_state: i32,
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
        #[using(VarI32)]
        x: i32,
        #[using(VarI32)]
        z: i32,
        buffer: Vec<u8>,
    },
    ClearTitles {
        reset_times: bool,
    },
    CommandSuggestions {
        #[using(VarI32)]
        id: i32,
        #[using(VarI32)]
        suggestions_start: i32,
        #[using(VarI32)]
        suggestions_length: i32,
        suggestions: Vec<(String, Option<Json<Component>>)>,
    },
    Commands {
        entries: Vec<CommandsPacketEntry>,
        #[using(VarI32)]
        root_index: i32,
    },
    ContainerClose {
        container_id: u8,
    },
    ContainerSetContent {
        container_id: u8,
        #[using(VarI32)]
        state_id: i32,
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
        #[using(VarI32)]
        state_id: i32,
        slot: i16,
        item_stack: Option<ItemStack>,
    },
    Cooldown {
        #[using(VarI32)]
        item: i32,
        #[using(VarI32)]
        duration: i32,
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
        #[using(VarI32)]
        entity_id: i32,
        #[using(VarI32)]
        source_type_id: i32,
        #[using(VarI32)]
        source_cause_id: i32,
        #[using(VarI32)]
        source_direct_id: i32,
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
        #[using(VarI32)]
        size: i32,
        entity_id: i32,
    },
    HurtAnimation {
        #[using(VarI32)]
        id: i32,
        yaw: f32,
    },
    InitializeBorder {
        new_center_x: f64,
        new_center_z: f64,
        old_size: f64,
        new_size: f64,
        #[using(VarI64)]
        lerp_time: i64,
        #[using(VarI32)]
        new_absolute_max_size: i32,
        #[using(VarI32)]
        warning_blocks: i32,
        #[using(VarI32)]
        warning_time: i32,
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
        #[using(VarI32)]
        particle_type: i32,
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
        #[using(VarI32)]
        x: i32,
        #[using(VarI32)]
        z: i32,
        light_data: LightUpdatePacketData,
    },
    Login {
        player_id: i32,
        hardcore: bool,
        game_type: GameType,
        previous_game_type: i8,
        levels: Vec<String>,
        registry_holder: Nbt<Registries<'a>>,
        dimension_type: String,
        dimension: String,
        seed: i64,
        #[using(VarI32)]
        max_players: i32,
        #[using(VarI32)]
        chunk_radius: i32,
        #[using(VarI32)]
        simulation_distance: i32,
        reduced_debug_info: bool,
        show_death_screen: bool,
        is_debug: bool,
        is_flat: bool,
        last_death_location: Option<(String, IVec3)>,
    },
    MapItemData {
        #[using(VarI32)]
        map_id: i32,
        scale: i8,
        locked: bool,
        decorations: Option<Vec<MapDecoration>>,
        color_patch: Option<MapPatch>,
    },
    MerchantOffers {
        #[using(VarI32)]
        container_id: i32,
        offers: Vec<MerchantOffer>,
        #[using(VarI32)]
        villager_level: i32,
        #[using(VarI32)]
        villager_xp: i32,
        show_progress: bool,
        can_restock: bool,
    },
    MoveEntityPos {
        #[using(VarI32)]
        entity_id: i32,
        xa: i16,
        ya: i16,
        za: i16,
        on_ground: bool,
    },
    MoveEntityPosRot {
        #[using(VarI32)]
        entity_id: i32,
        xa: i16,
        ya: i16,
        za: i16,
        #[using(Angle)]
        yaw: f32,
        #[using(Angle)]
        pitch: f32,
        on_ground: bool,
    },
    MoveEntityRot {
        #[using(VarI32)]
        entity_id: i32,
        #[using(Angle)]
        yaw: f32,
        #[using(Angle)]
        pitch: f32,
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
        #[using(VarI32)]
        container_id: i32,
        #[using(VarI32)]
        type_: i32,
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
        #[using(VarI32)]
        index: i32,
        signature: Option<[u8; 256]>,
        message: String,
        timestamp: i64,
        salt: i64,
        unsigned_content: Option<Json<Component>>,
        chat_type: ChatTypeBound,
    },
    PlayerCombatEnd {
        #[using(VarI32)]
        duration: i32,
        killer_id: i32,
    },
    PlayerCombatEnter,
    PlayerCombatKill {
        #[using(VarI32)]
        player_id: i32,
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
        #[using(VarI32)]
        id: i32,
    },
    Recipe(RecipePacket),
    RemoveEntities {
        entity_ids: Vec<VarI32>,
    },
    RemoveMobEffect {
        #[using(VarI32)]
        entity_id: i32,
        #[using(VarI32)]
        effect: i32,
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
        #[using(VarI32)]
        entity_id: i32,
        #[using(Angle)]
        head_yaw: f32,
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
        #[using(VarI64)]
        lerp_time: i64,
    },
    SetBorderSize {
        size: f64,
    },
    SetBorderWarningDelay {
        #[using(VarI32)]
        warning_delay: i32,
    },
    SetBorderWarningDistance {
        #[using(VarI32)]
        warning_blocks: i32,
    },
    SetCamera {
        #[using(VarI32)]
        camera_id: i32,
    },
    SetCarriedItem {
        slot: i8,
    },
    SetChunkCacheCenter {
        #[using(VarI32)]
        x: i32,
        #[using(VarI32)]
        z: i32,
    },
    SetChunkCacheRadius {
        #[using(VarI32)]
        radius: i32,
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
        #[using(VarI32)]
        id: i32,
        #[using(EntityData)]
        packed_items: HashMap<u8, EntityDataValue>,
    },
    SetEntityLink {
        source_id: i32,
        dest_id: i32,
    },
    SetEntityMotion {
        #[using(VarI32)]
        id: i32,
        xa: i16,
        ya: i16,
        za: i16,
    },
    SetEquipment {
        #[using(VarI32)]
        entity: i32,
        #[using(SetEquipmentPacketSlots)]
        slots: HashMap<EquipmentSlot, Option<ItemStack>>,
    },
    SetExperience {
        experience_progress: f32,
        #[using(VarI32)]
        experience_level: i32,
        #[using(VarI32)]
        total_experience: i32,
    },
    SetHealth {
        health: f32,
        #[using(VarI32)]
        food: i32,
        saturation: f32,
    },
    SetObjective {
        objective_name: String,
        method: SetObjectivePacketMethod,
    },
    SetPassengers {
        #[using(VarI32)]
        vehicle: i32,
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
        #[using(VarI32)]
        simulation_distance: i32,
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
        #[using(VarI32)]
        id: i32,
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
        #[using(VarI32)]
        transaction_id: i32,
        tag: Nbt<serde_value::Value>,
    },
    TakeItemEntity {
        #[using(VarI32)]
        item_id: i32,
        #[using(VarI32)]
        player_id: i32,
        #[using(VarI32)]
        amount: i32,
    },
    TeleportEntity {
        #[using(VarI32)]
        id: i32,
        pos: DVec3,
        #[using(Angle)]
        yaw: f32,
        #[using(Angle)]
        pitch: f32,
        on_ground: bool,
    },
    UpdateAdvancements {
        reset: bool,
        added: Vec<(String, Advancement)>,
        removed: Vec<String>,
        progress: Vec<(String, Vec<(String, Option<i64>)>)>,
    },
    UpdateAttributes {
        #[using(VarI32)]
        entity_id: i32,
        attributes: Vec<(String, f64, Vec<(Uuid, f64, i8)>)>,
    },
    UpdateEnabledFeatures {
        features: Vec<String>,
    },
    UpdateMobEffect {
        #[using(VarI32)]
        entity_id: i32,
        #[using(VarI32)]
        id: i32,
        effect_amplifier: i8,
        #[using(VarI32)]
        effect_duration_ticks: i32,
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

#[derive(Encode, Decode, Clone, Debug)]
#[using(u8)]
pub enum AnimatePacketAction {
    SwingMainHand,
    WakeUp,
    SwingOffHand,
    CriticalHit,
    MagicCriticalHit,
}

#[derive(Encode, Decode, Clone, Debug)]
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
    fn encode(&self, output: &mut impl Write) -> Result<()> {
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

impl Decode<'_> for CommandsPacketEntry {
    fn decode(input: &mut &'_ [u8]) -> Result<Self> {
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

#[derive(Encode, Decode, Clone, Debug)]
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
    fn encode(&self, output: &mut impl Write) -> Result<()> {
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

impl<'a, T: Decode<'a>> Decode<'a> for CommandsPacketArgumentTypeNumber<T> {
    fn decode(input: &mut &'a [u8]) -> Result<Self> {
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

#[derive(Encode, Decode, Clone, Debug)]
pub enum CommandsPacketArgumentTypeString {
    SingleWord,
    QuotablePhrase,
    GreedyPhrase,
}

#[derive(Encode, Decode, Clone, Debug)]
pub enum CustomChatCompletionsPacketAction {
    Add,
    Remove,
    Set,
}

#[derive(Encode, Decode, Clone, Debug)]
#[using(u8)]
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

#[derive(Encode, Decode, Clone, Debug)]
pub struct LevelChunkPacketData {
    pub heightmaps: Nbt<serde_value::Value>,
    pub buffer: Vec<u8>,
    pub block_entities_data: Vec<LevelChunkPacketDataBlockEntity>,
}

#[derive(Encode, Decode, Clone, Debug)]
pub struct LevelChunkPacketDataBlockEntity {
    pub xz: i8,
    pub y: i16,
    #[using(VarI32)]
    pub type_: i32,
    pub data: Nbt<serde_value::Value>,
}

#[derive(Encode, Decode, Clone, Debug)]
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
    fn encode(&self, output: &mut impl Write) -> Result<()> {
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

impl Decode<'_> for PlayerAbilitiesPacket {
    fn decode(input: &mut &'_ [u8]) -> Result<Self> {
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
    fn encode(&self, output: &mut impl Write) -> Result<()> {
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

impl Decode<'_> for PlayerInfoUpdatePacket {
    fn decode(input: &mut &'_ [u8]) -> Result<Self> {
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

#[derive(Encode, Decode, Clone, Debug)]
pub struct PlayerLookAtPacketAtEntity {
    #[using(VarI32)]
    pub entity: i32,
    pub to_anchor: Anchor,
}

#[derive(Clone, Debug)]
pub struct SectionBlocksUpdatePacket {
    pub section_pos: IVec3,
    pub suppress_light_updates: bool,
    pub position_and_states: Vec<SectionBlocksUpdatePacketPositionAndState>,
}

impl Encode for SectionBlocksUpdatePacket {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        match (self.section_pos.x, self.section_pos.y, self.section_pos.z) {
            (-0x200000..=0x1FFFFF, -0x80000..=0x7FFFF, -0x200000..=0x1FFFFF) => {
                ((self.section_pos.x as i64) << 42
                    | ((self.section_pos.z & 0x3FFFFF) as i64) << 20
                    | ((self.section_pos.y & 0xFFFFF) as i64))
                    .encode(output)?
            }
            _ => unimplemented!(),
        }
        self.suppress_light_updates.encode(output)?;
        self.position_and_states.encode(output)?;
        Ok(())
    }
}

impl Decode<'_> for SectionBlocksUpdatePacket {
    fn decode(input: &mut &'_ [u8]) -> Result<Self> {
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
    pub x: u8,
    pub y: u8,
    pub z: u8,
    pub block_state: i64,
}

impl Encode for SectionBlocksUpdatePacketPositionAndState {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
        match (self.block_state, self.x, self.y, self.z) {
            (0x0..=0x1FFFFFFFFFFFF, 0x0..=0xF, 0x0..=0xF, 0x0..=0xF) => VarI64(
                (self.block_state) << 12 | (((self.x as i64) << 8) | (self.z << 4 | self.y) as i64),
            )
            .encode(output),
            _ => unimplemented!(),
        }
    }
}

impl Decode<'_> for SectionBlocksUpdatePacketPositionAndState {
    fn decode(input: &mut &'_ [u8]) -> Result<Self> {
        let value = VarI64::decode(input)?.0;
        Ok(Self {
            x: (value >> 8) as u8 & 0xF,
            y: value as u8 & 0xF,
            z: (value >> 4) as u8 & 0xF,
            block_state: value >> 12,
        })
    }
}

#[derive(Encode, Decode, Clone, Debug)]
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

pub struct SetEquipmentPacketSlots(HashMap<EquipmentSlot, Option<ItemStack>>);

impl Encode for SetEquipmentPacketSlots {
    fn encode(&self, output: &mut impl Write) -> Result<()> {
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

impl Decode<'_> for SetEquipmentPacketSlots {
    fn decode(input: &mut &'_ [u8]) -> Result<Self> {
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

#[derive(Encode, Decode, Clone, Debug)]
pub enum SetObjectivePacketMethod {
    Add {
        display_name: Json<Component>,
        #[using(VarI32)]
        render_type: i32,
    },
    Remove,
    Change {
        display_name: Json<Component>,
        #[using(VarI32)]
        render_type: i32,
    },
}

#[derive(Encode, Decode, Clone, Debug)]
pub enum SetPlayerTeamPacketMethod {
    Add {
        display_name: Json<Component>,
        options: i8,
        nametag_visibility: String,
        collision_rule: String,
        #[using(VarI32)]
        color: i32,
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
        #[using(VarI32)]
        color: i32,
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

#[derive(Encode, Decode, Clone, Debug)]
pub enum SetScorePacketMethod {
    Change {
        objective_name: String,
        #[using(VarI32)]
        score: i32,
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
    fn encode(&self, output: &mut impl Write) -> Result<()> {
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

impl Decode<'_> for StopSoundPacket {
    fn decode(input: &mut &'_ [u8]) -> Result<Self> {
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
