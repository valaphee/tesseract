use std::{collections::HashMap, io::Write};

use glam::{DVec3, IVec3};
use uuid::Uuid;

use crate::{
    types::{
        Advancement, Anchor, Angle, BossEventColor, BossEventOverlay, ChatType, Difficulty,
        EntityData, EquipmentSlot, GameType, Hand, ItemStack, MapDecoration, MapPatch,
        MerchantOffer, Nbt, Registries, Sound, SoundSource, TrailingBytes, VarI32, VarI64,
    },
    Decode, Encode,
};

#[derive(Clone, Debug, Encode, Decode)]
pub enum GamePacket {
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
        action: u8,
    },
    AwardStats,
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
    ClearTitles {
        reset_times: bool,
    },
    CommandSuggestions {
        id: VarI32,
        suggestions_start: VarI32,
        suggestions_length: VarI32,
        suggestions: Vec<(String, Option<String>)>,
    },
    Commands,
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
        data: TrailingBytes,
    },
    DeleteChat {
        message_signature: Vec<u8>,
    },
    Disconnect {
        reason: String,
    },
    DisguisedChatPacket {
        message: String,
        chat_type: ChatType,
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
        event: u8,
        param: f32,
    },
    HorseScreenOpen {
        container_id: u8,
        size: VarI32,
        entity_id: i32,
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
        y: i32,
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
        title: String,
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
    PlayerAbilities {
        flags: i8,
        flying_speed: f32,
        walking_speed: f32,
    },
    PlayerChat {
        sender: Uuid,
        index: VarI32,
        signature: Option<[u8; 256]>,
        message: String,
        timestamp: i64,
        salt: i64,
        unsigned_content: Option<String>,
        chat_type: ChatType,
    },
    PlayerCombatEnd {
        duration: VarI32,
        killer_id: i32,
    },
    PlayerCombatEnter,
    PlayerCombatKill {
        player_id: VarI32,
        killer_id: i32,
        message: String,
    },
    PlayerInfoRemove {
        profile_ids: Vec<Uuid>,
    },
    PlayerInfoUpdate,
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
        dismount_vehicle: bool,
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
        prompt: Option<String>,
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
    SectionBlocksUpdate,
    SelectAdvancementsTab {
        tab: Option<String>,
    },
    ServerData {
        motd: Option<String>,
        icon_base64: Option<String>,
        previews_chat: bool,
    },
    SetActionBarText {
        text: String,
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
        warrning_blocks: VarI32,
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
        angle: f32,
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
        text: String,
    },
    SetTime {
        game_time: i64,
        day_time: i64,
    },
    SetTitleText {
        text: String,
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
        content: String,
        overlay: bool,
    },
    TabList {
        header: String,
        footer: String,
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
        attributes: (),
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
        recipes: Vec<()>,
    },
    UpdateTags {
        tags: Vec<()>,
    },
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum BossEventPacketOperation {
    Add {
        name: String,
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
        name: String,
    },
    UpdateStyle {
        color: BossEventColor,
        overlay: BossEventOverlay,
    },
    UpdateProperties {
        properties: u8,
    },
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum CustomChatCompletionsPacketAction {
    Add,
    Remove,
    Set,
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

#[derive(Clone, Debug, Encode, Decode)]
pub struct PlayerLookAtPacketAtEntity {
    pub entity: VarI32,
    pub to_anchor: Anchor,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum RecipePacket {
    Init {
        recipes: Vec<String>,
        to_highlight: Vec<String>,
    },
    Add {
        recipes: Vec<String>,
    },
    Remove {
        recipes: Vec<String>,
    },
}

#[derive(Clone, Debug)]
pub struct SetEquipmentPacketSlots(HashMap<EquipmentSlot, ItemStack>);

impl Encode for SetEquipmentPacketSlots {
    fn encode<W: Write>(&self, output: &mut W) -> crate::Result<()> {
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
    fn decode(input: &mut &[u8]) -> crate::Result<Self> {
        let mut slots = HashMap::new();
        /*loop {
            let equipment_slot_and_next_bit = u8::decode(input)?;
            slots.insert(
                EquipmentSlot::try_from(equipment_slot_and_next_bit & 0x7F).unwrap(),
                Decode::decode(input)?,
            );
            if equipment_slot_and_next_bit & 0x80 == 0 {
                break;
            }
        }*/
        Ok(Self(slots))
    }
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum SetObjectivePacketMethod {
    Add {
        display_name: String,
        render_type: VarI32,
    },
    Remove,
    Change {
        display_name: String,
        render_type: VarI32,
    },
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum SetPlayerTeamPacketMethod {
    Add {
        display_name: String,
        options: i8,
        nametag_visibility: String,
        collision_rule: String,
        color: VarI32,
        prefix: String,
        suffix: String,
        players: Vec<String>,
    },
    Remove,
    Change {
        display_name: String,
        options: i8,
        nametag_visibility: String,
        collision_rule: String,
        color: VarI32,
        prefix: String,
        suffix: String,
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
    fn encode<W: Write>(&self, output: &mut W) -> crate::Result<()> {
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
    fn decode(input: &mut &[u8]) -> crate::Result<Self> {
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
