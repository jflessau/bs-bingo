use crate::error::{Error, Result};

pub mod access;
pub mod field;
pub mod player;
pub mod websocket;

#[derive(Debug, Eq, PartialEq)]
enum Direction {
    Right,
    Down,
    RightDown,
    RightUp,
}

fn bingos(fields: Vec<bool>) -> i32 {
    match grid_size(fields.len()) {
        Ok(grid_size) => {
            let directions = vec![
                Direction::Right,
                Direction::Down,
                Direction::RightDown,
                Direction::RightUp,
            ];

            let mut border_fields = vec![];
            for n in 0..fields.len() - 1 {
                if n < grid_size || n % grid_size == 0 {
                    border_fields.push(n);
                }
            }

            let mut bingo_amount = 0;
            for direction in &directions {
                for border_field in &border_fields {
                    if walk(
                        &fields,
                        (*border_field) as i32,
                        (*border_field) as i32,
                        direction,
                        0,
                        0,
                    ) == grid_size
                    {
                        bingo_amount += 1
                    }
                }
            }

            bingo_amount
        }
        Err(_) => {
            tracing::error!("invalid field amount: {}", fields.len());
            0
        }
    }
}

fn walk(
    fields: &Vec<bool>,
    start_idx: i32,
    idx: i32,
    direction: &Direction,
    mut hits: usize,
    mut n: usize,
) -> usize {
    n += 1;
    let grid_size = grid_size(fields.len()).expect("grid_size fails") as i32;

    if idx > (fields.len() - 1) as i32 || idx < 0 {
        return hits;
    }

    if direction == &Direction::Right && (idx != start_idx && idx % grid_size == 0) {
        return hits;
    }

    if fields[idx as usize] {
        hits += 1;
    };

    if n as i32 == grid_size || n > 1000 {
        return hits;
    }

    let next_idx = match direction {
        Direction::Right => idx + 1,
        Direction::Down => idx + grid_size,
        Direction::RightDown => idx + grid_size + 1,
        Direction::RightUp => {
            if idx - (grid_size - 1) != 0 {
                idx - (grid_size - 1)
            } else {
                -1
            }
        }
    };

    walk(fields, start_idx, next_idx, direction, hits, n)
}

fn grid_size(field_amount: usize) -> Result<usize> {
    let grid_size = (field_amount as f32).sqrt();

    if !(2..10).into_iter().any(|v| v as f32 == grid_size) {
        return Err(Error::InternalServer);
    }

    Ok(grid_size as usize)
}

#[test]
fn test_grid_size() {
    assert!(grid_size(0).is_err());
    assert!(grid_size(1).is_err());
    assert!(grid_size(2).is_err());
    assert!(grid_size(3).is_err());
    assert_eq!(grid_size(4).unwrap(), 2);
    assert!(grid_size(5).is_err());
    assert!(grid_size(6).is_err());
    assert!(grid_size(7).is_err());
    assert!(grid_size(8).is_err());
    assert_eq!(grid_size(9).unwrap(), 3);
    assert!(grid_size(10).is_err());
}

#[test]
fn test_bingos() {
    // invalid field amounts

    assert_eq!(bingos(vec![]), 0);
    assert_eq!(bingos(vec![false]), 0);
    assert_eq!(bingos(vec![false, false]), 0);
    assert_eq!(bingos(vec![false, false, false]), 0);
    assert_eq!(bingos(vec![false, false, false, false, false]), 0);
    assert_eq!(bingos(vec![false, false, false, false, false, false]), 0);
    assert_eq!(
        bingos(vec![false, false, false, false, false, false, false]),
        0
    );
    assert_eq!(
        bingos(vec![false, false, false, false, false, false, false, false]),
        0
    );

    // 2x2

    let fields = vec![
        false, false, //
        false, false, //
    ];
    assert_eq!(bingos(fields), 0);

    let fields = vec![
        true, true, //
        false, false, //
    ];
    assert_eq!(bingos(fields), 1);

    let fields = vec![
        false, false, //
        true, true, //
    ];
    assert_eq!(bingos(fields), 1);

    let fields = vec![
        true, false, //
        false, true, //
    ];
    assert_eq!(bingos(fields), 1);

    let fields = vec![
        false, true, //
        true, false, //
    ];
    assert_eq!(bingos(fields), 1);

    let fields = vec![
        true, false, //
        true, false, //
    ];
    assert_eq!(bingos(fields), 1);

    let fields = vec![
        false, true, //
        false, true, //
    ];
    assert_eq!(bingos(fields), 1);

    let fields = vec![
        true, true, //
        true, true, //
    ];
    assert_eq!(bingos(fields), 6);

    // 3x3
    let fields = vec![
        true, true, true, //
        true, true, true, //
        true, true, true, //
    ];
    assert_eq!(bingos(fields), 8);

    // 4x4
    let fields = vec![
        true, true, true, true, //
        true, true, true, true, //
        true, true, true, true, //
        true, true, true, true, //
    ];
    assert_eq!(bingos(fields), 10);

    // 5x5

    let fields = vec![
        true, true, true, true, true, //
        true, true, true, true, true, //
        true, true, true, true, true, //
        true, true, true, true, true, //
        true, true, true, true, true, //
    ];
    assert_eq!(bingos(fields), 12);

    let fields = vec![
        true, true, true, true, true, //
        true, true, true, true, true, //
        true, true, true, true, true, //
        true, false, true, true, true, //
        true, true, true, true, true, //
    ];
    assert_eq!(bingos(fields), 9);

    let fields = vec![
        true, true, true, true, true, //
        true, true, true, true, true, //
        true, true, false, true, true, //
        true, true, true, true, true, //
        true, true, true, true, true, //
    ];
    assert_eq!(bingos(fields), 8);

    // 6 x 6

    let fields = vec![
        true, true, true, true, true, true, //
        true, true, true, true, true, true, //
        true, true, true, true, true, true, //
        true, true, true, true, true, true, //
        true, true, true, true, true, true, //
        true, true, true, true, true, true, //
    ];
    assert_eq!(bingos(fields), 14);
}
