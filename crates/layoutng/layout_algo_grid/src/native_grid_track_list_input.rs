//! Grid-owned construction and mutation of authored track-list input.

use super::{GridTrack, GridTrackListInput, GridTrackRepeater};

impl GridTrackListInput {
    #[inline(always)]
    pub fn from_tracks(tracks: &[GridTrack]) -> Self {
        copy_grid_track_list_input(tracks)
    }

    #[inline(always)]
    pub fn empty(&self) -> bool {
        self.repeaters.is_empty()
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        clear_grid_track_list_input(self)
    }
}

impl From<Vec<GridTrack>> for GridTrackListInput {
    #[inline(always)]
    fn from(tracks: Vec<GridTrack>) -> Self {
        take_grid_track_list_input(tracks)
    }
}

#[inline(never)]
fn copy_grid_track_list_input(tracks: &[GridTrack]) -> GridTrackListInput {
    take_grid_track_list_input(tracks.to_vec())
}

#[inline(never)]
fn take_grid_track_list_input(tracks: Vec<GridTrack>) -> GridTrackListInput {
    let mut result = GridTrackListInput::default();
    if !tracks.is_empty() {
        result.repeaters.push(GridTrackRepeater {
            tracks,
            ..GridTrackRepeater::default()
        });
    }
    result
}

#[inline(never)]
fn clear_grid_track_list_input(input: &mut GridTrackListInput) {
    input.repeaters.clear();
}
