use plotters::{prelude::*, series};
use std::collections::BTreeMap;
use std::fs;

// plot audiogram result
pub fn plot_audiogram(result: BTreeMap<String, BTreeMap<i32, f32>>, dir_path: &str) {
    // if dir_path is not exist, create dir
    if !fs::metadata(dir_path).is_ok() {
        fs::create_dir(dir_path).unwrap();
    }

    // get current YYYYMMDD_HHMMSS
    let now = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();

    // use now as filename and png
    let filename = format!("{}_audiogram.png", now);
    let path_str = format!("{}/{}", dir_path, filename);
    let root = BitMapBackend::new(&path_str, (960, 720)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    // result is like this:
    // {
    //    "L": {62: -24.0, 125: -24.0, 250: -24.0, 500: -24.0, 1000: -24.0, 1500: -24.0, 2000: -24.0, 3000: -24.0, 4000: -24.0, 6000: -24.0, 8000: -24.0, 10000: -24.0, 12000: -24.0},
    //    "R": {62: -24.0, 125: -24.0, 250: -24.0, 500: -24.0, 1000: -24.0, 1500: -24.0, 2000: -24.0, 3000: -24.0, 4000: -24.0, 6000: -24.0, 8000: -24.0, 10000: -24.0, 12000: -24.0}
    //}

    let mut chart_builder = ChartBuilder::on(&root);
    let mut chart_context = chart_builder
        .caption(&filename, ("sans-serif", 15).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d((20.0..20000.0).log_scale(), 0.0..100.0)
        .unwrap();

    // draw x axis
    chart_context
        .configure_mesh()
        .x_desc("Frequency [Hz]")
        .y_desc("Volume [dB]")
        .axis_desc_style(("sans-serif", 15).into_font())
        .x_labels(10)
        .y_labels(10)
        .draw()
        .unwrap();

    // plot L
    chart_context
        .draw_series(series::LineSeries::new(
            result
                .get("L")
                .unwrap()
                .iter()
                .map(|(x, y)| (*x as f32, (*y).abs() as f64)),
            &BLUE,
        ))
        .unwrap()
        .label("L")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
    chart_context
        .draw_series(
            result
                .get("L")
                .unwrap()
                .iter()
                .map(|(x, y)| Circle::new((*x as f32, (*y).abs() as f64), 3, BLUE.filled())),
        )
        .unwrap();

    // plot R
    chart_context
        .draw_series(series::LineSeries::new(
            result
                .get("R")
                .unwrap()
                .iter()
                .map(|(x, y)| (*x as f32, (*y).abs() as f64)),
            &RED,
        ))
        .unwrap()
        .label("R")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    chart_context
        .draw_series(
            result
                .get("R")
                .unwrap()
                .iter()
                .map(|(x, y)| Circle::new((*x as f32, (*y).abs() as f64), 3, RED.filled())),
        )
        .unwrap();

    // labels
    chart_context
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
}
