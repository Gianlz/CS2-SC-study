use std::time::Instant;

use crate::{
    constants::cs2,
    cs2::{CS2, offsets::Offsets, schema::Schema},
};

impl CS2 {
    pub fn find_offsets(&self) -> Option<Offsets> {
        let start = Instant::now();
        let mut offsets = Offsets::default();

        offsets.library.client = self.process.module_base_address(cs2::CLIENT_LIB)?;
        offsets.library.engine = self.process.module_base_address(cs2::ENGINE_LIB)?;
        offsets.library.tier0 = self.process.module_base_address(cs2::TIER0_LIB)?;
        offsets.library.input = self.process.module_base_address(cs2::INPUT_LIB)?;
        offsets.library.sdl = self.process.module_base_address(cs2::SDL_LIB)?;
        offsets.library.schema = self.process.module_base_address(cs2::SCHEMA_LIB)?;

        let Some(resource_offset) = self
            .process
            .get_interface_offset(offsets.library.engine, "GameResourceServiceClientV0")
        else {
            log::warn!("could not get offset for GameResourceServiceClient");
            return None;
        };
        offsets.interface.resource = resource_offset;

        offsets.interface.entity =
            self.process.read::<u64>(offsets.interface.resource + 0x50) + 0x10;

        let Some(cvar_address) = self
            .process
            .get_interface_offset(offsets.library.tier0, "VEngineCvar0")
        else {
            log::warn!("could not get convar interface offset");
            return None;
        };
        offsets.interface.cvar = cvar_address;

        let Some(input_address) = self
            .process
            .get_interface_offset(offsets.library.input, "InputSystemVersion0")
        else {
            log::warn!("could not get input interface offset");
            return None;
        };
        offsets.interface.input = input_address;

        let Some(local_player) = self
            .process
            .scan("48 83 3D ? ? ? ? 00 0F 95 C0 C3", offsets.library.client)
        else {
            log::warn!("could not find local player offset");
            return None;
        };
        offsets.direct.local_player = self.process.get_relative_address(local_player, 0x03, 0x08);

        // Network Game Client
        let network_client_patterns = [
            "48 89 3D ? ? ? ? 48 8D 15 ? ? ? ? 48 8B 05",
            "48 89 3D ? ? ? ? 48 8D 15",
            "48 89 1D ? ? ? ? 49 8B 04 24",
            "4C 8B 0D ? ? ? ? 4C 8B D2",
        ];

        let mut network_client = None;
        for pattern in network_client_patterns {
            if let Some(addr) = self.process.scan(pattern, offsets.library.engine) {
                network_client = Some(addr);
                break;
            }
        }

        if let Some(network_client_addr) = network_client {
             offsets.direct.network_client = self.process.get_relative_address(network_client_addr, 0x03, 0x07);
        } else {
             log::warn!("could not find network client offset via scan");
             
             // Try fetching from cs2-dumper
             let dumper_success = crate::cs2::dumper::update_offsets_from_dumper(&mut offsets);
             
             if !dumper_success {
                 log::warn!("using hardcoded fallback (0x8EB538)");
                 offsets.direct.network_client = offsets.library.engine + 0x8EB538;
             }
        }

        // Network client offsets
        if offsets.network_client.delta_tick == 0 {
            offsets.network_client.delta_tick = 0x158;
        }

        let schema = Schema::new(&self.process, offsets.library.schema)?;
        let client = schema.get_library(cs2::CLIENT_LIB)?;

        offsets.controller.pawn = client.get("CBasePlayerController", "m_hPawn")?;

        offsets.pawn.weapon = client.get("C_CSPlayerPawn", "m_pClippingWeapon")?;
        offsets.pawn.weapon_services = client.get("C_BasePlayerPawn", "m_pWeaponServices")?;

        offsets.weapon_services.weapons = client.get("CPlayer_WeaponServices", "m_hMyWeapons")?;

        offsets.weapon.attribute_manager = client.get("C_EconEntity", "m_AttributeManager")?;
        offsets.weapon.item = client.get("C_AttributeContainer", "m_Item")?;
        offsets.weapon.item_definition_index =
            client.get("C_EconItemView", "m_iItemDefinitionIndex")?;

        offsets.entity_identity.size = client.get_class("CEntityIdentity")?.size();

        // Skin changer offsets from CEconItemView
        offsets.skin.item_id_high =
            client.get("C_EconItemView", "m_iItemIDHigh").unwrap_or(0);
        offsets.skin.item_id_low =
            client.get("C_EconItemView", "m_iItemIDLow").unwrap_or(0);
        offsets.skin.account_id =
            client.get("C_EconItemView", "m_iAccountID").unwrap_or(0);
        offsets.skin.entity_quality =
            client.get("C_EconItemView", "m_iEntityQuality").unwrap_or(0);
        offsets.skin.initialized =
            client.get("C_EconItemView", "m_bInitialized").unwrap_or(0);
        offsets.skin.attribute_list =
            client.get("C_EconItemView", "m_AttributeList").unwrap_or(0);
        offsets.skin.networked_dynamic_attrs =
            client.get("C_EconItemView", "m_NetworkedDynamicAttributes").unwrap_or(0);
        
        // Fallback fields for client-side skin changing (C_EconEntity)
        offsets.skin.fallback_paint_kit =
            client.get("C_EconEntity", "m_nFallbackPaintKit").unwrap_or(0);
        offsets.skin.fallback_seed =
            client.get("C_EconEntity", "m_nFallbackSeed").unwrap_or(0);
        offsets.skin.fallback_wear =
            client.get("C_EconEntity", "m_flFallbackWear").unwrap_or(0);
        offsets.skin.fallback_stattrak =
            client.get("C_EconEntity", "m_nFallbackStatTrak").unwrap_or(0);
        offsets.skin.custom_name =
            client.get("C_EconItemView", "m_szCustomName").unwrap_or(0);
        offsets.skin.original_owner_xuid_low =
            client.get("C_EconEntity", "m_OriginalOwnerXuidLow").unwrap_or(0);
        offsets.skin.original_owner_xuid_high =
            client.get("C_EconEntity", "m_OriginalOwnerXuidHigh").unwrap_or(0);

        log::info!(
            "Skin offsets: item_id_high=0x{:X}, fallback_paint_kit=0x{:X}, fallback_seed=0x{:X}, fallback_wear=0x{:X}, fallback_stattrak=0x{:X}",
            offsets.skin.item_id_high,
            offsets.skin.fallback_paint_kit,
            offsets.skin.fallback_seed,
            offsets.skin.fallback_wear,
            offsets.skin.fallback_stattrak
        );

        log::debug!("offsets: {:?} ({:?})", offsets, Instant::now() - start);
        Some(offsets)
    }
}
