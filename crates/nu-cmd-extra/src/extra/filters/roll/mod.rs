mod roll_;
mod roll_down;
mod roll_left;
mod roll_right;
mod roll_up;

use nu_protocol::{ShellError, Value};

pub use roll_::Roll;
pub use roll_down::RollDown;
pub use roll_left::RollLeft;
pub use roll_right::RollRight;
pub use roll_up::RollUp;

enum VerticalDirection {
    Up,
    Down,
}

// This is a reimplementation of rotate_left and rotate_right's methods from Rust's Vec,
// hopefully not losing too much performance.
fn rotate_im_vector<T: Clone>(v: &im::Vector<T>, by: Option<usize>, direction: VerticalDirection) -> im::Vector<T> {
    let rotations = by.map(|n| n % v.len()).unwrap_or(1);
    let mut scratch = v.clone();

    match direction {
        VerticalDirection::Up => {
            let suffix = scratch.slice(0..rotations);
            scratch.append(suffix);

            scratch
        }
        VerticalDirection::Down => {
            let mut prefix = scratch.slice((v.len() - rotations)..);
            prefix.append(scratch);

            prefix
        }
    }
}

fn vertical_rotate_value(
    value: Value,
    by: Option<usize>,
    direction: VerticalDirection,
) -> Result<Value, ShellError> {
    let span = value.span();
    match value {
        Value::List { vals, .. } => {
            Ok(Value::list(rotate_im_vector(&vals, by, direction), span))
        }
        _ => Err(ShellError::TypeMismatch {
            err_message: "list".to_string(),
            span: value.span(),
        }),
    }
}

enum HorizontalDirection {
    Left,
    Right,
}

fn horizontal_rotate_value(
    value: Value,
    by: Option<usize>,
    cells_only: bool,
    direction: &HorizontalDirection,
) -> Result<Value, ShellError> {
    let span = value.span();
    match value {
        Value::Record { val: record, .. } => {
            let rotations = by.map(|n| n % record.len()).unwrap_or(1);

            let (mut cols, mut vals): (Vec<_>, Vec<_>) = record.into_owned().into_iter().unzip();
            if !cells_only {
                match direction {
                    HorizontalDirection::Right => cols.rotate_right(rotations),
                    HorizontalDirection::Left => cols.rotate_left(rotations),
                }
            };

            match direction {
                HorizontalDirection::Right => vals.rotate_right(rotations),
                HorizontalDirection::Left => vals.rotate_left(rotations),
            }

            let record = cols.into_iter().zip(vals).collect();
            Ok(Value::record(record, span))
        }
        Value::List { vals, .. } => {
            let values = vals
                .into_iter()
                .map(|value| horizontal_rotate_value(value, by, cells_only, direction))
                .collect::<Result<im::Vector<Value>, ShellError>>()?;

            Ok(Value::list(values, span))
        }
        _ => Err(ShellError::TypeMismatch {
            err_message: "record".to_string(),
            span: value.span(),
        }),
    }
}
