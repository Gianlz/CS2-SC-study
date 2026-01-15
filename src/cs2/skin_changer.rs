use crate::{
    config::SkinChangerConfig,
    cs2::{entity::weapon::Weapon, CS2},
};

impl CS2 {
    /// Applies skin changes to weapons using the fallback field system.
    ///
    /// CS2 has fallback fields on C_EconEntity that override inventory data:
    /// - m_nFallbackPaintKit: Skin ID
    /// - m_nFallbackSeed: Pattern seed
    /// - m_flFallbackWear: Wear value (0.0 - 1.0)
    /// - m_nFallbackStatTrak: StatTrak kill count (-1 = disabled)
    ///
    /// The key is setting m_iItemIDHigh to -1 on the CEconItemView to force
    /// the game to use fallback values instead of inventory lookup.
    pub fn skin_changer(&mut self, config: &SkinChangerConfig) {
        if !config.enabled {
            return;
        }

        // Validate required offsets
        if self.offsets.skin.item_id_high == 0 || self.offsets.skin.fallback_paint_kit == 0 {
            log::trace!(
                "Skin changer: missing offsets - item_id_high={}, fallback_paint_kit={}",
                self.offsets.skin.item_id_high,
                self.offsets.skin.fallback_paint_kit
            );
            return;
        }

        let Some(local_player) = crate::cs2::entity::player::Player::local_player(self) else {
            return;
        };

        // Get active weapon directly from pawn (this is the weapon entity pointer)
        let active_weapon: u64 = self
            .process
            .read(local_player.pawn + self.offsets.pawn.weapon);

        // Apply to active weapon
        if active_weapon != 0 {
            self.apply_skin_to_weapon(active_weapon, config);
        }

        // Get weapon services to iterate all weapons in inventory
        let weapon_services: u64 = self
            .process
            .read(local_player.pawn + self.offsets.pawn.weapon_services);

        if weapon_services == 0 {
            return;
        }

        // CUtlVector structure for m_hMyWeapons:
        // offset+0x00: count (i32)
        // offset+0x08: data pointer
        let weapons_count: i32 = self
            .process
            .read(weapon_services + self.offsets.weapon_services.weapons);

        let weapons_data: u64 = self
            .process
            .read(weapon_services + self.offsets.weapon_services.weapons + 0x08);

        if weapons_data == 0 || weapons_count <= 0 || weapons_count > 64 {
            return;
        }

        // Iterate through all weapons in inventory
        for i in 0..weapons_count as u64 {
            let weapon_handle: i32 = self.process.read(weapons_data + 0x04 * i);
            let weapon_index = (weapon_handle as u64) & 0xFFF;

            let Some(weapon_entity) =
                crate::cs2::entity::player::Player::get_client_entity(self, weapon_index)
            else {
                continue;
            };

            if weapon_entity == 0 || weapon_entity == active_weapon {
                continue;
            }

            self.apply_skin_to_weapon(weapon_entity, config);
        }

        // Note: We don't call force_full_update here because it can trigger the game
        // to reload weapon data from inventory, which overwrites our fallback values.
        // Instead, we rely on continuous reapplication every frame to keep skins persistent.
        // The game may reset ItemIDHigh, but we'll catch it immediately and reapply.
    }

    #[allow(dead_code)]
    fn force_full_update(&self) {
        let network_client: u64 = self.process.read(self.offsets.direct.network_client);
        if network_client != 0 {
            log::info!("Forcing full update. Client: 0x{:X}, DeltaTickOffset: 0x{:X}", network_client, self.offsets.network_client.delta_tick);
            self.process.write(
                network_client + self.offsets.network_client.delta_tick,
                -1i32,
            );
        } else {
            log::warn!("Cannot force update: network_client is 0");
        }
    }

    fn apply_skin_to_weapon(&self, weapon_entity: u64, config: &SkinChangerConfig) -> bool {
        // Validate weapon entity
        if weapon_entity == 0 {
            return false;
        }

        // Get weapon type from item definition index
        let weapon = Weapon::from_handle(weapon_entity, self);
        if weapon == Weapon::Unknown {
            return false;
        }

        // Check if we have a skin config for this weapon
        let Some(skin_config) = config.skins.get(&weapon) else {
            return false;
        };

        if !skin_config.enabled || skin_config.paint_kit <= 0 {
            return false;
        }

        // Get the CEconItemView address (weapon_entity + m_AttributeManager + m_Item)
        let econ_item_view = weapon_entity
            + self.offsets.weapon.attribute_manager
            + self.offsets.weapon.item;

        // Check current values to determine if we need to apply
        let current_paint_kit: i32 =
            self.process
                .read(weapon_entity + self.offsets.skin.fallback_paint_kit);

        let current_item_id_high: i32 = self.process.read(econ_item_view + self.offsets.skin.item_id_high);

        // Only apply if values are incorrect (optimization to avoid unnecessary writes)
        // But we still check every frame to catch resets immediately
        if current_item_id_high == -1 && current_paint_kit == skin_config.paint_kit {
            return false;
        }

        // Log when values were reset (most common case after death/round change)
        if current_item_id_high != -1 || current_paint_kit != skin_config.paint_kit {
            log::info!("Applying skin: Entity=0x{:X}, Weapon={:?}, PaintKit={}->{}, ItemIdHigh={}->-1", 
                weapon_entity, weapon, current_paint_kit, skin_config.paint_kit, current_item_id_high);
        }

        // Based on external CS2 skin changer implementations:
        // 1. Set ItemIDHigh/Low to -1 FIRST to prevent inventory lookup
        // 2. Then set all fallback values
        // 3. Set additional fields for ownership/quality
        // 4. Verify and re-set ItemIDHigh if needed
        
        // STEP 1: Set ItemIDHigh and ItemIDLow to -1 FIRST
        // This prevents the game from reading inventory data while we set fallback values
        if self.offsets.skin.item_id_low != 0 {
            self.process
                .write(econ_item_view + self.offsets.skin.item_id_low, -1i32);
        }
        
        self.process
            .write(econ_item_view + self.offsets.skin.item_id_high, -1i32);

        // STEP 2: Set all fallback values on C_EconEntity
        // These are the actual skin properties the game will use
        self.process.write(
            weapon_entity + self.offsets.skin.fallback_paint_kit,
            skin_config.paint_kit,
        );

        if self.offsets.skin.fallback_seed != 0 {
            self.process.write(
                weapon_entity + self.offsets.skin.fallback_seed,
                skin_config.seed,
            );
        }

        if self.offsets.skin.fallback_wear != 0 {
            self.process.write(
                weapon_entity + self.offsets.skin.fallback_wear,
                skin_config.wear,
            );
        }

        if self.offsets.skin.fallback_stattrak != 0 {
            self.process.write(
                weapon_entity + self.offsets.skin.fallback_stattrak,
                skin_config.stattrak,
            );
        }

        // STEP 3: Set additional CEconItemView fields for proper skin display
        if self.offsets.skin.account_id != 0 {
            self.process
                .write(econ_item_view + self.offsets.skin.account_id, 1u32);
        }

        // Set entity quality for StatTrak (9 = StatTrak quality)
        if skin_config.stattrak >= 0 && self.offsets.skin.entity_quality != 0 {
            self.process
                .write(econ_item_view + self.offsets.skin.entity_quality, 9i32);
        }

        // Set to normal quality if not StatTrak
        else if self.offsets.skin.entity_quality != 0 {
            self.process
                .write(econ_item_view + self.offsets.skin.entity_quality, 0i32);
        }

        // STEP 4: Set original owner XUID fields on C_EconEntity
        // These help prevent the game from resetting skins by indicating ownership
        if self.offsets.skin.original_owner_xuid_low != 0 {
            self.process
                .write(weapon_entity + self.offsets.skin.original_owner_xuid_low, 1u32);
        }
        if self.offsets.skin.original_owner_xuid_high != 0 {
            self.process
                .write(weapon_entity + self.offsets.skin.original_owner_xuid_high, 0u32);
        }

        // STEP 5: CRITICAL - Re-set ItemIDHigh to -1 after all writes
        // The game might reset it during our writes, so we set it again
        self.process
            .write(econ_item_view + self.offsets.skin.item_id_high, -1i32);

        // STEP 6: Verify ItemIDHigh is still -1 and fix if needed
        // Some implementations check multiple times to ensure persistence
        let verify_item_id_high: i32 = self.process.read(econ_item_view + self.offsets.skin.item_id_high);
        if verify_item_id_high != -1 {
            // Game reset it, try again
            self.process
                .write(econ_item_view + self.offsets.skin.item_id_high, -1i32);
        }

        // STEP 7: Toggle initialized flag to force re-initialization
        // This ensures the game recognizes the changes immediately
        if self.offsets.skin.initialized != 0 {
            self.process
                .write(econ_item_view + self.offsets.skin.initialized, 0u8);
            self.process
                .write(econ_item_view + self.offsets.skin.initialized, 1u8);
        }
        
        true
    }
}
