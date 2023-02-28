use glam::IVec3;
use uuid::Uuid;

use crate::{
    types::{
        Anchor, BossEventColor, BossEventOverlay, ChatType, GameType, Hand, ItemStack,
        MapDecoration, MapPatch, MerchantOffer, SoundSource, TrailingBytes, VarInt,
    },
    Decode, Encode,
};

#[derive(Debug, Encode, Decode)]
pub enum GamePacket {
    AddEntity {
        id: VarInt,
        uuid: Uuid,
        type_: VarInt,
        x: f64,
        y: f64,
        z: f64,
        x_rot: i8,
        y_rot: i8,
        y_head_rot: i8,
        data: VarInt,
        xa: i16,
        ya: i16,
        za: i16,
    },
    AddExperienceOrb {
        id: VarInt,
        x: f64,
        y: f64,
        z: f64,
        value: i16,
    },
    AddPlayer {
        entity_id: VarInt,
        player_id: Uuid,
        x: f64,
        y: f64,
        z: f64,
        y_rot: i8,
        x_rot: i8,
    },
    Animate {
        id: VarInt,
        action: u8,
    },
    AwardStats,
    BlockChangedAck {
        sequence: VarInt,
    },
    BlockDestruction {
        id: VarInt,
        pos: IVec3,
        progress: u8,
    },
    BlockEntityData {
        pos: IVec3,
        type_: VarInt,
        tag: (),
    },
    BlockEvent {
        pos: IVec3,
        b0: u8,
        b1: u8,
        block: VarInt,
    },
    BlockUpdate {
        pos: IVec3,
        block_state: VarInt,
    },
    BossEvent {
        id: Uuid,
        operation: BossEventPacketOperation,
    },
    ChangeDifficulty {
        difficulty: u8,
        locked: bool,
    },
    ClearTitles {
        reset_times: bool,
    },
    CommandSuggestions {
        id: VarInt,
        suggestions_start: VarInt,
        suggestions_length: VarInt,
        suggestions: Vec<(String, Option<String>)>,
    },
    Commands,
    ContainerClose {
        container_id: u8,
    },
    ContainerSetContent {
        container_id: u8,
        state_id: VarInt,
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
        state_id: VarInt,
        slot: i16,
        item_stack: Option<ItemStack>,
    },
    Cooldown {
        item: VarInt,
        duration: VarInt,
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
        x: f64,
        y: f64,
        z: f64,
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
        size: VarInt,
        entity_id: i32,
    },
    InitializeBorder {
        new_center_x: f64,
        new_center_z: f64,
        old_size: f64,
        new_size: f64,
        lerp_time: VarInt,
        new_absolute_max_size: VarInt,
        warning_blocks: VarInt,
        warning_time: VarInt,
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
        particle_type: VarInt,
        override_limiter: bool,
        x: f64,
        y: f64,
        z: f64,
        x_dist: f32,
        y_dist: f32,
        z_dist: f32,
        max_speed: f32,
        count: i32,
        particle: (),
    },
    LightUpdate {
        x: VarInt,
        z: VarInt,
        light_data: LightUpdatePacketData,
    },
    Login {
        player_id: i32,
        hardcore: bool,
        game_type: GameType,
        previous_game_type: i8,
        levels: Vec<String>,
        registry_holder: (),
        dimension_type: String,
        dimension: String,
        seed: i64,
        max_players: VarInt,
        chunk_radius: VarInt,
        simulation_distance: VarInt,
        reduced_debug_info: bool,
        show_death_screen: bool,
        is_debug: bool,
        is_flat: bool,
        last_death_location: Option<(String, IVec3)>,
    },
    MapItemData {
        map_id: VarInt,
        scale: i8,
        locked: bool,
        decorations: Option<Vec<MapDecoration>>,
        color_patch: Option<MapPatch>,
    },
    MerchantOffers {
        container_id: VarInt,
        offers: Vec<MerchantOffer>,
        villager_level: VarInt,
        villager_xp: VarInt,
        show_progress: bool,
        can_restock: bool,
    },
    MoveEntityPos {
        entity_id: VarInt,
        xa: i16,
        ya: i16,
        za: i16,
        on_ground: bool,
    },
    MoveEntityPosRot {
        entity_id: VarInt,
        xa: i16,
        ya: i16,
        za: i16,
        y_rot: i8,
        x_rot: i8,
        on_ground: bool,
    },
    MoveEntityRot {
        entity_id: VarInt,
        y_rot: i8,
        x_rot: i8,
        on_ground: bool,
    },
    MoveVehicle {
        x: f64,
        y: f64,
        z: f64,
        y_rot: f32,
        x_rot: f32,
    },
    OpenBook {
        hand: Hand,
    },
    OpenScreen {
        container_id: VarInt,
        type_: VarInt,
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
        index: VarInt,
        signature: Option<[u8; 256]>,
        message: String,
        timestamp: i64,
        salt: i64,
        unsigned_content: Option<String>,
        // filter mask
        chat_type: ChatType,
    },
    PlayerCombatEnd {
        duration: VarInt,
        killer_id: i32,
    },
    PlayerCombatEnter,
    PlayerCombatKill {
        player_id: VarInt,
        killer_id: i32,
        message: String,
    },
    PlayerInfoRemove {
        profile_ids: Vec<Uuid>,
    },
    PlayerInfoUpdate,
    PlayerLookAt {
        from_anchor: Anchor,
        x: f64,
        y: f64,
        z: f64,
        at_entity: Option<PlayerLookAtPacketAtEntity>,
    },
    PlayerPosition {
        x: f64,
        y: f64,
        z: f64,
        y_rot: f32,
        x_rot: f32,
        relative_arguments: u8,
        id: VarInt,
        dismount_vehicle: bool,
    },
    Recipe,
    RemoveEntities {
        entity_ids: Vec<VarInt>,
    },
    RemoveMobEffect {
        entity_id: VarInt,
        effect: VarInt,
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
        entity_id: VarInt,
        y_head_rot: i8,
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
        lerp_time: VarInt,
    },
    SetBorderSize {
        size: f64,
    },
    SetBorderWarningDelay {
        warning_delay: VarInt,
    },
    SetBorderWarningDistance {
        warrning_blocks: VarInt,
    },
    SetCamera {
        camera_id: VarInt,
    },
    SetCarriedItem {
        slot: i8,
    },
    SetChunkCacheCenter {
        x: VarInt,
        z: VarInt,
    },
    SetChunkCacheRadius {
        radius: VarInt,
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
        id: VarInt,
        //
    },
    SetEntityLink {
        source_id: i32,
        dest_id: i32,
    },
    SetEntityMotion {
        id: VarInt,
        xa: i16,
        ya: i16,
        za: i16,
    },
    SetEquipment,
    SetExperience {
        experience_progress: f32,
        experience_level: VarInt,
        total_experience: VarInt,
    },
    SetHealth {
        health: f32,
        food: VarInt,
        saturation: f32,
    },
    SetObjective {
        objective_name: String,
        method: SetObjectivePacketMethod,
    },
    SetPassengers {
        vehicle: VarInt,
        passengers: Vec<VarInt>,
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
        simulation_distance: VarInt,
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
        sound: VarInt,
        source: SoundSource,
        id: VarInt,
        volume: f32,
        pitch: f32,
        seed: i64,
    },
    Sound {
        sound: VarInt,
        source: SoundSource,
        x: i32,
        y: i32,
        z: i32,
        volume: f32,
        pitch: f32,
        seed: i64,
    },
    StopSound,
    SystemChat {
        content: String,
        overlay: bool,
    },
    TabList {
        header: String,
        footer: String,
    },
    TagQuery {
        transaction_id: VarInt,
        tag: (),
    },
    TakeItemEntity {
        item_id: VarInt,
        player_id: VarInt,
        amount: VarInt,
    },
    TeleportEntity {
        id: VarInt,
        x: f64,
        y: f64,
        z: f64,
        y_rot: i8,
        x_rot: i8,
        on_ground: bool,
    },
    UpdateAdvancements {
        reset: bool,
        added: Vec<()>,
        removed: Vec<String>,
        progress: Vec<()>,
    },
    UpdateAttributes {
        entity_id: VarInt,
        attributes: (),
    },
    UpdateEnabledFeatures {
        features: Vec<String>,
    },
    UpdateMobEffect {
        entity_id: VarInt,
        id: VarInt,
        effect_amplifier: i8,
        effect_duration_ticks: VarInt,
        flags: u8,
        factor_data: (),
    },
    UpdateRecipes {
        recipes: Vec<()>,
    },
    UpdateTags {
        tags: Vec<()>,
    },
}

#[derive(Debug, Encode, Decode)]
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

#[derive(Debug, Encode, Decode)]
pub enum CustomChatCompletionsPacketAction {
    Add,
    Remove,
    Set,
}

#[derive(Debug, Encode, Decode)]
pub struct LevelChunkPacketData {
    heightmaps: (),
    buffer: Vec<u8>,
    block_entities_data: Vec<()>,
}

#[derive(Debug, Encode, Decode)]
pub struct LightUpdatePacketData {
    trust_edges: bool,
    sky_y_mask: Vec<i64>,
    block_y_mask: Vec<i64>,
    empty_sky_y_mask: Vec<i64>,
    empty_block_y_mask: Vec<i64>,
    sky_updates: Vec<Vec<u8>>,
    block_updates: Vec<Vec<u8>>,
}

#[derive(Debug, Encode, Decode)]
pub struct PlayerLookAtPacketAtEntity {
    entity: VarInt,
    to_anchor: Anchor,
}

#[derive(Debug, Encode, Decode)]
pub enum SetObjectivePacketMethod {
    Add {
        display_name: String,
        render_type: VarInt,
    },
    Remove,
    Change {
        display_name: String,
        render_type: VarInt,
    },
}

#[derive(Debug, Encode, Decode)]
pub enum SetPlayerTeamPacketMethod {
    Add {
        display_name: String,
        options: i8,
        nametag_visibility: String,
        collision_rule: String,
        color: VarInt,
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
        color: VarInt,
        prefix: String,
        suffix: String,
        players: Vec<String>,
    },
    Join {
        players: Vec<String>,
    },
    Leave {
        players: Vec<String>,
    },
}

#[derive(Debug, Encode, Decode)]
pub enum SetScorePacketMethod {
    Change {
        objective_name: String,
        score: VarInt,
    },
    Remove {
        objective_name: String,
    },
}
