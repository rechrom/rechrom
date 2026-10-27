// Source: cc/tiles/tile_manager.h:60-68
// Source: cc/tiles/tile_manager.cc:354-366

/// Counts produced by raster-task completion handling. Formatting and trace
/// transport are supplied by the compositor facade.
#[derive(Default)]
pub struct RasterTaskCompletionStats {
    pub completed_count: usize,
    pub canceled_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RasterCompletionToken<ManagerId, TileId> {
    pub manager: ManagerId,
    pub tile_id: TileId,
    pub prepare_tiles_id: u64,
    pub trace_task_id: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RasterCompletionTokenError {
    WrongManager,
    FuturePrepareGeneration,
}

pub fn ValidateRasterCompletionToken<ManagerId: PartialEq, TileId>(
    token: &RasterCompletionToken<ManagerId, TileId>,
    manager: &ManagerId,
    current_prepare_tiles_id: u64,
) -> Result<(), RasterCompletionTokenError> {
    if &token.manager != manager {
        return Err(RasterCompletionTokenError::WrongManager);
    }
    if token.prepare_tiles_id > current_prepare_tiles_id {
        return Err(RasterCompletionTokenError::FuturePrepareGeneration);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_completion_from_future_prepare_generation() {
        let token = RasterCompletionToken {
            manager: 7u64,
            tile_id: 9u64,
            prepare_tiles_id: 4,
            trace_task_id: 11,
        };
        assert_eq!(
            ValidateRasterCompletionToken(&token, &7, 3),
            Err(RasterCompletionTokenError::FuturePrepareGeneration)
        );
    }
}
