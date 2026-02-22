use crate::layout::{Layout, Rect, pad};

pub struct MasterLayout;

impl Layout for MasterLayout {
    fn generate_layout(
        &self,
        area: Rect,
        weights: &[u32],
        border_width: u32,
        window_gap: u32,
    ) -> Vec<Rect> {
        let total_border = border_width + (window_gap / 2);
        let mut prev_x: u32 = window_gap;
        let mut prev_y: u32 = window_gap;
        let mut prev_h: u32 = area.h - window_gap;
        let mut prev_w: u32 = area.w - window_gap;
        let layout: Vec<Rect> = weights
            .iter()
            .enumerate()
            .map(|(i, _weight)| {
                if weights.len() - 1 == i {
                    Rect {
                        x: prev_x as i32,
                        y: prev_y as i32,
                        w: pad(prev_w, total_border),
                        h: pad(prev_h, total_border),
                    }
                } else if i % 2 == 0 {
                    let inner_w = prev_w / 2;
                    let rect = Rect {
                        x: prev_x as i32,
                        y: prev_y as i32,
                        w: pad(inner_w, total_border),
                        h: pad(prev_h, total_border),
                    };
                    prev_x += inner_w;
                    prev_w = inner_w;
                    rect
                } else {
                    let inner_h = prev_h / 2;
                    let rect = Rect {
                        x: prev_x as i32,
                        y: prev_y as i32,
                        w: pad(prev_w, total_border),
                        h: pad(inner_h, total_border),
                    };
                    prev_y += inner_h;
                    prev_h = inner_h;
                    rect
                }
            })
            .collect();

        layout
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::Rect;

    fn area(w: u32, h: u32) -> Rect {
        Rect { x: 0, y: 0, w, h }
    }

    // ── empty weights ───────────────────────────────────────────────

    #[test]
    fn empty_weights_returns_empty_vec() {
        let rects = MasterLayout.generate_layout(area(1000, 800), &[], 0, 0);
        assert!(rects.is_empty());
    }

    #[test]
    fn empty_weights_with_border_and_gap() {
        let rects = MasterLayout.generate_layout(area(1000, 800), &[], 5, 10);
        assert!(rects.is_empty());
    }

    // ── single window ───────────────────────────────────────────────

    #[test]
    fn single_window_no_border_no_gap() {
        // i=0, last window → takes full remaining space
        // prev_x=0, prev_y=0, prev_w=1000, prev_h=800
        // rect = {x:0, y:0, w:pad(1000,0)=1000, h:pad(800,0)=800}
        let rects = MasterLayout.generate_layout(area(1000, 800), &[1], 0, 0);
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].x, 0);
        assert_eq!(rects[0].y, 0);
        assert_eq!(rects[0].w, 1000);
        assert_eq!(rects[0].h, 800);
    }

    #[test]
    fn single_window_with_gap() {
        // total_border = 0 + 10/2 = 5
        // prev_x=10, prev_y=10, prev_w=990, prev_h=790
        // i=0, last: rect = {x:10, y:10, w:pad(990,5)=980, h:pad(790,5)=780}
        let rects = MasterLayout.generate_layout(area(1000, 800), &[1], 0, 10);
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].x, 10);
        assert_eq!(rects[0].y, 10);
        assert_eq!(rects[0].w, 980);
        assert_eq!(rects[0].h, 780);
    }

    #[test]
    fn single_window_with_border_only() {
        // total_border = 3 + 0/2 = 3
        // prev_x=0, prev_y=0, prev_w=1000, prev_h=800
        // i=0, last: rect = {x:0, y:0, w:pad(1000,3)=994, h:pad(800,3)=794}
        let rects = MasterLayout.generate_layout(area(1000, 800), &[1], 3, 0);
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].x, 0);
        assert_eq!(rects[0].y, 0);
        assert_eq!(rects[0].w, 994);
        assert_eq!(rects[0].h, 794);
    }

    #[test]
    fn single_window_with_border_and_gap() {
        // total_border = 2 + 4/2 = 4
        // prev_x=4, prev_y=4, prev_w=896, prev_h=596
        // i=0, last: rect = {x:4, y:4, w:pad(896,4)=888, h:pad(596,4)=588}
        let rects = MasterLayout.generate_layout(area(900, 600), &[1], 2, 4);
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].x, 4);
        assert_eq!(rects[0].y, 4);
        assert_eq!(rects[0].w, 888);
        assert_eq!(rects[0].h, 588);
    }

    // ── two windows (first even split, second is last) ──────────────

    #[test]
    fn two_windows_no_border_no_gap() {
        // prev_x=0, prev_y=0, prev_w=1000, prev_h=800, total_border=0
        // i=0, even, not last: inner_w=500
        //   rect={x:0,y:0,w:pad(500,0)=500,h:pad(800,0)=800}
        //   prev_x=500, prev_w=500
        // i=1, last:
        //   rect={x:500,y:0,w:pad(500,0)=500,h:pad(800,0)=800}
        let rects = MasterLayout.generate_layout(area(1000, 800), &[1, 1], 0, 0);
        assert_eq!(rects.len(), 2);

        assert_eq!(rects[0].x, 0);
        assert_eq!(rects[0].y, 0);
        assert_eq!(rects[0].w, 500);
        assert_eq!(rects[0].h, 800);

        assert_eq!(rects[1].x, 500);
        assert_eq!(rects[1].y, 0);
        assert_eq!(rects[1].w, 500);
        assert_eq!(rects[1].h, 800);
    }

    #[test]
    fn two_windows_with_gap() {
        // total_border = 0 + 10/2 = 5
        // prev_x=10, prev_y=10, prev_w=990, prev_h=790
        // i=0, even, not last: inner_w=990/2=495
        //   rect={x:10,y:10,w:pad(495,5)=485,h:pad(790,5)=780}
        //   prev_x=505, prev_w=495
        // i=1, last:
        //   rect={x:505,y:10,w:pad(495,5)=485,h:pad(790,5)=780}
        let rects = MasterLayout.generate_layout(area(1000, 800), &[1, 1], 0, 10);
        assert_eq!(rects.len(), 2);

        assert_eq!(rects[0].x, 10);
        assert_eq!(rects[0].y, 10);
        assert_eq!(rects[0].w, 485);
        assert_eq!(rects[0].h, 780);

        assert_eq!(rects[1].x, 505);
        assert_eq!(rects[1].y, 10);
        assert_eq!(rects[1].w, 485);
        assert_eq!(rects[1].h, 780);
    }

    // ── three windows (even split, odd split, last) ─────────────────

    #[test]
    fn three_windows_no_border_no_gap() {
        // prev_x=0, prev_y=0, prev_w=1000, prev_h=800, total_border=0
        // i=0, even, not last: inner_w=500
        //   rect={x:0,y:0,w:500,h:800}, prev_x=500, prev_w=500
        // i=1, odd, not last: inner_h=400
        //   rect={x:500,y:0,w:500,h:400}, prev_y=400, prev_h=400
        // i=2, last:
        //   rect={x:500,y:400,w:500,h:400}
        let rects = MasterLayout.generate_layout(area(1000, 800), &[1, 1, 1], 0, 0);
        assert_eq!(rects.len(), 3);

        // Master window takes left half
        assert_eq!(rects[0].x, 0);
        assert_eq!(rects[0].y, 0);
        assert_eq!(rects[0].w, 500);
        assert_eq!(rects[0].h, 800);

        // Second window: top-right
        assert_eq!(rects[1].x, 500);
        assert_eq!(rects[1].y, 0);
        assert_eq!(rects[1].w, 500);
        assert_eq!(rects[1].h, 400);

        // Third window: bottom-right
        assert_eq!(rects[2].x, 500);
        assert_eq!(rects[2].y, 400);
        assert_eq!(rects[2].w, 500);
        assert_eq!(rects[2].h, 400);
    }

    #[test]
    fn three_windows_with_border_and_gap() {
        // total_border = 2 + 4/2 = 4
        // prev_x=4, prev_y=4, prev_w=896, prev_h=596
        // i=0, even, not last: inner_w=896/2=448
        //   rect={x:4,y:4,w:pad(448,4)=440,h:pad(596,4)=588}
        //   prev_x=452, prev_w=448
        // i=1, odd, not last: inner_h=596/2=298
        //   rect={x:452,y:4,w:pad(448,4)=440,h:pad(298,4)=290}
        //   prev_y=302, prev_h=298
        // i=2, last:
        //   rect={x:452,y:302,w:pad(448,4)=440,h:pad(298,4)=290}
        let rects = MasterLayout.generate_layout(area(900, 600), &[1, 1, 1], 2, 4);
        assert_eq!(rects.len(), 3);

        assert_eq!(rects[0].x, 4);
        assert_eq!(rects[0].y, 4);
        assert_eq!(rects[0].w, 440);
        assert_eq!(rects[0].h, 588);

        assert_eq!(rects[1].x, 452);
        assert_eq!(rects[1].y, 4);
        assert_eq!(rects[1].w, 440);
        assert_eq!(rects[1].h, 290);

        assert_eq!(rects[2].x, 452);
        assert_eq!(rects[2].y, 302);
        assert_eq!(rects[2].w, 440);
        assert_eq!(rects[2].h, 290);
    }

    // ── four windows (alternating even/odd splits) ──────────────────

    #[test]
    fn four_windows_no_border_no_gap() {
        // prev_x=0, prev_y=0, prev_w=1000, prev_h=800, total_border=0
        // i=0, even, not last: inner_w=500
        //   rect={x:0,y:0,w:500,h:800}, prev_x=500, prev_w=500
        // i=1, odd, not last: inner_h=400
        //   rect={x:500,y:0,w:500,h:400}, prev_y=400, prev_h=400
        // i=2, even, not last: inner_w=250
        //   rect={x:500,y:400,w:250,h:400}, prev_x=750, prev_w=250
        // i=3, last:
        //   rect={x:750,y:400,w:250,h:400}
        let rects = MasterLayout.generate_layout(area(1000, 800), &[1, 1, 1, 1], 0, 0);
        assert_eq!(rects.len(), 4);

        assert_eq!(rects[0].x, 0);
        assert_eq!(rects[0].y, 0);
        assert_eq!(rects[0].w, 500);
        assert_eq!(rects[0].h, 800);

        assert_eq!(rects[1].x, 500);
        assert_eq!(rects[1].y, 0);
        assert_eq!(rects[1].w, 500);
        assert_eq!(rects[1].h, 400);

        assert_eq!(rects[2].x, 500);
        assert_eq!(rects[2].y, 400);
        assert_eq!(rects[2].w, 250);
        assert_eq!(rects[2].h, 400);

        assert_eq!(rects[3].x, 750);
        assert_eq!(rects[3].y, 400);
        assert_eq!(rects[3].w, 250);
        assert_eq!(rects[3].h, 400);
    }

    // ── five windows (full spiral pattern) ──────────────────────────

    #[test]
    fn five_windows_no_border_no_gap() {
        // prev_x=0, prev_y=0, prev_w=1000, prev_h=800, total_border=0
        // i=0, even, not last: inner_w=500
        //   rect={x:0,y:0,w:500,h:800}, prev_x=500, prev_w=500
        // i=1, odd, not last: inner_h=400
        //   rect={x:500,y:0,w:500,h:400}, prev_y=400, prev_h=400
        // i=2, even, not last: inner_w=250
        //   rect={x:500,y:400,w:250,h:400}, prev_x=750, prev_w=250
        // i=3, odd, not last: inner_h=200
        //   rect={x:750,y:400,w:250,h:200}, prev_y=600, prev_h=200
        // i=4, last:
        //   rect={x:750,y:600,w:250,h:200}
        let rects = MasterLayout.generate_layout(area(1000, 800), &[1, 1, 1, 1, 1], 0, 0);
        assert_eq!(rects.len(), 5);

        assert_eq!(rects[0].x, 0);
        assert_eq!(rects[0].y, 0);
        assert_eq!(rects[0].w, 500);
        assert_eq!(rects[0].h, 800);

        assert_eq!(rects[1].x, 500);
        assert_eq!(rects[1].y, 0);
        assert_eq!(rects[1].w, 500);
        assert_eq!(rects[1].h, 400);

        assert_eq!(rects[2].x, 500);
        assert_eq!(rects[2].y, 400);
        assert_eq!(rects[2].w, 250);
        assert_eq!(rects[2].h, 400);

        assert_eq!(rects[3].x, 750);
        assert_eq!(rects[3].y, 400);
        assert_eq!(rects[3].w, 250);
        assert_eq!(rects[3].h, 200);

        assert_eq!(rects[4].x, 750);
        assert_eq!(rects[4].y, 600);
        assert_eq!(rects[4].w, 250);
        assert_eq!(rects[4].h, 200);
    }

    // ── master window is always the largest ─────────────────────────

    #[test]
    fn master_window_has_largest_area() {
        let rects = MasterLayout.generate_layout(area(1200, 800), &[1, 1, 1, 1], 0, 0);
        let master_area = rects[0].w as u64 * rects[0].h as u64;
        for r in &rects[1..] {
            let window_area = r.w as u64 * r.h as u64;
            assert!(
                master_area >= window_area,
                "master area {} should be >= secondary area {}",
                master_area,
                window_area
            );
        }
    }

    // ── weights are ignored by MasterLayout ─────────────────────────

    #[test]
    fn weights_values_are_ignored() {
        let rects_ones = MasterLayout.generate_layout(area(1000, 800), &[1, 1, 1], 0, 0);
        let rects_mixed = MasterLayout.generate_layout(area(1000, 800), &[5, 10, 2], 0, 0);

        assert_eq!(rects_ones.len(), rects_mixed.len());
        for (a, b) in rects_ones.iter().zip(rects_mixed.iter()) {
            assert_eq!(a.x, b.x);
            assert_eq!(a.y, b.y);
            assert_eq!(a.w, b.w);
            assert_eq!(a.h, b.h);
        }
    }

    // ── no overlapping windows ──────────────────────────────────────

    #[test]
    fn windows_do_not_overlap_three() {
        let rects = MasterLayout.generate_layout(area(1000, 800), &[1, 1, 1], 0, 0);
        for i in 0..rects.len() {
            for j in (i + 1)..rects.len() {
                let a = &rects[i];
                let b = &rects[j];
                let no_overlap = a.x + a.w as i32 <= b.x
                    || b.x + b.w as i32 <= a.x
                    || a.y + a.h as i32 <= b.y
                    || b.y + b.h as i32 <= a.y;
                assert!(
                    no_overlap,
                    "window {} ({:?}) overlaps window {} ({:?})",
                    i, a, j, b
                );
            }
        }
    }

    #[test]
    fn windows_do_not_overlap_five() {
        let rects = MasterLayout.generate_layout(area(1600, 900), &[1, 1, 1, 1, 1], 2, 6);
        for i in 0..rects.len() {
            for j in (i + 1)..rects.len() {
                let a = &rects[i];
                let b = &rects[j];
                let no_overlap = a.x + a.w as i32 <= b.x
                    || b.x + b.w as i32 <= a.x
                    || a.y + a.h as i32 <= b.y
                    || b.y + b.h as i32 <= a.y;
                assert!(
                    no_overlap,
                    "window {} ({:?}) overlaps window {} ({:?})",
                    i, a, j, b
                );
            }
        }
    }

    // ── all windows stay within the area bounds ─────────────────────

    #[test]
    fn all_windows_within_bounds_no_gap() {
        let a = area(1000, 800);
        let rects = MasterLayout.generate_layout(a, &[1, 1, 1, 1], 0, 0);
        for (i, r) in rects.iter().enumerate() {
            assert!(r.x >= 0, "window {} x={} out of bounds", i, r.x);
            assert!(r.y >= 0, "window {} y={} out of bounds", i, r.y);
            assert!(
                r.x as u32 + r.w <= a.w,
                "window {} right edge {} exceeds width {}",
                i,
                r.x as u32 + r.w,
                a.w
            );
            assert!(
                r.y as u32 + r.h <= a.h,
                "window {} bottom edge {} exceeds height {}",
                i,
                r.y as u32 + r.h,
                a.h
            );
        }
    }

    // ── gap applies initial offset ──────────────────────────────────

    #[test]
    fn gap_offsets_first_window() {
        let rects = MasterLayout.generate_layout(area(1000, 800), &[1], 0, 20);
        assert_eq!(rects[0].x, 20);
        assert_eq!(rects[0].y, 20);
    }

    #[test]
    fn gap_zero_no_offset() {
        let rects = MasterLayout.generate_layout(area(1000, 800), &[1], 0, 0);
        assert_eq!(rects[0].x, 0);
        assert_eq!(rects[0].y, 0);
    }

    // ── border reduces window dimensions ────────────────────────────

    #[test]
    fn border_reduces_dimensions() {
        let rects_no_border = MasterLayout.generate_layout(area(1000, 800), &[1, 1], 0, 0);
        let rects_with_border = MasterLayout.generate_layout(area(1000, 800), &[1, 1], 5, 0);

        // Same positions (no gap change), but smaller dimensions
        assert_eq!(rects_no_border[0].x, rects_with_border[0].x);
        assert!(rects_with_border[0].w < rects_no_border[0].w);
        assert!(rects_with_border[0].h < rects_no_border[0].h);
    }

    // ── odd window_gap (integer division of gap/2) ──────────────────

    #[test]
    fn odd_gap_value() {
        // total_border = 0 + 7/2 = 3 (integer division)
        // prev_x=7, prev_y=7, prev_w=993, prev_h=793
        // i=0, last: rect={x:7,y:7,w:pad(993,3)=987,h:pad(793,3)=787}
        let rects = MasterLayout.generate_layout(area(1000, 800), &[1], 0, 7);
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].x, 7);
        assert_eq!(rects[0].y, 7);
        assert_eq!(rects[0].w, 987);
        assert_eq!(rects[0].h, 787);
    }

    // ── pad clamp edge case (very small remaining space) ────────────

    #[test]
    fn small_area_clamps_to_one() {
        // area 20x20, border=4, gap=4
        // total_border = 4 + 2 = 6
        // prev_x=4, prev_y=4, prev_w=16, prev_h=16
        // i=0, last: rect={x:4,y:4,w:pad(16,6)=4,h:pad(16,6)=4}
        let rects = MasterLayout.generate_layout(area(20, 20), &[1], 4, 4);
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].x, 4);
        assert_eq!(rects[0].y, 4);
        assert_eq!(rects[0].w, 4);
        assert_eq!(rects[0].h, 4);
    }

    #[test]
    fn very_small_area_pad_returns_one() {
        // area 14x14, border=3, gap=4
        // total_border = 3 + 2 = 5
        // prev_x=4, prev_y=4, prev_w=10, prev_h=10
        // i=0, last: rect={x:4,y:4,w:pad(10,5)=0->1,h:pad(10,5)=0->1}
        let rects = MasterLayout.generate_layout(area(14, 14), &[1], 3, 4);
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].w, 1);
        assert_eq!(rects[0].h, 1);
    }

    // ── number of output rects matches number of weights ────────────

    #[test]
    fn output_count_matches_weight_count() {
        for n in 1..=8 {
            let weights: Vec<u32> = vec![1; n];
            let rects = MasterLayout.generate_layout(area(2000, 1500), &weights, 2, 4);
            assert_eq!(rects.len(), n, "expected {} rects, got {}", n, rects.len());
        }
    }

    // ── area origin is ignored by layout ────────────────────────────

    #[test]
    fn area_origin_not_used_in_computation() {
        let shifted = Rect {
            x: 200,
            y: 100,
            w: 1000,
            h: 800,
        };
        let origin = area(1000, 800);

        let rects_shifted = MasterLayout.generate_layout(shifted, &[1, 1, 1], 0, 0);
        let rects_origin = MasterLayout.generate_layout(origin, &[1, 1, 1], 0, 0);

        // Layout uses area.w and area.h only, not area.x/area.y
        for (a, b) in rects_shifted.iter().zip(rects_origin.iter()) {
            assert_eq!(a.x, b.x);
            assert_eq!(a.y, b.y);
            assert_eq!(a.w, b.w);
            assert_eq!(a.h, b.h);
        }
    }

    // ── dwindle property: each subsequent region is <= half ─────────

    #[test]
    fn regions_shrink_with_more_windows() {
        let rects = MasterLayout.generate_layout(area(1000, 800), &[1, 1, 1, 1, 1], 0, 0);

        // Each non-last window splits in half, so areas should not increase
        let areas: Vec<u64> = rects.iter().map(|r| r.w as u64 * r.h as u64).collect();
        for i in 1..areas.len() {
            assert!(
                areas[i] <= areas[i - 1],
                "area[{}]={} should be <= area[{}]={}",
                i,
                areas[i],
                i - 1,
                areas[i - 1]
            );
        }
    }

    // ── even split is horizontal, odd split is vertical ─────────────

    #[test]
    fn even_index_splits_horizontally() {
        // With 3 windows: i=0 (even) does horizontal split
        // Window 0 should occupy the left half of the screen
        let rects = MasterLayout.generate_layout(area(1000, 800), &[1, 1, 1], 0, 0);
        // Window 0 width should be half the total
        assert_eq!(rects[0].w, 500);
        // Window 0 height should be full height
        assert_eq!(rects[0].h, 800);
    }

    #[test]
    fn odd_index_splits_vertically() {
        // With 4 windows: i=1 (odd) does vertical split
        // Window 1 should occupy the top half of the right side
        let rects = MasterLayout.generate_layout(area(1000, 800), &[1, 1, 1, 1], 0, 0);
        // Window 1 height should be half the total
        assert_eq!(rects[1].h, 400);
        // Window 1 width should span the remaining horizontal space
        assert_eq!(rects[1].w, 500);
    }

    // ── large number of windows ─────────────────────────────────────

    #[test]
    fn eight_windows_all_have_positive_dimensions() {
        let weights = vec![1u32; 8];
        let rects = MasterLayout.generate_layout(area(1920, 1080), &weights, 1, 2);
        assert_eq!(rects.len(), 8);
        for (i, r) in rects.iter().enumerate() {
            assert!(r.w > 0, "window {} has zero width", i);
            assert!(r.h > 0, "window {} has zero height", i);
        }
    }
}
