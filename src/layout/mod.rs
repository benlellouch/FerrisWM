use indexmap::IndexMap;
use log::{debug, error};

use crate::{
    config::DEFAULT_LAYOUT,
    layout::{horizontal_layout::HorizontalLayout, master_layout::MasterLayout},
};

pub mod horizontal_layout;
pub mod master_layout;

macro_rules! define_layouts {
    ( $( $variant:ident => $ty:path ),+ $(,)? ) => {
        #[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
        pub enum LayoutType {
            $( $variant ),+
        }

        fn build_layout_map() -> IndexMap<LayoutType, Box<dyn Layout>> {
            let mut map: IndexMap<LayoutType, Box<dyn Layout>> = IndexMap::default();
            $( map.insert(LayoutType::$variant, Box::new($ty)); )+
            map
        }
    };
}

// DEFINE LAYOUTS HERE
define_layouts! {
    HorizontalLayout => HorizontalLayout,
    MasterLayout => MasterLayout,
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

pub trait Layout {
    fn generate_layout(
        &self,
        area: Rect,
        weights: &[u32],
        border_width: u32,
        window_gap: u32,
    ) -> Vec<Rect>;
}

pub(super) fn pad(dim: u32, border: u32) -> u32 {
    (dim - 2 * border).max(1)
}

pub struct LayoutManager {
    layout_map: IndexMap<LayoutType, Box<dyn Layout>>,
    current_layout: LayoutType,
}

impl LayoutManager {
    pub fn new() -> Self {
        let map = build_layout_map();

        if map.is_empty() {
            panic!(
                "No layouts defined, layouts need to be defined in layout/mod.rs using the define_layouts! macro."
            )
        }

        let current_layout = if map.contains_key(&DEFAULT_LAYOUT) {
            DEFAULT_LAYOUT
        } else {
            // This shouldn't be possible
            error!("Layout {DEFAULT_LAYOUT:?} not defined in LayoutType.");
            map.get_index(0).map(|(key, _)| *key).unwrap()
        };

        LayoutManager {
            layout_map: map,
            current_layout,
        }
    }

    pub fn get_current_layout(&self) -> &dyn Layout {
        self.layout_map
            .get(&self.current_layout)
            .map(|layout| layout.as_ref())
            .unwrap()
    }

    pub fn cycle_layout(&mut self) {
        if let Some(current_idx) = self.layout_map.get_index_of(&self.current_layout) {
            let next_idx = (current_idx + 1) % self.layout_map.len();
            if let Some(layout) = self.layout_map.get_index(next_idx).map(|(key, _)| *key) {
                debug!("New layout activated: {layout:?}");
                self.current_layout = layout
            } else {
                error!("Failed to cycle layout");
            }
        }
    }
}

#[cfg(test)]
mod pad_tests {
    use super::*;

    #[test]
    fn pad_normal_subtraction() {
        // 100 - 2*10 = 80
        assert_eq!(pad(100, 10), 80);
    }

    #[test]
    fn pad_result_is_zero_returns_one() {
        // 20 - 2*10 = 0 → max(1) = 1
        assert_eq!(pad(20, 10), 1);
    }

    #[test]
    fn pad_border_zero() {
        // 50 - 0 = 50
        assert_eq!(pad(50, 0), 50);
    }

    #[test]
    fn pad_both_zero() {
        // 0 - 0 = 0 → max(1) = 1
        assert_eq!(pad(0, 0), 1);
    }

    #[test]
    fn pad_dim_one_border_zero() {
        assert_eq!(pad(1, 0), 1);
    }

    #[test]
    fn pad_large_values() {
        assert_eq!(pad(10000, 100), 9800);
    }

    #[test]
    fn pad_border_one() {
        // 100 - 2 = 98
        assert_eq!(pad(100, 1), 98);
    }

    #[test]
    fn pad_exact_fit() {
        // 2 - 2*1 = 0 → max(1) = 1
        assert_eq!(pad(2, 1), 1);
    }

    #[test]
    fn pad_just_above_zero() {
        // 3 - 2*1 = 1
        assert_eq!(pad(3, 1), 1);
    }
}

#[cfg(test)]
mod rect_tests {
    use super::*;

    #[test]
    fn rect_construction() {
        let r = Rect {
            x: 10,
            y: 20,
            w: 300,
            h: 400,
        };
        assert_eq!(r.x, 10);
        assert_eq!(r.y, 20);
        assert_eq!(r.w, 300);
        assert_eq!(r.h, 400);
    }

    #[test]
    fn rect_copy_semantics() {
        let r1 = Rect {
            x: 1,
            y: 2,
            w: 3,
            h: 4,
        };
        let r2 = r1; // Copy
        assert_eq!(r1.x, r2.x);
        assert_eq!(r1.y, r2.y);
        assert_eq!(r1.w, r2.w);
        assert_eq!(r1.h, r2.h);
    }

    #[test]
    fn rect_clone() {
        let r1 = Rect {
            x: 5,
            y: 6,
            w: 7,
            h: 8,
        };
        let r2 = r1.clone();
        assert_eq!(r1.x, r2.x);
        assert_eq!(r1.y, r2.y);
        assert_eq!(r1.w, r2.w);
        assert_eq!(r1.h, r2.h);
    }

    #[test]
    fn rect_debug_format() {
        let r = Rect {
            x: 0,
            y: 0,
            w: 100,
            h: 200,
        };
        let debug_str = format!("{:?}", r);
        assert!(debug_str.contains("Rect"));
        assert!(debug_str.contains("100"));
        assert!(debug_str.contains("200"));
    }

    #[test]
    fn rect_negative_coordinates() {
        let r = Rect {
            x: -10,
            y: -20,
            w: 50,
            h: 60,
        };
        assert_eq!(r.x, -10);
        assert_eq!(r.y, -20);
    }
}

#[cfg(test)]
mod layout_type_tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn layout_type_eq() {
        assert_eq!(LayoutType::HorizontalLayout, LayoutType::HorizontalLayout);
        assert_eq!(LayoutType::MasterLayout, LayoutType::MasterLayout);
    }

    #[test]
    fn layout_type_ne() {
        assert_ne!(LayoutType::HorizontalLayout, LayoutType::MasterLayout);
    }

    #[test]
    fn layout_type_clone_copy() {
        let a = LayoutType::HorizontalLayout;
        let b = a; // Copy
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn layout_type_debug() {
        let debug_str = format!("{:?}", LayoutType::HorizontalLayout);
        assert_eq!(debug_str, "HorizontalLayout");
        let debug_str = format!("{:?}", LayoutType::MasterLayout);
        assert_eq!(debug_str, "MasterLayout");
    }

    #[test]
    fn layout_type_hash() {
        let mut set = HashSet::new();
        set.insert(LayoutType::HorizontalLayout);
        set.insert(LayoutType::MasterLayout);
        set.insert(LayoutType::HorizontalLayout); // duplicate

        assert_eq!(set.len(), 2);
        assert!(set.contains(&LayoutType::HorizontalLayout));
        assert!(set.contains(&LayoutType::MasterLayout));
    }
}

#[cfg(test)]
mod layout_manager_tests {
    use super::*;

    fn test_area() -> Rect {
        Rect {
            x: 0,
            y: 0,
            w: 900,
            h: 600,
        }
    }

    #[test]
    fn new_creates_successfully() {
        let _manager = LayoutManager::new();
    }

    #[test]
    fn default_layout_is_horizontal() {
        // DEFAULT_LAYOUT is HorizontalLayout, which lays out windows side by side.
        // With 3 equal-weight windows all y values should be the same (horizontal tiling).
        let manager = LayoutManager::new();
        let rects = manager
            .get_current_layout()
            .generate_layout(test_area(), &[1, 1, 1], 0, 0);

        assert_eq!(rects.len(), 3);
        // HorizontalLayout: all windows share the same y
        assert_eq!(rects[0].y, rects[1].y);
        assert_eq!(rects[1].y, rects[2].y);
        // Each window is 1/3 width
        assert_eq!(rects[0].w, rects[1].w);
        assert_eq!(rects[1].w, rects[2].w);
    }

    #[test]
    fn cycle_layout_switches_to_master() {
        let mut manager = LayoutManager::new();
        manager.cycle_layout();

        // MasterLayout with 3 windows: first window takes the left half,
        // remaining two split the right half vertically.
        let rects = manager
            .get_current_layout()
            .generate_layout(test_area(), &[1, 1, 1], 0, 0);

        assert_eq!(rects.len(), 3);
        // Window 0 is on the left half, windows 1 and 2 share the right half
        assert_eq!(rects[1].x, rects[2].x);
        // Windows 1 and 2 are stacked vertically
        assert!(rects[2].y > rects[1].y);
    }

    #[test]
    fn cycle_layout_wraps_around() {
        let mut manager = LayoutManager::new();

        // We have 2 layouts: HorizontalLayout and MasterLayout.
        // Cycling twice should return to the original.
        let rects_before =
            manager
                .get_current_layout()
                .generate_layout(test_area(), &[1, 1, 1], 0, 0);

        manager.cycle_layout(); // → MasterLayout
        manager.cycle_layout(); // → back to HorizontalLayout

        let rects_after =
            manager
                .get_current_layout()
                .generate_layout(test_area(), &[1, 1, 1], 0, 0);

        assert_eq!(rects_before.len(), rects_after.len());
        for (a, b) in rects_before.iter().zip(rects_after.iter()) {
            assert_eq!(a.x, b.x);
            assert_eq!(a.y, b.y);
            assert_eq!(a.w, b.w);
            assert_eq!(a.h, b.h);
        }
    }

    #[test]
    fn cycle_layout_multiple_full_cycles() {
        let mut manager = LayoutManager::new();

        let rects_initial =
            manager
                .get_current_layout()
                .generate_layout(test_area(), &[1, 1], 0, 0);

        // Cycle through all layouts 3 full times (2 layouts × 3 = 6 cycles)
        for _ in 0..6 {
            manager.cycle_layout();
        }

        let rects_final = manager
            .get_current_layout()
            .generate_layout(test_area(), &[1, 1], 0, 0);

        for (a, b) in rects_initial.iter().zip(rects_final.iter()) {
            assert_eq!(a.x, b.x);
            assert_eq!(a.y, b.y);
            assert_eq!(a.w, b.w);
            assert_eq!(a.h, b.h);
        }
    }

    #[test]
    fn get_current_layout_single_window() {
        let manager = LayoutManager::new();
        let rects = manager
            .get_current_layout()
            .generate_layout(test_area(), &[1], 0, 0);

        assert_eq!(rects.len(), 1);
        // Single window should fill the area
        assert_eq!(rects[0].x, 0);
        assert_eq!(rects[0].y, 0);
        assert_eq!(rects[0].w, 900);
        assert_eq!(rects[0].h, 600);
    }

    #[test]
    fn layout_with_borders_and_gaps() {
        let manager = LayoutManager::new();
        let rects = manager
            .get_current_layout()
            .generate_layout(test_area(), &[1, 1], 2, 4);

        assert_eq!(rects.len(), 2);
        // With border_width=2 and window_gap=4, total_border=6
        // Windows should be smaller than raw halves
        assert!(rects[0].w < 450);
        assert!(rects[0].h < 600);
    }

    #[test]
    fn build_layout_map_contains_both_layouts() {
        let map = build_layout_map();
        assert_eq!(map.len(), 2);
        assert!(map.contains_key(&LayoutType::HorizontalLayout));
        assert!(map.contains_key(&LayoutType::MasterLayout));
    }
}
