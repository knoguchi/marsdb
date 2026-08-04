//! Shared plotting helpers for the `marsdb` examples. Lives in a
//! subdirectory (not directly under `examples/`) so cargo doesn't try to
//! build it as its own example -- each example pulls it in with
//! `#[path = "common/viz.rs"] mod viz;`.
//!
//! SVG output, not PNG: plotters' bitmap backend needs a rasterized font
//! (either the system's, via fontconfig -- absent on the CI runner -- or a
//! bundled font file). SVG text is just `<text>` elements the browser lays
//! out, so there's no font dependency at all.

use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};

fn centered(size: u32, color: &'static RGBColor) -> TextStyle<'static> {
    TextStyle::from(("sans-serif", size).into_font())
        .color(color)
        .pos(Pos::new(HPos::Center, VPos::Center))
}

/// A simple categorical bar chart: one bar per `(label, value, color)`.
#[allow(dead_code)]
pub fn bar_chart(
    path: &str,
    title: &str,
    y_desc: &str,
    bars: &[(&str, f64, RGBColor)],
) -> Result<(), Box<dyn std::error::Error>> {
    let root = SVGBackend::new(path, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let max = bars
        .iter()
        .map(|(_, v, _)| *v)
        .fold(0.0_f64, f64::max)
        .max(1.0);
    let n = bars.len() as f64;
    let y_top = max * 1.15;
    let y_bottom = -(max * 0.08);

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 24))
        .margin(20)
        .x_label_area_size(0)
        .y_label_area_size(50)
        .build_cartesian_2d(0.0..n, y_bottom..y_top)?;
    chart
        .configure_mesh()
        .disable_x_mesh()
        .x_labels(0)
        .y_desc(y_desc)
        .draw()?;

    chart.draw_series(bars.iter().enumerate().map(|(i, (_, v, color))| {
        Rectangle::new(
            [(i as f64 + 0.12, 0.0), (i as f64 + 0.88, *v)],
            color.filled(),
        )
    }))?;

    chart.draw_series(bars.iter().enumerate().map(|(i, (label, _, _))| {
        Text::new(
            label.to_string(),
            (i as f64 + 0.5, y_bottom * 0.5),
            centered(16, &BLACK),
        )
    }))?;

    chart.draw_series(bars.iter().enumerate().map(|(i, (_, v, _))| {
        Text::new(
            format!("{v:.0}"),
            (i as f64 + 0.5, *v + max * 0.06),
            centered(14, &BLACK),
        )
    }))?;

    root.present()?;
    Ok(())
}

/// A directed-graph drawing: `nodes` are labels, `edges` are `(from, to)`
/// indices into `nodes`, laid out on a circle (fine for the handful of
/// nodes these examples have -- not a general graph-layout algorithm).
#[allow(dead_code)]
pub fn graph_viz(
    path: &str,
    title: &str,
    nodes: &[&str],
    edges: &[(usize, usize)],
) -> Result<(), Box<dyn std::error::Error>> {
    let root = SVGBackend::new(path, (640, 640)).into_drawing_area();
    root.fill(&WHITE)?;
    root.draw(&Text::new(title, (320, 20), centered(24, &BLACK)))?;

    let center = (320.0_f64, 340.0_f64);
    let radius = 220.0_f64;
    let n = nodes.len().max(1) as f64;
    let pos: Vec<(f64, f64)> = (0..nodes.len())
        .map(|i| {
            let angle = std::f64::consts::TAU * (i as f64) / n - std::f64::consts::FRAC_PI_2;
            (
                center.0 + radius * angle.cos(),
                center.1 + radius * angle.sin(),
            )
        })
        .collect();
    let px = |p: (f64, f64)| -> (i32, i32) { (p.0.round() as i32, p.1.round() as i32) };

    for &(from, to) in edges {
        let (x0, y0) = pos[from];
        let (x1, y1) = pos[to];
        // Stop short of the target node's circle so the line doesn't run
        // under the arrowhead/label, then draw a small arrowhead there.
        let (dx, dy) = (x1 - x0, y1 - y0);
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let (ux, uy) = (dx / len, dy / len);
        let node_r = 22.0;
        let tip = (x1 - ux * node_r, y1 - uy * node_r);
        root.draw(&PathElement::new(
            vec![px((x0, y0)), px(tip)],
            BLACK.mix(0.6).stroke_width(2),
        ))?;
        let back = 10.0;
        let spread = 6.0;
        let base = (tip.0 - ux * back, tip.1 - uy * back);
        let (nx, ny) = (-uy, ux);
        root.draw(&Polygon::new(
            vec![
                px(tip),
                px((base.0 + nx * spread, base.1 + ny * spread)),
                px((base.0 - nx * spread, base.1 - ny * spread)),
            ],
            BLACK.mix(0.6).filled(),
        ))?;
    }

    for (i, label) in nodes.iter().enumerate() {
        let center = px(pos[i]);
        root.draw(&Circle::new(center, 20, BLUE.mix(0.85).filled()))?;
        root.draw(&Text::new(label.to_string(), center, centered(14, &WHITE)))?;
    }

    root.present()?;
    Ok(())
}
