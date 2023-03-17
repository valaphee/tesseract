use glam::IVec3;
use uuid::Uuid;

use crate::{
    types::{
        ChatSession, ChatVisibility, ClickType, Difficulty, Direction, Hand, ItemStack,
        LastSeenMessages, MainHand, RecipeBookType, TrailingBytes, VarI32, VarI64,
    },
    Decode, Encode,
};

#[derive(Clone, Debug, Encode, Decode)]
pub enum GamePacket {
    AcceptTeleportation {
        id: VarI32,
    },
    BlockEntityTagQuery {
        transaction_id: VarI32,
        pos: IVec3,
    },
    ChangeDifficulty {
        difficulty: Difficulty,
    },
    ChatAck {
        offset: VarI32,
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
    ChatSessionUpdate(ChatSession),
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
        id: VarI32,
        command: String,
    },
    ContainerButtonClick {
        container_id: i8,
        button_id: i8,
    },
    ContainerClick {
        container_id: i8,
        state_id: VarI32,
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
        data: TrailingBytes<{ (1 << 15) - 1 }>,
    },
    EditBook {
        slot: VarI32,
        pages: Vec<String>,
        title: Option<String>,
    },
    EntityTagQuery {
        transaction_id: VarI32,
        entity_id: VarI32,
    },
    Interact {
        entity_id: VarI32,
        action: InteractPacketAction,
        using_secondary_action: bool,
    },
    JigsawGenerate {
        pos: IVec3,
        levels: VarI32,
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
        slot: VarI32,
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
        sequence: VarI32,
    },
    PlayerCommand {
        id: VarI32,
        action: PlayerCommandPacketAction,
        data: VarI32,
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
    ResourcePack(ResourcePackPacket),
    SeenAdvancements(SeenAdvancementsPacket),
    SelectTrade {
        item: VarI32,
    },
    SetBeacon {
        primary: Option<VarI32>,
        secondary: Option<VarI32>,
    },
    SetCarriedItem {
        slot: i16,
    },
    SetCommandBlock {
        pos: IVec3,
        command: String,
        mode: VarI32,
        flags: i8,
    },
    SetCommandMinecart {
        entity: VarI32,
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
        update_type: VarI32,
        mode: VarI32,
        offset_x: i8,
        offset_y: i8,
        offset_z: i8,
        size_x: i8,
        size_y: i8,
        size_z: i8,
        mirror: VarI32,
        rotation: VarI32,
        data: String,
        integrity: f32,
        seed: VarI64,
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
        direction: VarI32,
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
        inside: bool,
        sequence: VarI32,
    },
    UseItem {
        hand: Hand,
        sequence: VarI32,
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
