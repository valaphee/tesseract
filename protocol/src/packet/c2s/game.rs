use glam::IVec3;
use uuid::Uuid;

use crate::{
    types::{
        ChatSession, ChatVisibility, ClickType, Difficulty, Direction, Hand, ItemStack,
        LastSeenMessages, MainHand, RecipeBookType, TrailingBytes, VarInt32, VarInt64,
    },
    Decode, Encode,
};

#[derive(Clone, Debug, Encode, Decode)]
pub enum GamePacket {
    AcceptTeleportation {
        id: VarInt32,
    },
    BlockEntityTagQuery {
        transaction_id: VarInt32,
        pos: IVec3,
    },
    ChangeDifficulty {
        difficulty: Difficulty,
    },
    ChatAck {
        offset: VarInt32,
    },
    ChatCommand {
        command: String,
        timestamp: i64,
        salt: i64,
        argument_signatures: Vec<(String, [u8; 256])>,
        last_seen_messages: LastSeenMessages,
    },
    Chat {
        message: String,
        timestamp: i64,
        salt: i64,
        signature: Option<[u8; 256]>,
        last_seen_messages: LastSeenMessages,
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
        id: VarInt32,
        command: String,
    },
    ContainerButtonClick {
        container_id: i8,
        button_id: i8,
    },
    ContainerClick {
        container_id: i8,
        state_id: VarInt32,
        slot_num: i16,
        button_num: i8,
        click_type: ClickType,
        changed_slots: Vec<(i16, Option<ItemStack>)>,
        carried_item: Option<ItemStack>,
    },
    ContainerClose {
        container_id: i8,
    },
    CustomPayload {
        identifier: String,
        data: TrailingBytes,
    },
    EditBook {
        slot: VarInt32,
        pages: Vec<String>,
        title: Option<String>,
    },
    EntityTagQuery {
        transaction_id: VarInt32,
        entity_id: VarInt32,
    },
    Interact {
        entity_id: VarInt32,
        action: InteractPacketAction,
        using_secondary_action: bool,
    },
    JigsawGenerate {
        pos: IVec3,
        levels: VarInt32,
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
        yaw: f32,
        pitch: f32,
        on_ground: bool,
    },
    MovePlayerRot {
        yaw: f32,
        pitch: f32,
        on_ground: bool,
    },
    MovePlayerStatusOnly {
        on_ground: bool,
    },
    MoveVehicle {
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
    },
    PaddleBoat {
        left: bool,
        right: bool,
    },
    PickItem {
        slot: VarInt32,
    },
    PlaceRecipe {
        container_id: i8,
        recipe: String,
        shift_down: bool,
    },
    PlayerAbilities {
        flags: i8,
    },
    PlayerAction {
        action: PlayerActionPacketAction,
        pos: IVec3,
        direction: Direction,
        sequence: VarInt32,
    },
    PlayerCommand {
        id: VarInt32,
        action: PlayerCommandPacketAction,
        data: VarInt32,
    },
    PlayerInput {
        xxa: f32,
        zza: f32,
        flags: i8,
    },
    Pong {
        id: i32,
    },
    ChatSessionUpdate(ChatSession),
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
    ResourcePack(ResourcePackPacket),
    SeenAdvancements(SeenAdvancementsPacket),
    SelectTrade {
        item: VarInt32,
    },
    SetBeacon {
        primary: Option<VarInt32>,
        secondary: Option<VarInt32>,
    },
    SetCarriedItem {
        slot: i16,
    },
    SetCommandBlock {
        pos: IVec3,
        command: String,
        mode: VarInt32,
        flags: i8,
    },
    SetCommandMinecart {
        entity: VarInt32,
        command: String,
        track_output: bool,
    },
    SetCreativeModeSlot {
        slot_num: i16,
        item_stack: Option<ItemStack>,
    },
    SetJigsawBlock {
        pos: IVec3,
        name: String,
        target: String,
        pool: String,
        final_state: String,
        joint: String,
    },
    SetStructureBlock {
        pos: IVec3,
        update_type: VarInt32,
        mode: VarInt32,
        offset_x: i8,
        offset_y: i8,
        offset_z: i8,
        size_x: i8,
        size_y: i8,
        size_z: i8,
        mirror: VarInt32,
        rotation: VarInt32,
        data: String,
        integrity: f32,
        seed: VarInt64,
        flags: i8,
    },
    SignUpdate {
        pos: IVec3,
        lines: [String; 4],
    },
    SwingPacket {
        hand: Hand,
    },
    TeleportToEntity {
        uuid: Uuid,
    },
    UseItemOn {
        hand: Hand,
        block_pos: IVec3,
        direction: VarInt32,
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
        inside: bool,
        sequence: VarInt32,
    },
    UseItem {
        hand: Hand,
        sequence: VarInt32,
    },
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum ClientCommandPacketAction {
    PerformRespawn,
    RequestStats,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum InteractPacketAction {
    Interact { hand: Hand },
    Attack,
    InteractAt { x: f32, y: f32, z: f32, hand: Hand },
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum PlayerActionPacketAction {
    StartDestroyBlock,
    AbortDestroyBlock,
    StopDestroyBlock,
    DropAllItems,
    DropItem,
    ReleaseUseItem,
    SwapItemWithOffhand,
}

#[derive(Clone, Debug, Encode, Decode)]
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

#[derive(Clone, Debug, Encode, Decode)]
pub enum ResourcePackPacket {
    SuccessfullyLoaded,
    Declined,
    FailedDownload,
    Accepted,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum SeenAdvancementsPacket {
    OpenedTab { tab: String },
    ClosedScreen,
}
