use crate::cs2::CS2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Player {
    controller: u64,
    pub(crate) pawn: u64,
}

impl Player {
    pub fn local_player(cs2: &CS2) -> Option<Self> {
        let controller = cs2.process.read(cs2.offsets.direct.local_player);
        if controller == 0 {
            return None;
        }
        let pawn_handle: i32 = cs2.process.read(controller + cs2.offsets.controller.pawn);
        if pawn_handle == -1 {
            return None;
        }
        Self::get_entity(cs2, pawn_handle).map(|pawn| Self { controller, pawn })
    }

    pub fn get_client_entity(cs2: &CS2, index: u64) -> Option<u64> {
        let bucket_index = index >> 9;
        let index_in_bucket = index & 0x1FF;
        let bucket_ptr: u64 = cs2
            .process
            .read(cs2.offsets.interface.entity + 0x08 * bucket_index);
        if bucket_ptr == 0 {
            return None;
        }
        let entity = cs2
            .process
            .read(bucket_ptr + cs2.offsets.entity_identity.size as u64 * index_in_bucket);
        if entity == 0 {
            return None;
        }
        Some(entity)
    }

    fn get_entity(cs2: &CS2, handle: i32) -> Option<u64> {
        let index = handle as u64 & 0x7FFF;
        let bucket_index = index >> 9;
        let index_in_bucket = index & 0x1FF;
        let bucket_ptr: u64 = cs2
            .process
            .read(cs2.offsets.interface.entity + 8 * bucket_index);
        if bucket_ptr == 0 {
            return None;
        }

        let entity = cs2
            .process
            .read(bucket_ptr + cs2.offsets.entity_identity.size as u64 * index_in_bucket);
        if entity == 0 {
            return None;
        }
        Some(entity)
    }
}
