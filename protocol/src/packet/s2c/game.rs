use uuid::Uuid;

use crate::{
    types::{
        Anchor, BossEventColor, BossEventOverlay, GameType, Hand, MapDecoration, MerchantOffer,
        SoundSource, VarInt,
    },
    Decode, Encode,
};

#[derive(Encode, Decode)]
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
        pos: i64,
        progress: u8,
    },
    BlockEntityData {
        pos: i64,
        type_: VarInt,
    },
    BlockEvent {
        pos: i64,
        b0: u8,
        b1: u8,
        block: VarInt,
    },
    BlockUpdate {
        pos: i64,
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
    ChatPreview {
        query_id: i32,
        preview: Option<String>,
    },
    ClearTitles {
        reset_times: bool,
    },
    CommandSuggestions {
        id: VarInt,
    },
    Commands,
    ContainerClose {
        container_id: u8,
    },
    ContainerSetContent {
        container_id: u8,
        state_id: VarInt,
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
    },
    Cooldown {
        item: VarInt,
        duration: VarInt,
    },
    CustomPayload {
        identifier: String,
    },
    CustomSound {
        name: String,
        source: SoundSource,
        x: i32,
        y: i32,
        z: i32,
        volume: f32,
        pitch: f32,
        seed: i64,
    },
    Disconnect {
        reason: String,
    },
    EntityEvent {
        entity_id: i32,
        event_id: i8,
    },
    Explode {
        x: f32,
        y: f32,
        z: f32,
        power: f32,
        // to_blow
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
    },
    LevelEvent {
        type_: i32,
        pos: i64,
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
    },
    LightUpdate {
        x: VarInt,
        z: VarInt,
    },
    Login {
        player_id: i32,
        hardcore: bool,
        game_type: GameType,
        previous_game_type: Option<GameType>,
        levels: Vec<String>,
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
    },
    MapItemData {
        map_id: VarInt,
        scale: i8,
        locked: bool,
        decorations: Option<Vec<MapDecoration>>,
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
        pos: i64,
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
        signed_content: String,
        unsigned_content: Option<String>,
        type_id: VarInt,
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
    PlayerInfo,
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
        previous_player_game_type: Option<GameType>,
        is_debug: bool,
        is_flat: bool,
        keep_all_player_data: bool,
        /*last_death_location: Option<>*/
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
        pos: i64,
        angle: f32,
    },
    SetDisplayChatPreview {
        enabled: bool,
    },
    SetDisplayObjective {
        slot: i8,
        objective_name: String,
    },
    SetEntityData {
        id: VarInt,
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
        method: i8,
        display_name: String,
        render_type: VarInt,
    },
    SetPassengers {
        vehicle: VarInt,
        passengers: Vec<VarInt>,
    },
    SetPlayerTeam,
    SetScore {
        owner: String,
        method: VarInt,
        objective_name: String,
        score: VarInt,
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
        type_id: VarInt,
    },
    TabList {
        header: String,
        footer: String,
    },
    TagQuery {
        transaction_id: VarInt,
        // tag
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
    UpdateAdvancements,
    UpdateAttributes,
    UpdateMobEffect {
        entity_id: VarInt,
        id: VarInt,
        effect_amplifier: i8,
        effect_duration_ticks: VarInt,
        flags: u8,
        // factor_data
    },
    UpdateRecipes,
    UpdateTags,
}

#[derive(Encode, Decode)]
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

#[derive(Encode, Decode)]
pub struct PlayerLookAtPacketAtEntity {
    entity: VarInt,
    to_anchor: Anchor,
}
