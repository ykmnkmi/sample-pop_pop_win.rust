use std::time::{Duration, Instant};

use super::field::Field;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SquareState {
    Hidden,
    Revealed,
    Flagged,
    Bomb,
    Safe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Reset,
    Started,
    Won,
    Lost,
}

#[derive(Debug, Clone)]
pub struct Game {
    pub width: usize,
    pub height: usize,
    pub bomb_count: usize,
    field: Option<Field>,
    states: Vec<SquareState>,
    state: GameState,
    bombs_left: i32,
    reveals_left: usize,
    start_time: Option<Instant>,
    stopped_duration: Option<Duration>,
}

impl Game {
    pub fn new(width: usize, height: usize, bomb_count: usize) -> Self {
        let total = width * height;
        Self {
            width,
            height,
            bomb_count,
            field: None,
            states: vec![SquareState::Hidden; total],
            state: GameState::Reset,
            bombs_left: bomb_count as i32,
            reveals_left: total - bomb_count,
            start_time: None,
            stopped_duration: None,
        }
    }

    pub fn from_field(field: Field) -> Self {
        let width = field.cols;
        let height = field.rows;
        let bomb_count = field.bomb_count;
        let total = field.len();
        Self {
            width,
            height,
            bomb_count,
            states: vec![SquareState::Hidden; total],
            state: GameState::Reset,
            bombs_left: bomb_count as i32,
            reveals_left: total - bomb_count,
            field: Some(field),
            start_time: None,
            stopped_duration: None,
        }
    }

    #[inline]
    pub fn state(&self) -> GameState {
        self.state
    }

    #[inline]
    pub fn bombs_left(&self) -> i32 {
        self.bombs_left
    }

    #[inline]
    pub fn reveals_left(&self) -> usize {
        self.reveals_left
    }

    #[inline]
    pub fn game_ended(&self) -> bool {
        self.state == GameState::Won || self.state == GameState::Lost
    }

    #[inline]
    pub fn index(&self, x: usize, y: usize) -> usize {
        assert!(x < self.width && y < self.height);
        y * self.width + x
    }

    #[inline]
    pub fn get_square_state(&self, x: usize, y: usize) -> SquareState {
        self.states[self.index(x, y)]
    }

    #[inline]
    pub fn get_square_state_by_index(&self, index: usize) -> SquareState {
        self.states[index]
    }

    pub fn field(&self) -> Option<&Field> {
        self.field.as_ref()
    }

    pub fn field_mut(&mut self) -> &mut Field {
        if self.field.is_none() {
            self.field = Some(Field::new(self.width, self.height, self.bomb_count, None, None));
        }
        self.field.as_mut().unwrap()
    }

    pub fn duration(&self) -> Option<Duration> {
        if let Some(stopped) = self.stopped_duration {
            Some(stopped)
        } else if let Some(start) = self.start_time {
            Some(start.elapsed())
        } else {
            None
        }
    }

    pub fn can_toggle_flag(&self, x: usize, y: usize) -> bool {
        let current = self.get_square_state(x, y);
        current == SquareState::Hidden || current == SquareState::Flagged
    }

    pub fn set_flag(&mut self, x: usize, y: usize, value: bool) {
        self.ensure_started(None);
        let idx = self.index(x, y);
        let current = self.states[idx];

        if value {
            assert_eq!(current, SquareState::Hidden, "Can only flag hidden squares");
            self.states[idx] = SquareState::Flagged;
            self.bombs_left -= 1;
        } else {
            assert_eq!(current, SquareState::Flagged, "Can only unflag flagged squares");
            self.states[idx] = SquareState::Hidden;
            self.bombs_left += 1;
        }
    }

    pub fn can_reveal(&mut self, x: usize, y: usize) -> bool {
        let current = self.get_square_state(x, y);
        if current == SquareState::Hidden {
            true
        } else if self.can_chord(x, y) {
            true
        } else {
            false
        }
    }

    pub fn reveal(&mut self, x: usize, y: usize) -> Option<Vec<(usize, usize)>> {
        self.ensure_started(Some((x, y)));
        assert!(self.can_reveal(x, y), "Item cannot be revealed.");

        let current = self.get_square_state(x, y);
        let mut reveals = None;

        if current == SquareState::Hidden {
            if self.field.as_ref().unwrap().get(x, y) {
                self.set_lost();
                reveals = Some(Vec::new());
            } else {
                let mut list = Vec::new();
                self.do_reveal(x, y, &mut list);
                reveals = Some(list);
            }
        } else if self.can_chord(x, y) {
            reveals = Some(self.do_chord(x, y));
        }

        if self.state == GameState::Lost {
            None
        } else {
            reveals
        }
    }

    fn can_chord(&mut self, x: usize, y: usize) -> bool {
        let current = self.get_square_state(x, y);
        if current == SquareState::Revealed {
            let adj_count = self.field_mut().get_adjacent_count(x, y);
            if adj_count > 0 {
                let adj_hidden = self.get_adjacent_count_by_state(x, y, SquareState::Hidden);
                if adj_hidden > 0 {
                    let adj_flags = self.get_adjacent_count_by_state(x, y, SquareState::Flagged);
                    if adj_flags == adj_count {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn do_chord(&mut self, x: usize, y: usize) -> Vec<(usize, usize)> {
        assert_eq!(self.get_square_state(x, y), SquareState::Revealed);
        let adj_count = self.field_mut().get_adjacent_count(x, y);
        assert!(adj_count > 0);

        let adj_indices = self.field.as_ref().unwrap().get_adjacent_indices(x, y);
        let mut hidden = Vec::new();
        let mut flagged = Vec::new();
        let mut failed = false;

        for &i in &adj_indices {
            let s = self.states[i];
            if s == SquareState::Hidden {
                hidden.push(i);
                if self.field.as_ref().unwrap().get_by_index(i) {
                    failed = true;
                }
            } else if s == SquareState::Flagged {
                flagged.push(i);
            }
        }

        assert_eq!(flagged.len() as u8, adj_count);
        let mut reveals = Vec::new();

        if failed {
            self.set_lost();
        } else {
            for i in hidden {
                let (cx, cy) = self.field.as_ref().unwrap().coordinate(i);
                if self.can_reveal(cx, cy) {
                    if let Some(r) = self.reveal(cx, cy) {
                        reveals.extend(r);
                    }
                }
            }
        }

        reveals
    }

    fn do_reveal(&mut self, x: usize, y: usize, acc: &mut Vec<(usize, usize)>) {
        let idx = self.index(x, y);
        assert_eq!(self.states[idx], SquareState::Hidden);
        self.states[idx] = SquareState::Revealed;
        self.reveals_left -= 1;
        acc.push((x, y));

        if self.reveals_left == 0 {
            self.set_won();
        } else if self.field_mut().get_adjacent_count(x, y) == 0 {
            let adj_indices = self.field.as_ref().unwrap().get_adjacent_indices(x, y);
            for i in adj_indices {
                if self.states[i] == SquareState::Hidden {
                    let (cx, cy) = self.field.as_ref().unwrap().coordinate(i);
                    self.do_reveal(cx, cy, acc);
                    assert!(self.state == GameState::Started || self.state == GameState::Won);
                }
            }
        }
    }

    fn set_won(&mut self) {
        assert_eq!(self.state, GameState::Started);
        let field = self.field.as_ref().unwrap();
        for i in 0..field.len() {
            if field.get_by_index(i) {
                self.states[i] = SquareState::Safe;
            }
        }
        self.set_state(GameState::Won);
    }

    fn set_lost(&mut self) {
        assert_eq!(self.state, GameState::Started);
        let field = self.field.as_ref().unwrap();
        for i in 0..field.len() {
            if field.get_by_index(i) {
                self.states[i] = SquareState::Bomb;
            }
        }
        self.set_state(GameState::Lost);
    }

    fn set_state(&mut self, new_state: GameState) {
        if self.state != new_state {
            self.state = new_state;
            if self.state == GameState::Started {
                self.start_time = Some(Instant::now());
                self.stopped_duration = None;
            } else if self.game_ended() {
                if let Some(start) = self.start_time {
                    self.stopped_duration = Some(start.elapsed());
                }
            }
        }
    }

    pub fn ensure_started(&mut self, first_click: Option<(usize, usize)>) {
        if self.state == GameState::Reset {
            if self.field.is_none() {
                self.field = Some(Field::new(
                    self.width,
                    self.height,
                    self.bomb_count,
                    first_click,
                    None,
                ));
            }
            self.set_state(GameState::Started);
        }
        assert_eq!(self.state, GameState::Started);
    }

    fn get_adjacent_count_by_state(&self, x: usize, y: usize, state: SquareState) -> u8 {
        let mut count = 0u8;
        let adj_indices = self.field.as_ref().unwrap().get_adjacent_indices(x, y);
        for i in adj_indices {
            if self.states[i] == state {
                count += 1;
            }
        }
        count
    }

    pub fn to_board_string(&mut self) -> String {
        let mut s = String::new();
        let w = self.width as isize;
        let h = self.height as isize;

        for y in -2..h {
            if y > -2 {
                s.push('\n');
            }
            for x in -2..w {
                if y == -2 {
                    if x == -2 {
                        s.push(' ');
                    } else if x == -1 {
                        s.push('|');
                    } else {
                        s.push_str(&(x % 10).to_string());
                    }
                } else if y == -1 {
                    if x == -1 {
                        s.push('+');
                    } else {
                        s.push('-');
                    }
                } else {
                    if x == -2 {
                        s.push_str(&(y % 10).to_string());
                    } else if x == -1 {
                        s.push('|');
                    } else {
                        let ux = x as usize;
                        let uy = y as usize;
                        let ch = match self.get_square_state(ux, uy) {
                            SquareState::Flagged => 'F',
                            SquareState::Revealed => {
                                let c = self.field_mut().get_adjacent_count(ux, uy);
                                char::from_digit(c as u32, 10).unwrap()
                            }
                            SquareState::Hidden => '?',
                            SquareState::Bomb => 'B',
                            SquareState::Safe => 'S',
                        };
                        s.push(ch);
                    }
                }
            }
        }
        s
    }
}
