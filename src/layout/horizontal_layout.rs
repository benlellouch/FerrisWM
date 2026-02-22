use crate::layout::{Layout, Rect, pad};

pub struct HorizontalLayout;

impl Layout for HorizontalLayout {
    fn generate_layout(
        &self,
        area: Rect,
        weights: &[u32],
        border_width: u32,
        window_gap: u32,
    ) -> Vec<Rect> {
        let total_weights: u32 = weights.iter().sum();
        let total_border = border_width + window_gap;
        let inner_h = pad(area.h, total_border);
        let partitions = area.w / total_weights;

        let mut cumulative = 0u32;
        let layout: Vec<Rect> = weights
            .iter()
            .map(|weight| {
                let cell = (area.w * weight) / total_weights;
                let inner_w = pad(cell, total_border);
                let x = cumulative * partitions + window_gap;
                cumulative += weight;
                Rect {
                    x: x as i32,
                    y: window_gap as i32,
                    w: inner_w,
                    h: inner_h,
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

    // ── single window ───────────────────────────────────────────────

    #[test]
    fn single_window_no_border_no_gap() {
        let rects = HorizontalLayout.generate_layout(area(1000, 800), &[1], 0, 0);
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].x, 0);
        assert_eq!(rects[0].y, 0);
        assert_eq!(rects[0].w, 1000);
        assert_eq!(rects[0].h, 800);
    }

    #[test]
    fn single_window_with_border_and_gap() {
        // total_border = border_width + window_gap = 2 + 4 = 6
        // inner_h = pad(800, 6) = 800 - 12 = 788
        // cell = (1000 * 1) / 1 = 1000
        // inner_w = pad(1000, 6) = 1000 - 12 = 988
        // x = 0 * 1000 + 4 = 4
        let rects = HorizontalLayout.generate_layout(area(1000, 800), &[1], 2, 4);
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].x, 4);
        assert_eq!(rects[0].y, 4);
        assert_eq!(rects[0].w, 988);
        assert_eq!(rects[0].h, 788);
    }

    // ── equal weights ───────────────────────────────────────────────

    #[test]
    fn two_equal_windows_no_border_no_gap() {
        // total_weights = 2, partitions = 1000/2 = 500
        // Window 0: cell=500, inner_w=500, x=0*500+0=0
        // Window 1: cell=500, inner_w=500, x=1*500+0=500
        let rects = HorizontalLayout.generate_layout(area(1000, 800), &[1, 1], 0, 0);
        assert_eq!(rects.len(), 2);

        assert_eq!(rects[0].x, 0);
        assert_eq!(rects[0].w, 500);
        assert_eq!(rects[0].h, 800);

        assert_eq!(rects[1].x, 500);
        assert_eq!(rects[1].w, 500);
        assert_eq!(rects[1].h, 800);

        // Both at same y
        assert_eq!(rects[0].y, rects[1].y);
    }

    #[test]
    fn three_equal_windows_no_border_no_gap() {
        // total_weights = 3, partitions = 900/3 = 300
        // Window 0: cell = 300, x = 0
        // Window 1: cell = 300, x = 1*300 = 300
        // Window 2: cell = 300, x = 2*300 = 600
        let rects = HorizontalLayout.generate_layout(area(900, 600), &[1, 1, 1], 0, 0);
        assert_eq!(rects.len(), 3);

        assert_eq!(rects[0].x, 0);
        assert_eq!(rects[0].w, 300);

        assert_eq!(rects[1].x, 300);
        assert_eq!(rects[1].w, 300);

        assert_eq!(rects[2].x, 600);
        assert_eq!(rects[2].w, 300);

        // All heights equal to area height
        for r in &rects {
            assert_eq!(r.h, 600);
            assert_eq!(r.y, 0);
        }
    }

    // ── unequal weights ─────────────────────────────────────────────

    #[test]
    fn two_windows_weight_2_1() {
        // total_weights = 3, partitions = 900/3 = 300
        // Window 0: weight=2, cell=(900*2)/3=600, inner_w=600, x=0*300+0=0
        // Window 1: weight=1, cell=(900*1)/3=300, inner_w=300, x=2*300+0=600
        let rects = HorizontalLayout.generate_layout(area(900, 600), &[2, 1], 0, 0);
        assert_eq!(rects.len(), 2);

        assert_eq!(rects[0].x, 0);
        assert_eq!(rects[0].w, 600);

        assert_eq!(rects[1].x, 600);
        assert_eq!(rects[1].w, 300);
    }

    #[test]
    fn three_windows_weight_1_2_1() {
        // total_weights = 4, partitions = 1000/4 = 250
        // Window 0: weight=1, cell=250, x = 0*250 = 0
        // Window 1: weight=2, cell=500, x = 1*250 = 250
        // Window 2: weight=1, cell=250, x = 3*250 = 750
        let rects = HorizontalLayout.generate_layout(area(1000, 600), &[1, 2, 1], 0, 0);
        assert_eq!(rects.len(), 3);

        assert_eq!(rects[0].x, 0);
        assert_eq!(rects[0].w, 250);

        assert_eq!(rects[1].x, 250);
        assert_eq!(rects[1].w, 500);

        assert_eq!(rects[2].x, 750);
        assert_eq!(rects[2].w, 250);
    }

    #[test]
    fn two_windows_weight_1_3() {
        // total_weights = 4, partitions = 800/4 = 200
        // Window 0: weight=1, cell=(800*1)/4=200, x=0*200=0
        // Window 1: weight=3, cell=(800*3)/4=600, x=1*200=200
        let rects = HorizontalLayout.generate_layout(area(800, 400), &[1, 3], 0, 0);
        assert_eq!(rects.len(), 2);

        assert_eq!(rects[0].x, 0);
        assert_eq!(rects[0].w, 200);

        assert_eq!(rects[1].x, 200);
        assert_eq!(rects[1].w, 600);
    }

    // ── borders and gaps ────────────────────────────────────────────

    #[test]
    fn two_equal_windows_with_gap() {
        // border_width=0, window_gap=10, total_border=10
        // inner_h = pad(800, 10) = 800 - 20 = 780
        // total_weights = 2, partitions = 1000/2 = 500
        // Window 0: cell=500, inner_w=pad(500,10)=480, x=0*500+10=10
        // Window 1: cell=500, inner_w=480, x=1*500+10=510
        let rects = HorizontalLayout.generate_layout(area(1000, 800), &[1, 1], 0, 10);
        assert_eq!(rects.len(), 2);

        assert_eq!(rects[0].x, 10);
        assert_eq!(rects[0].y, 10);
        assert_eq!(rects[0].w, 480);
        assert_eq!(rects[0].h, 780);

        assert_eq!(rects[1].x, 510);
        assert_eq!(rects[1].y, 10);
        assert_eq!(rects[1].w, 480);
        assert_eq!(rects[1].h, 780);
    }

    #[test]
    fn two_equal_windows_with_border_only() {
        // border_width=5, window_gap=0, total_border=5
        // inner_h = pad(600, 5) = 600 - 10 = 590
        // total_weights = 2, partitions = 1000/2 = 500
        // Window 0: cell=500, inner_w=pad(500,5)=490, x=0+0=0
        // Window 1: cell=500, inner_w=490, x=500+0=500
        let rects = HorizontalLayout.generate_layout(area(1000, 600), &[1, 1], 5, 0);
        assert_eq!(rects.len(), 2);

        assert_eq!(rects[0].x, 0);
        assert_eq!(rects[0].y, 0);
        assert_eq!(rects[0].w, 490);
        assert_eq!(rects[0].h, 590);

        assert_eq!(rects[1].x, 500);
        assert_eq!(rects[1].w, 490);
        assert_eq!(rects[1].h, 590);
    }

    #[test]
    fn three_windows_with_border_and_gap() {
        // border_width=2, window_gap=4, total_border=6
        // inner_h = pad(600, 6) = 600 - 12 = 588
        // total_weights = 3, partitions = 900/3 = 300
        // Window 0: cell=300, inner_w=pad(300,6)=288, x=0*300+4=4
        // Window 1: cell=300, inner_w=288, x=1*300+4=304
        // Window 2: cell=300, inner_w=288, x=2*300+4=604
        let rects = HorizontalLayout.generate_layout(area(900, 600), &[1, 1, 1], 2, 4);
        assert_eq!(rects.len(), 3);

        assert_eq!(rects[0].x, 4);
        assert_eq!(rects[0].y, 4);
        assert_eq!(rects[0].w, 288);
        assert_eq!(rects[0].h, 588);

        assert_eq!(rects[1].x, 304);
        assert_eq!(rects[1].w, 288);

        assert_eq!(rects[2].x, 604);
        assert_eq!(rects[2].w, 288);
    }

    // ── many windows ────────────────────────────────────────────────

    #[test]
    fn five_equal_windows() {
        let rects = HorizontalLayout.generate_layout(area(1000, 500), &[1, 1, 1, 1, 1], 0, 0);
        assert_eq!(rects.len(), 5);

        // partitions = 1000/5 = 200, each cell = 200
        for (i, r) in rects.iter().enumerate() {
            assert_eq!(r.x, (i as i32) * 200);
            assert_eq!(r.w, 200);
            assert_eq!(r.h, 500);
            assert_eq!(r.y, 0);
        }
    }

    // ── all y values are identical (horizontal tiling property) ─────

    #[test]
    fn all_windows_share_same_y() {
        let rects = HorizontalLayout.generate_layout(area(1200, 700), &[1, 2, 3, 1], 3, 6);
        for r in &rects {
            assert_eq!(r.y, 6);
        }
    }

    #[test]
    fn all_windows_share_same_height() {
        let rects = HorizontalLayout.generate_layout(area(1200, 700), &[1, 2, 3, 1], 3, 6);
        // total_border = 3 + 6 = 9, inner_h = pad(700, 9) = 700 - 18 = 682
        let expected_h = 682;
        for r in &rects {
            assert_eq!(r.h, expected_h);
        }
    }

    // ── x positions are monotonically increasing ────────────────────

    #[test]
    fn x_positions_are_increasing() {
        let rects = HorizontalLayout.generate_layout(area(1600, 900), &[1, 1, 1, 1], 2, 8);
        for i in 1..rects.len() {
            assert!(
                rects[i].x > rects[i - 1].x,
                "x[{}]={} should be > x[{}]={}",
                i,
                rects[i].x,
                i - 1,
                rects[i - 1].x
            );
        }
    }

    // ── large weight values ─────────────────────────────────────────

    #[test]
    fn large_weight_values() {
        // weights = [100, 100], should behave like [1, 1]
        let rects_big = HorizontalLayout.generate_layout(area(1000, 800), &[100, 100], 0, 0);
        let rects_small = HorizontalLayout.generate_layout(area(1000, 800), &[1, 1], 0, 0);

        assert_eq!(rects_big.len(), rects_small.len());
        for (a, b) in rects_big.iter().zip(rects_small.iter()) {
            assert_eq!(a.x, b.x);
            assert_eq!(a.y, b.y);
            assert_eq!(a.w, b.w);
            assert_eq!(a.h, b.h);
        }
    }

    // ── pad clamp edge case (very small cell) ───────────────────────

    #[test]
    fn small_area_clamps_to_one() {
        // area.w=20, weights=[1], border_width=5, window_gap=5
        // total_border = 10, cell = 20, inner_w = pad(20, 10) = 0 → 1
        // inner_h = pad(20, 10) = 0 → 1
        let rects = HorizontalLayout.generate_layout(area(20, 20), &[1], 5, 5);
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].w, 1);
        assert_eq!(rects[0].h, 1);
    }

    // ── non-zero area origin (x, y offsets are ignored by layout) ───

    #[test]
    fn area_origin_does_not_affect_width_height() {
        let shifted_area = Rect {
            x: 100,
            y: 50,
            w: 800,
            h: 600,
        };
        let rects = HorizontalLayout.generate_layout(shifted_area, &[1, 1], 0, 0);
        assert_eq!(rects.len(), 2);
        // The layout uses area.w and area.h, not area.x/area.y for sizing
        assert_eq!(rects[0].w, 400);
        assert_eq!(rects[1].w, 400);
        assert_eq!(rects[0].h, 600);
    }

    // ── weight proportionality ──────────────────────────────────────

    #[test]
    fn heavier_weight_gets_wider_window() {
        let rects = HorizontalLayout.generate_layout(area(1000, 500), &[1, 3], 0, 0);
        assert!(
            rects[1].w > rects[0].w,
            "window with weight 3 (w={}) should be wider than weight 1 (w={})",
            rects[1].w,
            rects[0].w
        );
    }

    #[test]
    fn equal_weights_produce_equal_widths() {
        let rects = HorizontalLayout.generate_layout(area(900, 600), &[2, 2, 2], 0, 0);
        assert_eq!(rects[0].w, rects[1].w);
        assert_eq!(rects[1].w, rects[2].w);
    }

    // ── empty weights panics (division by zero) ─────────────────────

    #[test]
    #[should_panic]
    fn empty_weights_panics() {
        HorizontalLayout.generate_layout(area(1000, 800), &[], 0, 0);
    }
}
