#[derive(Debug, Default)]
pub struct LibraryOffsets {
    pub client: u64,
    pub engine: u64,
    pub tier0: u64,
    pub input: u64,
    pub sdl: u64,
    pub schema: u64,
}

#[derive(Debug, Default)]
pub struct InterfaceOffsets {
    pub resource: u64,
    pub entity: u64,
    pub cvar: u64,
    pub input: u64,
}

#[derive(Debug, Default)]
pub struct DirectOffsets {
    pub local_player: u64,
    pub network_client: u64,
}

#[derive(Debug, Default)]
pub struct PlayerControllerOffsets {
    pub pawn: u64,
}

#[derive(Debug, Default)]
pub struct PawnOffsets {
    pub weapon: u64,
    pub weapon_services: u64,
}

#[derive(Debug, Default)]
pub struct WeaponServicesOffsets {
    pub weapons: u64,
}

#[derive(Debug, Default)]
pub struct WeaponOffsets {
    pub attribute_manager: u64,
    pub item: u64,
    pub item_definition_index: u64,
}

#[derive(Debug, Default)]
pub struct EntityIdentityOffsets {
    pub size: i32,
}

#[derive(Debug, Default)]
pub struct SkinOffsets {
    pub item_id_high: u64,
    pub item_id_low: u64,
    pub account_id: u64,
    pub entity_quality: u64,
    pub initialized: u64,
    pub attribute_list: u64,
    pub networked_dynamic_attrs: u64,
    // Fallback fields - these are the key for client-side skin changing
    pub fallback_paint_kit: u64,
    pub fallback_seed: u64,
    pub fallback_wear: u64,
    pub fallback_stattrak: u64,
    pub custom_name: u64,
    pub original_owner_xuid_low: u64,
    pub original_owner_xuid_high: u64,
}

#[derive(Debug, Default)]
pub struct NetworkGameClientOffsets {
    pub delta_tick: u64,
}

#[derive(Debug, Default)]
pub struct Offsets {
    pub library: LibraryOffsets,
    pub interface: InterfaceOffsets,
    pub direct: DirectOffsets,
    pub controller: PlayerControllerOffsets,
    pub pawn: PawnOffsets,
    pub weapon_services: WeaponServicesOffsets,
    pub weapon: WeaponOffsets,
    pub entity_identity: EntityIdentityOffsets,
    pub skin: SkinOffsets,
    pub network_client: NetworkGameClientOffsets,
}
