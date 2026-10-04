use pop_pop_win::game::{Field, Game, GameState, SquareState};
use rand::Rng;

// Grid representation:
// XXXXX2
// X7X8X3
// X5XXX2
// X32321
// 110000
// null/None = bomb (true), Some(n) = safe (false) with n adjacent bombs
const SAMPLE_FIELD: [Option<u8>; 30] = [
    None, None, None, None, None, Some(2), // row 0
    None, Some(7), None, Some(8), None, Some(3), // row 1
    None, Some(5), None, None, None, Some(2), // row 2
    None, Some(3), Some(2), Some(3), Some(2), Some(1), // row 3
    Some(1), Some(1), Some(0), Some(0), Some(0), Some(0), // row 4
];

fn get_sample_field() -> Field {
    let squares: Vec<bool> = SAMPLE_FIELD.iter().map(|&x| x.is_none()).collect();
    Field::from_squares(6, 5, squares)
}

#[test]
fn test_field_defaults() {
    let f = Field::new(16, 16, 40, None, None);
    assert_eq!(f.bomb_count, 40);
    assert_eq!(f.cols, 16);
    assert_eq!(f.rows, 16);
}

#[test]
fn test_field_bomb_count() {
    let f = Field::new(16, 16, 40, None, None);
    let mut bomb_count = 0;
    for x in 0..16 {
        for y in 0..16 {
            if f.get(x, y) {
                bomb_count += 1;
            }
        }
    }
    assert_eq!(bomb_count, f.bomb_count);
}

#[test]
fn test_field_from_squares() {
    let f = Field::from_squares(2, 2, vec![true, true, true, false]);
    assert_eq!(f.cols, 2);
    assert_eq!(f.rows, 2);
    assert_eq!(f.bomb_count, 3);
}

#[test]
fn test_field_adjacent() {
    let mut f = get_sample_field();
    assert_eq!(f.bomb_count, 13);

    for x in 0..f.cols {
        for y in 0..f.rows {
            let i = x + y * f.cols;
            let expected = SAMPLE_FIELD[i];
            if expected.is_none() {
                assert!(f.get(x, y));
            } else {
                assert_eq!(f.get_adjacent_count(x, y), expected.unwrap());
            }
        }
    }
}

#[test]
fn test_game_initial() {
    let f = get_sample_field();
    let g = Game::from_field(f);

    assert_eq!(g.bombs_left(), 13);
    assert_eq!(g.reveals_left(), 30 - 13);
    assert_eq!(g.state(), GameState::Reset);
    assert!(g.duration().is_none());

    for x in 0..g.width {
        for y in 0..g.height {
            assert_eq!(g.get_square_state(x, y), SquareState::Hidden);
        }
    }
}

#[test]
fn test_game_set_flag() {
    let mut g = Game::from_field(get_sample_field());

    assert_eq!(g.get_square_state(0, 0), SquareState::Hidden);
    g.set_flag(0, 0, true);
    assert_eq!(g.get_square_state(0, 0), SquareState::Flagged);
    assert_eq!(g.bombs_left(), 12);
    assert_eq!(g.state(), GameState::Started);
}

#[test]
#[should_panic]
fn test_cannot_reveal_flagged() {
    let mut g = Game::from_field(get_sample_field());
    g.set_flag(0, 0, true);
    let _ = g.reveal(0, 0);
}

#[test]
#[should_panic]
fn test_cannot_flag_revealed() {
    let mut g = Game::from_field(get_sample_field());
    g.reveal(1, 1);
    assert_eq!(g.get_square_state(1, 1), SquareState::Revealed);
    g.set_flag(1, 1, true);
}

#[test]
fn test_reveal_zero_cascade() {
    let f = get_sample_field();
    let start_reveals = f.len() - 13;
    let mut g = Game::from_field(f);

    assert_eq!(g.bombs_left(), 13);
    assert_eq!(g.reveals_left(), start_reveals);
    assert_eq!(g.state(), GameState::Reset);

    let reveals = g.reveal(5, 4).expect("reveal should succeed");
    // (5, 4) is 0, connected to several zeros and numbers
    assert_eq!(g.reveals_left(), start_reveals - 10);
    assert_eq!(reveals.len(), 10);
}

#[test]
fn test_loss() {
    let mut g = Game::from_field(get_sample_field());
    assert_eq!(g.get_square_state(0, 0), SquareState::Hidden);
    let revealed = g.reveal(0, 0);
    assert!(revealed.is_none());
    assert_eq!(g.state(), GameState::Lost);
    assert_eq!(g.get_square_state(0, 0), SquareState::Bomb);
}

#[test]
fn test_win() {
    let f = get_sample_field();
    let mut g = Game::from_field(f.clone());

    let mut bombs_left = f.bomb_count as i32;
    assert_eq!(g.reveals_left(), f.len() - 13);
    let mut reveals_left = g.reveals_left();

    for x in 0..f.cols {
        for y in 0..f.rows {
            if f.get(x, y) {
                g.set_flag(x, y, true);
                bombs_left -= 1;
                assert_eq!(g.bombs_left(), bombs_left);
            } else if g.get_square_state(x, y) == SquareState::Hidden {
                let r = g.reveal(x, y).expect("non-bomb reveal should succeed");
                reveals_left -= r.len();
                assert_eq!(reveals_left, g.reveals_left());
            } else {
                assert_eq!(g.get_square_state(x, y), SquareState::Revealed);
            }
            assert_ne!(g.state(), GameState::Reset);
            assert_ne!(g.state(), GameState::Lost);
        }
    }

    assert_eq!(g.state(), GameState::Won);
}

#[test]
fn test_good_chord() {
    let f = get_sample_field();
    let start_reveals = f.len() - 13;
    let mut g = Game::from_field(f);

    assert_eq!(g.bombs_left(), 13);
    assert_eq!(g.reveals_left(), start_reveals);
    assert_eq!(g.state(), GameState::Reset);

    g.reveal(2, 3);
    g.set_flag(2, 2, true);
    g.set_flag(3, 2, true);

    assert_eq!(g.bombs_left(), 11);
    assert_eq!(g.reveals_left(), start_reveals - 1);

    let chord_reveals = g.reveal(2, 3).expect("chord should succeed");
    assert_eq!(g.bombs_left(), 11);
    assert_eq!(g.reveals_left(), start_reveals - 11);
    assert!(!chord_reveals.is_empty());
    assert!(g.duration().is_some());
}

#[test]
fn test_bad_chord() {
    let f = get_sample_field();
    let start_reveals = f.len() - 13;
    let mut g = Game::from_field(f);

    g.reveal(2, 3);
    g.set_flag(1, 2, true); // Incorrect flag!
    g.set_flag(3, 2, true);

    assert_eq!(g.bombs_left(), 11);
    assert_eq!(g.reveals_left(), start_reveals - 1);

    let revealed = g.reveal(2, 3);
    assert!(revealed.is_none());
    assert_eq!(g.state(), GameState::Lost);
}

#[test]
#[should_panic]
fn test_noop_chord_panics_on_reveal() {
    let f = get_sample_field();
    let mut g = Game::from_field(f);

    g.reveal(2, 3);
    g.set_flag(2, 2, true); // Only 1 flag when 2 needed
    let _ = g.reveal(2, 3); // can_chord is false, can_reveal is false -> panics
}

#[test]
fn test_can_reveal_and_can_flag() {
    let f = get_sample_field();
    let mut g = Game::from_field(f);

    assert!(g.can_reveal(0, 0));
    assert_eq!(g.state(), GameState::Reset);
    g.set_flag(0, 0, true);
    assert_eq!(g.state(), GameState::Started);
    assert!(!g.can_reveal(0, 0));

    assert!(g.can_reveal(5, 4));
    g.reveal(5, 4);
    assert!(!g.can_reveal(5, 4));

    g.set_flag(4, 2, true);
    assert!(g.can_reveal(5, 3));
    assert!(!g.can_reveal(4, 3));
    g.set_flag(3, 2, true);
    assert!(g.can_reveal(4, 3));

    // Over-flag
    assert!(g.can_reveal(5, 3));
    g.set_flag(5, 2, true);
    assert!(!g.can_reveal(5, 3));
}

#[test]
fn test_safe_first_click() {
    for _ in 0..10 {
        for x in 0..8 {
            for y in 0..8 {
                let mut game = Game::new(8, 8, 10);
                let reveals = game.reveal(x, y);
                assert!(reveals.is_some(), "First click at ({}, {}) was a bomb!", x, y);
                assert_eq!(game.state(), GameState::Started);
                assert!(!game.field().unwrap().get(x, y));
            }
        }
    }
}

#[test]
fn test_random_fields_solvable() {
    let mut rng = rand::thread_rng();
    for _ in 0..5 {
        let f = Field::new(10, 10, 15, None, None);
        for _ in 0..5 {
            let mut g = Game::from_field(f.clone());
            while g.reveals_left() > 0 {
                let x = rng.gen_range(0..f.cols);
                let y = rng.gen_range(0..f.rows);
                if g.get_square_state(x, y) == SquareState::Hidden {
                    if f.get(x, y) {
                        g.set_flag(x, y, true);
                    } else {
                        g.reveal(x, y);
                    }
                }
            }
            assert_eq!(g.state(), GameState::Won);
        }
    }
}
