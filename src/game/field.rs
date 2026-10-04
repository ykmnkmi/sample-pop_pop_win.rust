use rand::seq::SliceRandom;
use rand::{rngs::StdRng, SeedableRng};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub cols: usize,
    pub rows: usize,
    pub bomb_count: usize,
    bombs: Vec<bool>,
    adjacents: Vec<Option<u8>>,
}

impl Field {
    /// Creates a field with a guaranteed safe position at (no_bomb.0, no_bomb.1).
    /// This ensures the first click in Minesweeper is never a bomb.
    pub fn new(
        cols: usize,
        rows: usize,
        bomb_count: usize,
        no_bomb: Option<(usize, usize)>,
        seed: Option<u64>,
    ) -> Self {
        assert!(cols > 0, "cols must be > 0");
        assert!(rows > 0, "rows must be > 0");
        let total = cols * rows;
        assert!(bomb_count > 0, "bomb_count must be > 0");
        assert!(bomb_count < total, "bomb_count must be < cols * rows");

        if let Some((nx, ny)) = no_bomb {
            assert!(nx < cols, "safe x out of bounds");
            assert!(ny < rows, "safe y out of bounds");
        }

        let safe_index = no_bomb.map(|(nx, ny)| ny * cols + nx);
        let mut available_positions: Vec<usize> = (0..total)
            .filter(|&i| Some(i) != safe_index)
            .collect();

        if let Some(s) = seed {
            let mut rng = StdRng::seed_from_u64(s);
            available_positions.shuffle(&mut rng);
        } else {
            let mut rng = rand::thread_rng();
            available_positions.shuffle(&mut rng);
        }

        let mut bombs = vec![false; total];
        for &pos in available_positions.iter().take(bomb_count) {
            bombs[pos] = true;
        }

        let adjacents = vec![None; total];
        Self {
            cols,
            rows,
            bomb_count,
            bombs,
            adjacents,
        }
    }

    pub fn from_squares(cols: usize, rows: usize, squares: Vec<bool>) -> Self {
        assert!(cols > 0);
        assert!(rows > 0);
        assert_eq!(squares.len(), cols * rows);

        let bomb_count = squares.iter().filter(|&&b| b).count();
        assert!(bomb_count > 0);
        assert!(bomb_count < squares.len());

        let adjacents = vec![None; squares.len()];
        Self {
            cols,
            rows,
            bomb_count,
            bombs: squares,
            adjacents,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.cols * self.rows
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn index(&self, x: usize, y: usize) -> usize {
        assert!(x < self.cols && y < self.rows);
        y * self.cols + x
    }

    #[inline]
    pub fn coordinate(&self, index: usize) -> (usize, usize) {
        assert!(index < self.len());
        (index % self.cols, index / self.cols)
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> bool {
        self.bombs[self.index(x, y)]
    }

    #[inline]
    pub fn get_by_index(&self, index: usize) -> bool {
        self.bombs[index]
    }

    pub fn get_adjacent_indices(&self, x: usize, y: usize) -> Vec<usize> {
        let mut indices = Vec::with_capacity(8);
        let x = x as isize;
        let y = y as isize;
        let cols = self.cols as isize;
        let rows = self.rows as isize;

        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if nx >= 0 && nx < cols && ny >= 0 && ny < rows {
                    indices.push((ny * cols + nx) as usize);
                }
            }
        }
        indices
    }

    pub fn get_adjacent_count(&mut self, x: usize, y: usize) -> u8 {
        assert!(!self.get(x, y), "Cannot get adjacent count from populated bomb square!");
        let idx = self.index(x, y);
        if let Some(val) = self.adjacents[idx] {
            return val;
        }

        let mut count = 0u8;
        for adj_idx in self.get_adjacent_indices(x, y) {
            if self.bombs[adj_idx] {
                count += 1;
            }
        }
        self.adjacents[idx] = Some(count);
        count
    }
}
