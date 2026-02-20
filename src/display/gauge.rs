use super::*;

pub(super) fn draw_gauge_scale<D>(display: &mut D)
where
    D: DrawTarget<Color = DisplayColor>,
{
    draw_gauge_gradient_span(
        display,
        GAUGE_START_DEG,
        GAUGE_TOTAL_SWEEP_DEG,
        GAUGE_GRADIENT_STEP_DEG_STATIC,
    );
}

pub(super) fn update_status_if_changed<D>(
    display: &mut D,
    cache: &mut DisplayCache,
    text: &str,
    pm25: Option<u16>,
) where
    D: DrawTarget<Color = DisplayColor>,
{
    let text_changed = cache.status_text.as_str() != text;
    let pm25_changed = cache.status_pm25 != pm25;
    if !text_changed && !pm25_changed {
        return;
    }

    let prev_angle = cache.status_pm25.map(|v| gauge_angle(status_ratio(v)));
    let next_angle = pm25.map(|v| gauge_angle(status_ratio(v)));
    let redraw_threshold = GAUGE_NEEDLE_MIN_REDRAW_DEG.max(0.0);
    let should_redraw_needle = match (prev_angle, next_angle) {
        (Some(prev), Some(next)) => (next - prev).abs() >= redraw_threshold,
        (None, None) => false,
        _ => true,
    };

    // Keep text and pointer synchronized: redraw both only when the pointer redraws.
    if !should_redraw_needle {
        return;
    }

    let should_erase_needle = prev_angle.is_some();
    let should_draw_needle = next_angle.is_some();

    if should_erase_needle {
        if let Some(angle) = prev_angle {
            erase_status_needle(display, angle);
            // Fast mode keeps the needle inside the arc, so no arc restore is needed.
            if !GAUGE_NEEDLE_FAST_MODE {
                restore_gauge_slice(display, angle);
            }
        }
    }

    let value_style = TextStyleCfg {
        font: STYLE_STATUS_TEXT.font,
        color: pm25
            .map(|value| gauge_gradient_color(status_ratio(value)))
            .unwrap_or(TEXT_DIM),
    };
    let font = font_for(value_style.font);
    clear_rect(display, status_text_clear_rect(font));

    let text_pos = centered_status_text_pos(font, text);
    draw_text_aa(display, text_pos, font, value_style.color, text);

    if should_draw_needle {
        if let Some(angle) = next_angle {
            draw_status_needle(display, angle);
        }
    }

    cache.status_text.clear();
    let _ = cache.status_text.push_str(text);
    cache.status_pm25 = pm25;
}

fn draw_arc_band<D>(display: &mut D, start_deg: f32, sweep_deg: f32, color: DisplayColor)
where
    D: DrawTarget<Color = DisplayColor>,
{
    let outer = darken(color, 8);
    let edge = darken(color, 5);
    let fill = color;
    let highlight = brighten(color, 4);

    let arc = Arc::with_center(
        GAUGE_CENTER,
        GAUGE_DIAMETER,
        start_deg.deg(),
        sweep_deg.deg(),
    );
    let _ = arc
        .into_styled(PrimitiveStyle::with_stroke(outer, GAUGE_BAND_OUTER_W))
        .draw(display);

    let arc = Arc::with_center(
        GAUGE_CENTER,
        GAUGE_DIAMETER,
        start_deg.deg(),
        sweep_deg.deg(),
    );
    let _ = arc
        .into_styled(PrimitiveStyle::with_stroke(edge, GAUGE_BAND_EDGE_W))
        .draw(display);

    let arc = Arc::with_center(
        GAUGE_CENTER,
        GAUGE_DIAMETER,
        start_deg.deg(),
        sweep_deg.deg(),
    );
    let _ = arc
        .into_styled(PrimitiveStyle::with_stroke(fill, GAUGE_BAND_FILL_W))
        .draw(display);

    let arc = Arc::with_center(
        GAUGE_CENTER,
        GAUGE_DIAMETER,
        start_deg.deg(),
        sweep_deg.deg(),
    );
    let _ = arc
        .into_styled(PrimitiveStyle::with_stroke(
            highlight,
            GAUGE_BAND_HIGHLIGHT_W,
        ))
        .draw(display);
}

fn draw_status_needle<D>(display: &mut D, angle_deg: f32)
where
    D: DrawTarget<Color = DisplayColor>,
{
    if GAUGE_NEEDLE_FAST_MODE {
        draw_status_needle_fast(display, angle_deg);
        return;
    }

    let inner_r = gauge_scale_i32(GAUGE_NEEDLE_INNER_R_BASE);
    let outer_r = pointer_outer_radius(inner_r);
    let arrow_len = gauge_scale_i32(GAUGE_ARROW_LEN_BASE);
    let arrow_half_w = gauge_scale_i32(GAUGE_ARROW_HALF_W_BASE);
    let arrow_tip_offset = gauge_scale_i32(GAUGE_ARROW_TIP_OFFSET_BASE);
    let shadow_pad = gauge_scale_i32_nonzero(GAUGE_ARROW_SHADOW_PAD_BASE);

    let (start, shaft_end, tip, left, right) = needle_geometry(
        angle_deg,
        inner_r,
        outer_r,
        arrow_len,
        arrow_half_w,
        arrow_tip_offset,
    );
    let (_, _, shadow_tip, shadow_left, shadow_right) = needle_geometry(
        angle_deg,
        inner_r,
        outer_r,
        arrow_len + shadow_pad,
        arrow_half_w + shadow_pad,
        arrow_tip_offset + shadow_pad,
    );

    draw_capsule_aa(
        display,
        start,
        shaft_end,
        gauge_scale_i32_nonzero(GAUGE_NEEDLE_SHADOW_W_BASE) as f32,
        GAUGE_NEEDLE_SHADOW_COLOR,
    );
    fill_triangle_aa(
        display,
        shadow_tip,
        shadow_left,
        shadow_right,
        GAUGE_NEEDLE_SHADOW_COLOR,
    );

    draw_capsule_aa(
        display,
        start,
        shaft_end,
        gauge_scale_i32_nonzero(GAUGE_NEEDLE_W_BASE) as f32,
        GAUGE_NEEDLE_COLOR,
    );
    fill_triangle_aa(display, tip, left, right, GAUGE_NEEDLE_COLOR);

    let _ = Circle::with_center(GAUGE_CENTER, gauge_scale_u32_nonzero(GAUGE_HUB_D_BASE))
        .into_styled(PrimitiveStyle::with_fill(GAUGE_HUB_COLOR))
        .draw(display);
}

fn erase_status_needle<D>(display: &mut D, angle_deg: f32)
where
    D: DrawTarget<Color = DisplayColor>,
{
    if GAUGE_NEEDLE_FAST_MODE {
        erase_status_needle_fast(display, angle_deg);
        return;
    }

    let inner_r = gauge_scale_i32(GAUGE_NEEDLE_INNER_R_BASE);
    let outer_r = pointer_outer_radius(inner_r);
    let clear_pad = gauge_scale_i32_nonzero(GAUGE_ARROW_CLEAR_PAD_BASE);

    let (start, _, tip, left, right) = needle_geometry(
        angle_deg,
        inner_r,
        outer_r,
        gauge_scale_i32(GAUGE_ARROW_LEN_BASE) + clear_pad,
        gauge_scale_i32(GAUGE_ARROW_HALF_W_BASE) + clear_pad,
        gauge_scale_i32(GAUGE_ARROW_TIP_OFFSET_BASE) + clear_pad,
    );

    draw_capsule_aa(
        display,
        start,
        tip,
        gauge_scale_i32_nonzero(GAUGE_NEEDLE_CLEAR_W_BASE) as f32,
        BG_COLOR,
    );

    fill_triangle_aa(display, tip, left, right, BG_COLOR);

    let _ = Circle::with_center(
        GAUGE_CENTER,
        gauge_scale_u32_nonzero(GAUGE_HUB_CLEAR_D_BASE),
    )
    .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
    .draw(display);
}

fn draw_status_needle_fast<D>(display: &mut D, angle_deg: f32)
where
    D: DrawTarget<Color = DisplayColor>,
{
    let inner_r = gauge_scale_i32(GAUGE_NEEDLE_INNER_R_BASE);
    let outer_r = pointer_outer_radius(inner_r);
    let start = polar_point(GAUGE_CENTER, inner_r, angle_deg);
    let end = polar_point(GAUGE_CENTER, outer_r, angle_deg);
    let needle_w = gauge_scale_i32_nonzero(GAUGE_NEEDLE_W_BASE) as u32;

    let _ = Line::new(start, end)
        .into_styled(PrimitiveStyle::with_stroke(GAUGE_NEEDLE_COLOR, needle_w))
        .draw(display);

    let _ = Circle::with_center(GAUGE_CENTER, gauge_scale_u32_nonzero(GAUGE_HUB_D_BASE))
        .into_styled(PrimitiveStyle::with_fill(GAUGE_HUB_COLOR))
        .draw(display);
}

fn erase_status_needle_fast<D>(display: &mut D, angle_deg: f32)
where
    D: DrawTarget<Color = DisplayColor>,
{
    let inner_r = gauge_scale_i32(GAUGE_NEEDLE_INNER_R_BASE);
    let outer_r = pointer_outer_radius(inner_r);
    let start = polar_point(GAUGE_CENTER, inner_r, angle_deg);
    let end = polar_point(GAUGE_CENTER, outer_r, angle_deg);

    let clear_w = (gauge_scale_i32_nonzero(GAUGE_NEEDLE_W_BASE) + 2)
        .max(gauge_scale_i32_nonzero(GAUGE_NEEDLE_CLEAR_W_BASE)) as u32;
    let _ = Line::new(start, end)
        .into_styled(PrimitiveStyle::with_stroke(BG_COLOR, clear_w))
        .draw(display);

    let _ = Circle::with_center(
        GAUGE_CENTER,
        gauge_scale_u32_nonzero(GAUGE_HUB_CLEAR_D_BASE),
    )
    .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
    .draw(display);
}

fn status_ratio(pm25: u16) -> f32 {
    ratio_from_pm25(pm25)
}

fn gauge_angle(ratio: f32) -> f32 {
    GAUGE_START_DEG + ratio.clamp(0.0, 1.0) * GAUGE_TOTAL_SWEEP_DEG
}

fn gauge_scale_i32(base: i32) -> i32 {
    ((base * GAUGE_DIAMETER as i32) + (GAUGE_REF_DIAMETER / 2)) / GAUGE_REF_DIAMETER
}

fn gauge_scale_i32_nonzero(base: i32) -> i32 {
    gauge_scale_i32(base).max(1)
}

fn gauge_scale_u32_nonzero(base: u32) -> u32 {
    let scaled =
        ((base as i32 * GAUGE_DIAMETER as i32) + (GAUGE_REF_DIAMETER / 2)) / GAUGE_REF_DIAMETER;
    scaled.max(1) as u32
}

fn pointer_outer_radius(inner_r: i32) -> i32 {
    let gauge_r = (GAUGE_DIAMETER as i32) / 2;
    let min_outer = inner_r + 1;
    let base_outer = gauge_scale_i32(GAUGE_NEEDLE_OUTER_R_BASE);
    let desired_outer = round_to_i32(base_outer as f32 * GAUGE_POINTER_LENGTH_FACTOR.max(0.1));
    let max_outer = gauge_r + gauge_scale_i32(GAUGE_POINTER_MAX_EXTRA_R_BASE);

    desired_outer.clamp(min_outer, max_outer.max(min_outer))
}

fn restore_gauge_slice<D>(display: &mut D, angle_deg: f32)
where
    D: DrawTarget<Color = DisplayColor>,
{
    let dir = if GAUGE_TOTAL_SWEEP_DEG >= 0.0 {
        1.0
    } else {
        -1.0
    };
    let sweep = GAUGE_RESTORE_SPAN_DEG * dir;
    let start = angle_deg - (sweep * 0.5);
    draw_gauge_gradient_span(display, start, sweep, GAUGE_GRADIENT_STEP_DEG_RESTORE);
}

fn angle_to_ratio(angle_deg: f32) -> f32 {
    if GAUGE_TOTAL_SWEEP_DEG.abs() < f32::EPSILON {
        0.0
    } else {
        ((angle_deg - GAUGE_START_DEG) / GAUGE_TOTAL_SWEEP_DEG).clamp(0.0, 1.0)
    }
}

fn draw_gauge_gradient_span<D>(display: &mut D, start_deg: f32, sweep_deg: f32, step_deg: f32)
where
    D: DrawTarget<Color = DisplayColor>,
{
    let total_sweep = sweep_deg.abs();
    if total_sweep <= f32::EPSILON {
        return;
    }

    let dir = if sweep_deg >= 0.0 { 1.0 } else { -1.0 };
    let step = step_deg.max(0.25);
    let mut walked = 0.0f32;

    while walked < total_sweep {
        let chunk = (total_sweep - walked).min(step);
        let chunk_start = start_deg + walked * dir;
        let chunk_mid = chunk_start + (chunk * 0.5) * dir;
        let color = gauge_gradient_color(angle_to_ratio(chunk_mid));
        draw_arc_band(display, chunk_start, chunk * dir, color);
        walked += chunk;
    }
}

fn polar_point(center: Point, radius: i32, angle_deg: f32) -> Point {
    let rad = angle_deg * (core::f32::consts::PI / 180.0);
    let x = center.x + round_to_i32((radius as f32) * rad.cos());
    let y = center.y + round_to_i32((radius as f32) * rad.sin());
    Point::new(x, y)
}

fn needle_geometry(
    angle_deg: f32,
    inner_r: i32,
    outer_r: i32,
    arrow_len: i32,
    arrow_half_w: i32,
    arrow_tip_offset: i32,
) -> (Point, Point, Point, Point, Point) {
    let start = polar_point(GAUGE_CENTER, inner_r, angle_deg);
    let base = polar_point(GAUGE_CENTER, outer_r - arrow_len, angle_deg);
    let tip = polar_point(GAUGE_CENTER, outer_r + arrow_tip_offset, angle_deg);
    let left = polar_point(base, arrow_half_w, angle_deg + 90.0);
    let right = polar_point(base, arrow_half_w, angle_deg - 90.0);
    (start, base, tip, left, right)
}

fn draw_capsule_aa<D>(display: &mut D, start: Point, end: Point, width: f32, color: DisplayColor)
where
    D: DrawTarget<Color = DisplayColor>,
{
    let half_w = (width * 0.5).max(0.5);
    let pad = round_to_i32(half_w + 1.0);
    let min_x = start.x.min(end.x) - pad;
    let max_x = start.x.max(end.x) + pad;
    let min_y = start.y.min(end.y) - pad;
    let max_y = start.y.max(end.y) + pad;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let alpha = capsule_alpha(x as f32 + 0.5, y as f32 + 0.5, start, end, half_w);
            if alpha <= 0.0 {
                continue;
            }

            let a = round_to_i32((alpha * 255.0).clamp(0.0, 255.0)) as u8;
            draw_pixel_safe(display, Point::new(x, y), scale_color(color, a));
        }
    }
}

fn capsule_alpha(px: f32, py: f32, start: Point, end: Point, half_w: f32) -> f32 {
    let dist = point_segment_distance(px, py, start, end);
    (half_w + 0.5 - dist).clamp(0.0, 1.0)
}

fn point_segment_distance(px: f32, py: f32, start: Point, end: Point) -> f32 {
    let ax = start.x as f32;
    let ay = start.y as f32;
    let bx = end.x as f32;
    let by = end.y as f32;
    let vx = bx - ax;
    let vy = by - ay;
    let len2 = vx * vx + vy * vy;

    if len2 <= 0.0001 {
        let dx = px - ax;
        let dy = py - ay;
        return (dx * dx + dy * dy).sqrt();
    }

    let t = (((px - ax) * vx + (py - ay) * vy) / len2).clamp(0.0, 1.0);
    let cx = ax + t * vx;
    let cy = ay + t * vy;
    let dx = px - cx;
    let dy = py - cy;
    (dx * dx + dy * dy).sqrt()
}

fn fill_triangle_aa<D>(display: &mut D, a: Point, b: Point, c: Point, color: DisplayColor)
where
    D: DrawTarget<Color = DisplayColor>,
{
    let min_x = a.x.min(b.x).min(c.x);
    let max_x = a.x.max(b.x).max(c.x);
    let min_y = a.y.min(b.y).min(c.y);
    let max_y = a.y.max(b.y).max(c.y);
    let sample_offsets = [0.25_f32, 0.75_f32];

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let mut covered = 0u8;
            for sy in sample_offsets {
                for sx in sample_offsets {
                    if point_in_triangle(x as f32 + sx, y as f32 + sy, a, b, c) {
                        covered += 1;
                    }
                }
            }

            if covered == 0 {
                continue;
            }

            let alpha = (covered as u16 * 255 / 4) as u8;
            draw_pixel_safe(display, Point::new(x, y), scale_color(color, alpha));
        }
    }
}

fn point_in_triangle(px: f32, py: f32, a: Point, b: Point, c: Point) -> bool {
    let w0 = edge_fn(b, c, px, py);
    let w1 = edge_fn(c, a, px, py);
    let w2 = edge_fn(a, b, px, py);
    let has_neg = w0 < 0.0 || w1 < 0.0 || w2 < 0.0;
    let has_pos = w0 > 0.0 || w1 > 0.0 || w2 > 0.0;
    !(has_neg && has_pos)
}

fn edge_fn(a: Point, b: Point, px: f32, py: f32) -> f32 {
    let ax = a.x as f32;
    let ay = a.y as f32;
    let bx = b.x as f32;
    let by = b.y as f32;
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

fn round_to_i32(v: f32) -> i32 {
    if v >= 0.0 {
        (v + 0.5) as i32
    } else {
        (v - 0.5) as i32
    }
}

fn gauge_gradient_color(ratio: f32) -> DisplayColor {
    let mut accum = 0.0f32;
    let total = gauge_total_span_deg();
    let target = ratio.clamp(0.0, 1.0) * total;
    let blend_half = (GAUGE_COLOR_BLEND_SPAN_DEG * 0.5).max(0.1);

    for (idx, seg) in GAUGE_SEGMENTS.iter().enumerate() {
        let seg_span = seg.sweep_deg.abs();
        let seg_end = accum + seg_span;

        if idx + 1 < GAUGE_SEGMENTS.len() {
            let blend_start = (seg_end - blend_half).max(accum);
            let blend_end = (seg_end + blend_half).min(total);

            if target >= blend_start && target <= blend_end {
                let next = GAUGE_SEGMENTS[idx + 1].color;
                let denom = (blend_end - blend_start).max(0.001);
                let t = ((target - blend_start) / denom).clamp(0.0, 1.0);
                return lerp_color(seg.color, next, t);
            }
        }

        if target <= seg_end {
            return seg.color;
        }

        accum = seg_end;
    }

    RED
}

fn gauge_total_span_deg() -> f32 {
    GAUGE_SEGMENTS
        .iter()
        .fold(0.0f32, |sum, seg| sum + seg.sweep_deg.abs())
        .max(1.0)
}

fn lerp_color(a: DisplayColor, b: DisplayColor, t: f32) -> DisplayColor {
    let clamped_t = t.clamp(0.0, 1.0);
    DisplayColor::new(
        lerp_u5(a.r(), b.r(), clamped_t),
        lerp_u6(a.g(), b.g(), clamped_t),
        lerp_u5(a.b(), b.b(), clamped_t),
    )
}

fn lerp_u5(a: u8, b: u8, t: f32) -> u8 {
    round_to_i32(a as f32 + (b as f32 - a as f32) * t).clamp(0, 31) as u8
}

fn lerp_u6(a: u8, b: u8, t: f32) -> u8 {
    round_to_i32(a as f32 + (b as f32 - a as f32) * t).clamp(0, 63) as u8
}
