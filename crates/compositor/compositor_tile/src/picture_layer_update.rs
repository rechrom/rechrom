//! Picture-layer tile update and hidden-resource release walk.

pub trait PictureLayerUpdate {
    fn has_valid_tile_priorities(&self) -> bool;
    fn release_resources(&mut self);
    fn update_tiles(&mut self) -> bool;
}

pub trait PictureLayerUpdateContext {
    fn needs_tile_update(&self) -> bool;
    fn begin_tile_update(&mut self);
    fn release_resources_for_hidden_layers(&self) -> bool;
    fn visit_picture_layers(&mut self, visitor: &mut dyn FnMut(&mut dyn PictureLayerUpdate));
    fn report_layers_updated(&mut self, count: usize);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PictureLayerUpdateStats {
    pub ran: bool,
    pub visited: usize,
    pub updated: usize,
    pub invalid_priorities: usize,
    pub released_resources: usize,
    pub changed: bool,
}

pub fn UpdatePictureLayerTiles<C: PictureLayerUpdateContext>(
    context: &mut C,
) -> PictureLayerUpdateStats {
    let mut trace =
        browser_tracing::span("compositor.tile", "Compositor.Tile.UpdatePictureLayerTiles");
    if !context.needs_tile_update() {
        trace.set("skipped", 1.0);
        return PictureLayerUpdateStats::default();
    }
    context.begin_tile_update();
    let release_hidden = context.release_resources_for_hidden_layers();
    let mut stats = PictureLayerUpdateStats {
        ran: true,
        ..PictureLayerUpdateStats::default()
    };
    context.visit_picture_layers(&mut |layer| {
        stats.visited += 1;
        if !layer.has_valid_tile_priorities() {
            stats.invalid_priorities += 1;
            if release_hidden {
                layer.release_resources();
                stats.released_resources += 1;
            }
            return;
        }
        stats.updated += 1;
        stats.changed |= layer.update_tiles();
    });
    context.report_layers_updated(stats.updated);
    trace.set("visited", stats.visited as f64);
    trace.set("updated", stats.updated as f64);
    trace.set("invalid_priorities", stats.invalid_priorities as f64);
    trace.set("released_resources", stats.released_resources as f64);
    trace.set("changed", stats.changed as u8 as f64);
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Layer {
        valid: bool,
        changed: bool,
        released: bool,
        updated: bool,
    }

    impl PictureLayerUpdate for Layer {
        fn has_valid_tile_priorities(&self) -> bool {
            self.valid
        }
        fn release_resources(&mut self) {
            self.released = true;
        }
        fn update_tiles(&mut self) -> bool {
            self.updated = true;
            self.changed
        }
    }

    struct Context {
        needs: bool,
        release: bool,
        layers: Vec<Layer>,
        reported: Option<usize>,
    }

    impl PictureLayerUpdateContext for Context {
        fn needs_tile_update(&self) -> bool {
            self.needs
        }
        fn begin_tile_update(&mut self) {
            self.needs = false;
        }
        fn release_resources_for_hidden_layers(&self) -> bool {
            self.release
        }
        fn visit_picture_layers(&mut self, visitor: &mut dyn FnMut(&mut dyn PictureLayerUpdate)) {
            for layer in &mut self.layers {
                visitor(layer);
            }
        }
        fn report_layers_updated(&mut self, count: usize) {
            self.reported = Some(count);
        }
    }

    #[test]
    fn invalid_layers_release_resources_without_updating_tiles() {
        let mut context = Context {
            needs: true,
            release: true,
            layers: vec![
                Layer {
                    valid: false,
                    changed: false,
                    released: false,
                    updated: false,
                },
                Layer {
                    valid: true,
                    changed: true,
                    released: false,
                    updated: false,
                },
            ],
            reported: None,
        };
        let stats = UpdatePictureLayerTiles(&mut context);
        assert_eq!(stats.visited, 2);
        assert_eq!(stats.updated, 1);
        assert_eq!(stats.invalid_priorities, 1);
        assert_eq!(stats.released_resources, 1);
        assert!(stats.changed);
        assert!(context.layers[0].released);
        assert!(!context.layers[0].updated);
        assert!(context.layers[1].updated);
        assert_eq!(context.reported, Some(1));
    }

    #[test]
    fn clean_tree_does_not_visit_or_clear_state() {
        let mut context = Context {
            needs: false,
            release: true,
            layers: Vec::new(),
            reported: None,
        };
        assert_eq!(
            UpdatePictureLayerTiles(&mut context),
            PictureLayerUpdateStats::default()
        );
        assert_eq!(context.reported, None);
    }
}
