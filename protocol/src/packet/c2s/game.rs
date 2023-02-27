use uuid::Uuid;

use crate::{
    types::{ChatVisibility, ClickType, Difficulty, Hand, MainHand, RecipeBookType, VarInt},
    Decode, Encode,
};

#[derive(Encode, Decode)]
pub enum GamePacket {
    AcceptTeleportation {
        id: VarInt,
    },
    BlockEntityTagQuery {
        transaction_id: VarInt,
        pos: i64,
    },
    ChangeDifficulty {
        difficulty: Difficulty,
    },
    ChatCommand {
        command: String,
        timestamp: i64,
        salt: i64,
        /*signatures: HashMap<String, Vec<u8>>,*/
        signed_preview: bool,
    },
    Chat {
        message: String,
        timestamp: i64,
        salt: i64,
        signatures: Vec<u8>,
        signed_preview: bool,
    },
    ChatPreview {
        query_id: i32,
        query: String,
    },
    ClientCommand {
        action: ClientCommandPacketAction,
    },
    ClientInformation {
        language: String,
        view_distance: i8,
        chat_visibility: ChatVisibility,
        chat_colors: bool,
        model_customisation: u8,
        main_hand: MainHand,
        text_filtering_enabled: bool,
        allow_listing: bool,
    },
    CommandSuggestion {
        id: VarInt,
        command: String,
    },
    ContainerButtonClick {
        container_id: i8,
        button_id: i8,
    },
    ContainerClick {
        container_id: i8,
        state_id: VarInt,
        slot_num: i16,
        button_num: i8,
        click_type: ClickType,
        // changed_slots, carried_item
    },
    ContainerClose {
        container_id: i8,
    },
    CustomPayload {
        identifier: String,
        // data
    },
    EditBook {
        slot: VarInt,
        pages: Vec<String>,
        title: Option<String>,
    },
    EntityTagQuery {
        transaction_id: VarInt,
        entity_id: VarInt,
    },
    Interact {
        entity_id: VarInt,
        action: InteractPacketAction,
        using_secondary_action: bool,
    },
    JigsawGenerate {
        pos: i64,
        levels: VarInt,
        keep_jigsaws: bool,
    },
    KeepAlive {
        id: i64,
    },
    LockDifficulty {
        locked: bool,
    },
    MovePlayerPos {
        x: f64,
        y: f64,
        z: f64,
        on_ground: bool,
    },
    MovePlayerPosRot {
        x: f64,
        y: f64,
        z: f64,
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
    },
    MovePlayerRot {
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
    },
    MovePlayerStatusOnly {
        on_ground: bool,
    },
    MoveVehicle {
        x: f64,
        y: f64,
        z: f64,
        y_rot: f32,
        x_rot: f32,
    },
    PaddleBoat {
        left: bool,
        right: bool,
    },
    PickItem {
        slot: VarInt,
    },
    PlaceRecipe {
        container_id: i8,
        recipe: String,
        shift_down: bool,
    },
    PlayerAbilities,
    PlayerAction {
        action: PlayerActionPacketAction,
        pos: i64,
        direction: u8,
        sequence: VarInt,
    },
    PlayerCommand {
        id: VarInt,
        action: PlayerCommandPacketAction,
        data: VarInt,
    },
    PlayerInput {
        xxa: f32,
        zza: f32,
        flags: i8,
    },
    Pong {
        id: i32,
    },
    RecipeBookChangeSettings {
        book_type: RecipeBookType,
        is_open: bool,
        is_filtering: bool,
    },
    RecipeBookSeenRecipe {
        recipe: String,
    },
    RenameItem {
        name: String,
    },
    ResourcePack {
        action: ResourcePackPacketAction,
    },
    SeenAdvancements,
    SelectTrade {
        item: VarInt,
    },
    SetBeacon {
        primary: Option<VarInt>,
        secondary: Option<VarInt>,
    },
    SetCarriedItem {
        slot: i16,
    },
    SetCommandBlock {
        pos: i64,
        command: String,
        mode: VarInt,
        flags: i8,
    },
    SetCommandMinecart {
        entity: VarInt,
        command: String,
        track_output: bool,
    },
    SetCreativeModeSlot {
        slot_num: i16,
        // item_stack
    },
    SetJigsawBlock {
        pos: i64,
        name: String,
        target: String,
        pool: String,
        final_state: String,
        joint: String,
    },
    SetStructureBlock {
        pos: i64,
        update_type: VarInt,
        mode: VarInt,
        offset_x: i8,
        offset_y: i8,
        offset_z: i8,
        size_x: i8,
        size_y: i8,
        size_z: i8,
        mirror: VarInt,
        rotation: VarInt,
        data: String,
        integrity: f32,
        seed: VarInt,
        flags: i8,
    },
    SignUpdate {
        pos: i64,
        /*lines: [String; 4],*/
    },
    SwingPacket {
        hand: Hand,
    },
    TeleportToEntity {
        uuid: Uuid,
    },
    UseItemOn {
        hand: Hand,
        block_pos: i64,
        direction: VarInt,
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
        inside: bool,
        sequence: VarInt,
    },
    UseItem {
        hand: Hand,
        sequence: VarInt,
    },
}

#[derive(Encode, Decode)]
pub enum ClientCommandPacketAction {
    PerformRespawn,
    RequestStats,
}

#[derive(Encode, Decode)]
pub enum InteractPacketAction {
    Interact { hand: Hand },
    Attack,
    InteractAt { x: f32, y: f32, z: f32, hand: Hand },
}

#[derive(Encode, Decode)]
pub enum PlayerActionPacketAction {
    StartDestroyBlock,
    AbortDestroyBlock,
    StopDestroyBlock,
    DropAllItems,
    DropItem,
    ReleaseUseItem,
    SwapItemWithOffhand,
}

#[derive(Encode, Decode)]
pub enum PlayerCommandPacketAction {
    PressShiftKey,
    ReleaseShiftKey,
    StopSleeping,
    StartSprinting,
    StopSprinting,
    StartRidingJump,
    StopRidingJump,
    OpenInventory,
    StartFallFlying,
}

#[derive(Encode, Decode)]
pub enum ResourcePackPacketAction {
    SuccessfullyLoaded,
    Declined,
    FailedDownload,
    Accepted,
}
