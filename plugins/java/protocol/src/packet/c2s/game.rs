use glam::IVec3;
use uuid::Uuid;

use crate::{
    types::{
        ChatSession, ChatVisibility, ClickType, Difficulty, Direction, Hand, ItemStack,
        LastSeenMessages, MainHand, RecipeBookType, TrailingBytes, VarI32, VarI64,
    },
    Decode, Encode,
};

#[derive(Encode, Decode, Clone, Debug)]
pub enum GamePacket {
    AcceptTeleportation {
        #[using(VarI32)]
        id: i32,
    },
    BlockEntityTagQuery {
        #[using(VarI32)]
        transaction_id: i32,
        pos: IVec3,
    },
    ChangeDifficulty {
        difficulty: Difficulty,
    },
    ChatAck {
        #[using(VarI32)]
        offset: i32,
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
        #[using(VarI32)]
        id: i32,
        command: String,
    },
    ContainerButtonClick {
        container_id: i8,
        button_id: i8,
    },
    ContainerClick {
        container_id: i8,
        #[using(VarI32)]
        state_id: i32,
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
        #[using(VarI32)]
        slot: i32,
        pages: Vec<String>,
        title: Option<String>,
    },
    EntityTagQuery {
        #[using(VarI32)]
        transaction_id: i32,
        #[using(VarI32)]
        entity_id: i32,
    },
    Interact {
        #[using(VarI32)]
        entity_id: i32,
        action: InteractPacketAction,
        using_secondary_action: bool,
    },
    JigsawGenerate {
        pos: IVec3,
        #[using(VarI32)]
        levels: i32,
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
        #[using(VarI32)]
        slot: i32,
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
        #[using(VarI32)]
        sequence: i32,
    },
    PlayerCommand {
        #[using(VarI32)]
        id: i32,
        action: PlayerCommandPacketAction,
        #[using(VarI32)]
        data: i32,
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
        #[using(VarI32)]
        item: i32,
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
        #[using(VarI32)]
        mode: i32,
        flags: i8,
    },
    SetCommandMinecart {
        #[using(VarI32)]
        entity: i32,
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
        #[using(VarI32)]
        update_type: i32,
        #[using(VarI32)]
        mode: i32,
        offset_x: i8,
        offset_y: i8,
        offset_z: i8,
        size_x: i8,
        size_y: i8,
        size_z: i8,
        #[using(VarI32)]
        mirror: i32,
        #[using(VarI32)]
        rotation: i32,
        data: String,
        integrity: f32,
        #[using(VarI64)]
        seed: i64,
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
        direction: Direction,
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
        inside: bool,
        #[using(VarI32)]
        sequence: i32,
    },
    UseItem {
        hand: Hand,
        #[using(VarI32)]
        sequence: i32,
    },
}

#[derive(Encode, Decode, Clone, Debug)]
pub enum ClientCommandPacketAction {
    PerformRespawn,
    RequestStats,
}

#[derive(Encode, Decode, Clone, Debug)]
pub enum InteractPacketAction {
    Interact { hand: Hand },
    Attack,
    InteractAt { x: f32, y: f32, z: f32, hand: Hand },
}

#[derive(Encode, Decode, Clone, Debug)]
pub enum PlayerActionPacketAction {
    StartDestroyBlock,
    AbortDestroyBlock,
    StopDestroyBlock,
    DropAllItems,
    DropItem,
    ReleaseUseItem,
    SwapItemWithOffhand,
}

#[derive(Encode, Decode, Clone, Debug)]
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

#[derive(Encode, Decode, Clone, Debug)]
pub enum ResourcePackPacket {
    SuccessfullyLoaded,
    Declined,
    FailedDownload,
    Accepted,
}

#[derive(Encode, Decode, Clone, Debug)]
pub enum SeenAdvancementsPacket {
    OpenedTab { tab: String },
    ClosedScreen,
}
