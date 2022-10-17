use std::fmt::Debug;

use log::tracing::{self, trace};
use maud::{html, Markup};
use typed_builder::TypedBuilder;

pub type ChartValue<T> = (f64, i64, T);

const SVG_MIN_X: f64 = 0.0;
const SVG_MAX_X: f64 = 1000.0;
const SVG_MIN_Y: f64 = 0.0;
const SVG_MAX_Y: f64 = 300.0;
const SVG_Y_RATIO: f64 = SVG_MAX_Y - SVG_MIN_Y;

const ESTIMATED_ONE_CHAR_SIZE: f64 = 16.0;
const RESERVED_CHARACTERS: f64 = 7.0;
const CHART_MIN_X: f64 = SVG_MIN_X + (ESTIMATED_ONE_CHAR_SIZE * RESERVED_CHARACTERS);
const CHART_MAX_X: f64 = SVG_MAX_X;
const CHART_X_RATIO: f64 = CHART_MAX_X - CHART_MIN_X;
const CHART_MIN_Y: f64 = SVG_MIN_Y + (SVG_Y_RATIO * 0.05);
const CHART_MAX_Y: f64 = SVG_MAX_Y - (SVG_Y_RATIO * 0.05);
const CHART_Y_RATIO: f64 = CHART_MAX_Y - CHART_MIN_Y;

type ChartCollectionBuilder<'a, 'b, T> = (&'a str, Option<&'b str>, Vec<ChartValue<T>>);

#[derive(Debug, Default, TypedBuilder)]
pub struct ChartContext<T> {
    #[builder(setter(transform = |collections: Vec<ChartCollectionBuilder<'_, '_, T>>| collections.into_iter().map(|(color, label, values)| (color.into(), label.map(|l| l.into()), values)).collect::<Vec<_>>()))]
    pub collections: Vec<(String, Option<String>, Vec<ChartValue<T>>)>,
    #[builder(setter(strip_option), default = None)]
    pub max_value: Option<f64>,
    #[builder(default = "%".to_string(), setter(into))]
    pub unit: String,
}

// Because the viewBox in SVG invert the values (top left corner is 0,0)
#[tracing::instrument]
fn svg_value_invert(value: f64, max: f64, min: f64) -> f64 {
    let result = ((value - min) + (max - min) * -1f64) * -1f64 + min;
    trace!(result);
    result
}

#[tracing::instrument(skip(raw_values))]
fn values_to_polyline<T: Debug>(
    raw_values: &[ChartValue<T>],
    (min_value_range, max_value_range): (f64, f64),
) -> Option<String> {
    if raw_values.is_empty() {
        return None;
    };

    let value_ratio = max_value_range - min_value_range;

    let first_date = raw_values.first().map(|(_, date, _)| date).unwrap();
    let last_date = raw_values.last().map(|(_, date, _)| date).unwrap();
    let date_ratio = (last_date - first_date) as f64;
    trace!(
        first_date_timestamp = first_date,
        last_date_timestamp = last_date
    );

    let values = raw_values
        .iter()
        .map(|(val, date, _)| {
            trace!(
                date,
                first_date,
                top = date - first_date,
                bottom = date_ratio * CHART_X_RATIO + CHART_MIN_X
            );
            format!(
                "{},{}",
                ((date - first_date) as f64 / date_ratio * CHART_X_RATIO + CHART_MIN_X).round(),
                svg_value_invert(
                    ((val - min_value_range) / value_ratio * CHART_Y_RATIO + CHART_MIN_Y).round(),
                    CHART_MAX_Y,
                    CHART_MIN_Y
                ),
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    trace!(svg_values = values);

    Some(values)
}

fn round_to_len(value: f64, len: usize) -> f64 {
    (value * 10f64.powi(len as i32)).round() / 10f64.powi(len as i32)
}

#[tracing::instrument(skip(ctx), fields(unit = ctx.unit))]
pub fn Chart<T: Debug>(ctx: ChartContext<T>) -> Markup {
    if ctx.collections.is_empty() {
        html! {
            p { "No data available." }
        }
    } else {
        let max_value = ctx.max_value.unwrap_or_else(|| {
            ctx.collections
                .iter()
                .flat_map(|(_, _, values)| values.iter().map(|(val, _, _)| val))
                .fold(0f64, |max, x| max.max(*x))
        });
        trace!(max_value);
        let mid_value = max_value / 2f64;
        let collections = ctx
            .collections
            .iter()
            .filter_map(|(color, _label, values)| {
                values_to_polyline(values, (0f64, max_value))
                    .map(|polyine| (color, polyine, max_value))
            })
            .collect::<Vec<_>>();

        let labels_offset = CHART_MIN_X - (ESTIMATED_ONE_CHAR_SIZE * 0.5) as f64;
        html! {
            svg.chart viewBox=(format!("{SVG_MIN_X} {SVG_MIN_Y} {SVG_MAX_X} {SVG_MAX_Y}")) {
                g.grid.x-grid {
                    line x1=(CHART_MIN_X) y1="5%" x2="100%" y2="5%" {}
                    line x1=(CHART_MIN_X) y1="50%" x2="100%" y2="50%" {}
                    line x1=(CHART_MIN_X) y1="95%" x2="100%" y2="95%" {}
                }
                g.labels.x-labels {
                    text x=(labels_offset) y="5%" dy="6" { (format!("{}{}", round_to_len(max_value, 2), ctx.unit)) }
                    text x=(labels_offset) y="50%" dy="6" { (format!("{}{}", round_to_len(mid_value, 2), ctx.unit)) }
                    text x=(labels_offset) y="95%" dy="6" { (format!("0{}", ctx.unit)) }
                }
                g.lines {
                    @for (color, polyline, _) in collections {
                        polyline.dataline fill="none" stroke=(color) stroke-width="2" points=(polyline) {}
                    }
                }
            }
        }
    }
}
