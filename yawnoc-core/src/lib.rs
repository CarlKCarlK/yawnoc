use serde::{Deserialize, Serialize};

pub const H: usize = 16;
pub const W: usize = 16;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Pattern {
    Glider,
    Random,
    Blinker,
    Toad,
    Beacon,
    Lwss,
    Block,
    Pentadecathlon,
    Cross,
    Custom9,
}

pub const PATTERNS: [Pattern; 10] = [
    Pattern::Glider,
    Pattern::Random,
    Pattern::Blinker,
    Pattern::Toad,
    Pattern::Beacon,
    Pattern::Lwss,
    Pattern::Block,
    Pattern::Pentadecathlon,
    Pattern::Cross,
    Pattern::Custom9,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Board {
    pub cells: [[bool; W]; H],
}

impl Board {
    pub const fn new() -> Self {
        Self {
            cells: [[false; W]; H],
        }
    }

    pub fn step(&mut self) {
        let mut next = [[false; W]; H];
        for row in 0..H {
            for col in 0..W {
                let live_neighbors = self.count_live_neighbors(row, col);
                let alive = self.cells[row][col];
                next[row][col] =
                    matches!((alive, live_neighbors), (true, 2) | (true, 3) | (false, 3));
            }
        }
        self.cells = next;
    }

    pub fn count_live_neighbors(&self, row: usize, col: usize) -> u8 {
        let mut count = 0;
        for row_offset in [-1, 0, 1] {
            for col_offset in [-1, 0, 1] {
                if row_offset == 0 && col_offset == 0 {
                    continue;
                }
                let neighbor_row = ((row as isize + row_offset).rem_euclid(H as isize)) as usize;
                let neighbor_col = ((col as isize + col_offset).rem_euclid(W as isize)) as usize;
                if self.cells[neighbor_row][neighbor_col] {
                    count += 1;
                }
            }
        }
        count
    }

    pub fn count_live_cells(&self) -> u16 {
        self.cells.iter().flatten().filter(|&&alive| alive).count() as u16
    }

    pub fn set_alive(&mut self, row: usize, col: usize) {
        self.cells[row % H][col % W] = true;
    }

    pub fn add_pattern(&mut self, pattern: Pattern, random_seed: u32) {
        match pattern {
            Pattern::Glider => self.add_glider(4, 2),
            Pattern::Random => self.add_random(random_seed),
            Pattern::Blinker => self.add_blinker(5, 4),
            Pattern::Toad => self.add_toad(5, 4),
            Pattern::Beacon => self.add_beacon(4, 4),
            Pattern::Lwss => self.add_lwss(5, 6),
            Pattern::Block => self.add_block(5, 4),
            Pattern::Pentadecathlon => self.add_pentadecathlon(),
            Pattern::Cross => self.add_cross(7, 7),
            Pattern::Custom9 => self.add_custom9(),
        }
    }

    fn add_glider(&mut self, row: usize, col: usize) {
        self.set_alive(row, col + 1);
        self.set_alive(row + 1, col + 2);
        self.set_alive(row + 2, col);
        self.set_alive(row + 2, col + 1);
        self.set_alive(row + 2, col + 2);
    }

    fn add_blinker(&mut self, row: usize, col: usize) {
        self.set_alive(row, col);
        self.set_alive(row, col + 1);
        self.set_alive(row, col + 2);
    }

    fn add_toad(&mut self, row: usize, col: usize) {
        self.set_alive(row, col + 1);
        self.set_alive(row, col + 2);
        self.set_alive(row, col + 3);
        self.set_alive(row + 1, col);
        self.set_alive(row + 1, col + 1);
        self.set_alive(row + 1, col + 2);
    }

    fn add_beacon(&mut self, row: usize, col: usize) {
        self.set_alive(row, col);
        self.set_alive(row, col + 1);
        self.set_alive(row + 1, col);
        self.set_alive(row + 1, col + 1);
        self.set_alive(row + 2, col + 2);
        self.set_alive(row + 2, col + 3);
        self.set_alive(row + 3, col + 2);
        self.set_alive(row + 3, col + 3);
    }

    fn add_lwss(&mut self, row: usize, col: usize) {
        self.set_alive(row, col + 1);
        self.set_alive(row + 1, col);
        self.set_alive(row + 2, col);
        self.set_alive(row + 2, col + 1);
        self.set_alive(row + 2, col + 2);
        self.set_alive(row + 2, col + 3);
        self.set_alive(row + 1, col + 3);
    }

    fn add_block(&mut self, row: usize, col: usize) {
        self.set_alive(row, col);
        self.set_alive(row, col + 1);
        self.set_alive(row + 1, col);
        self.set_alive(row + 1, col + 1);
    }

    fn add_cross(&mut self, row: usize, col: usize) {
        for c in 0..W {
            self.set_alive(row, c);
        }
        for r in 0..H {
            self.set_alive(r, col);
        }
    }

    fn add_random(&mut self, mut random_seed: u32) {
        for row in 0..H {
            for col in 0..W {
                random_seed = random_seed.wrapping_mul(1664525).wrapping_add(1013904223);
                self.cells[row][col] = (random_seed & 0x100) != 0;
            }
        }
    }

    fn add_pentadecathlon(&mut self) {
        self.draw_ascii(&[
            "................",
            "................",
            "................",
            "......###.......",
            ".....#...#......",
            "................",
            "....#.....#.....",
            "....#.....#.....",
            "................",
            ".....#...#......",
            "......###.......",
            "................",
            "................",
            "................",
            "................",
            "................",
        ]);
    }

    fn add_custom9(&mut self) {
        self.draw_ascii(&[
            "................",
            "...##.....##....",
            "....##...##.....",
            ".#..#.#.#.#..#..",
            ".###.##.##.###..",
            "..#.#.#.#.#.#...",
            "...###...###....",
            "................",
            "...###...###....",
            "..#.#.#.#.#.#...",
            ".###.##.##.###..",
            ".#..#.#.#.#..#..",
            "....##...##.....",
            "...##.....##....",
            "................",
            "................",
        ]);
    }

    fn draw_ascii(&mut self, rows: &[&str]) {
        for (row, text) in rows.iter().enumerate() {
            for (col, ch) in text.chars().enumerate() {
                if ch == '#' {
                    self.set_alive(row, col);
                }
            }
        }
    }
}

pub fn build_life_clauses(target: &Board) -> Vec<Vec<i32>> {
    let mut clauses = Vec::new();
    for row in 0..H {
        for col in 0..W {
            let target_alive = target.cells[row][col];
            let nb: [(usize, usize); 9] = [
                ((row + H - 1) % H, (col + W - 1) % W),
                ((row + H - 1) % H, col),
                ((row + H - 1) % H, (col + 1) % W),
                (row, (col + W - 1) % W),
                (row, col),
                (row, (col + 1) % W),
                ((row + 1) % H, (col + W - 1) % W),
                ((row + 1) % H, col),
                ((row + 1) % H, (col + 1) % W),
            ];
            for bits in 0u16..512 {
                let center = (bits >> 4) & 1;
                let neighbor_sum: u16 =
                    (0u16..9).filter(|&j| j != 4).map(|j| (bits >> j) & 1).sum();
                let produces_alive = (center == 1 && (neighbor_sum == 2 || neighbor_sum == 3))
                    || (center == 0 && neighbor_sum == 3);
                if produces_alive != target_alive {
                    let clause: Vec<i32> = nb
                        .iter()
                        .enumerate()
                        .map(|(j, &(nr, nc))| {
                            let var = (nr * W + nc + 1) as i32;
                            if (bits >> j) & 1 == 1 {
                                -var
                            } else {
                                var
                            }
                        })
                        .collect();
                    clauses.push(clause);
                }
            }
        }
    }
    clauses
}

pub fn add_atmost_k(clauses: &mut Vec<Vec<i32>>, k: usize) {
    let n = H * W;
    if k == 0 {
        for i in 1..=(n as i32) {
            clauses.push(vec![-i]);
        }
        return;
    }
    if k >= n {
        return;
    }
    let r = |i: usize, j: usize| -> i32 { (n + (i - 1) * k + j) as i32 };
    clauses.push(vec![-1, r(1, 1)]);
    for i in 2..n {
        clauses.push(vec![-(i as i32), r(i, 1)]);
        clauses.push(vec![-r(i - 1, 1), r(i, 1)]);
        for j in 2..=k {
            clauses.push(vec![-(i as i32), -r(i - 1, j - 1), r(i, j)]);
            clauses.push(vec![-r(i - 1, j), r(i, j)]);
        }
        clauses.push(vec![-(i as i32), -r(i - 1, k)]);
    }
    clauses.push(vec![-(n as i32), -r(n - 1, k)]);
}

pub enum SolveOutcome {
    Sat(Board),
    NotFound,
    Error,
}

pub enum SearchOutcome {
    Found(Board),
    NotFound,
    Error,
}

pub fn solve_clauses(clauses: Vec<Vec<i32>>) -> SolveOutcome {
    let old_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(|| splr::Certificate::try_from(clauses));
    std::panic::set_hook(old_hook);
    match result {
        Ok(Ok(splr::Certificate::SAT(ans))) => {
            let mut board = Board::new();
            for &lit in &ans {
                if lit > 0 {
                    let idx = (lit - 1) as usize;
                    if idx < H * W {
                        board.cells[idx / W][idx % W] = true;
                    }
                }
            }
            SolveOutcome::Sat(board)
        }
        Ok(Ok(_)) => SolveOutcome::NotFound,
        Ok(Err(_)) | Err(_) => SolveOutcome::Error,
    }
}

pub fn sat_predecessor_with_progress(
    target: &Board,
    on_progress: impl Fn(&Board),
) -> SearchOutcome {
    let base = build_life_clauses(target);

    // Probe increasingly looser density caps starting at N, then fall back to
    // one unconstrained solve if none of those caps admits a predecessor.
    let n = target.count_live_cells() as usize;
    let mut cap = n;
    let mut best = None;
    while cap < H * W {
        let mut constrained = base.clone();
        add_atmost_k(&mut constrained, cap);
        match solve_clauses(constrained) {
            SolveOutcome::Sat(board) => {
                best = Some(board);
                break;
            }
            SolveOutcome::NotFound => {}
            SolveOutcome::Error => break,
        }

        if cap == 0 {
            cap = 1;
        } else {
            cap = cap.saturating_mul(2);
        }
    }

    let mut best = match best {
        Some(board) => board,
        None => match solve_clauses(base.clone()) {
            SolveOutcome::Sat(board) => board,
            SolveOutcome::NotFound => return SearchOutcome::NotFound,
            SolveOutcome::Error => return SearchOutcome::Error,
        },
    };
    on_progress(&best);
    let mut hi = best.count_live_cells() as usize;
    let mut lo = 0usize;

    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let mut clauses = base.clone();
        add_atmost_k(&mut clauses, mid);
        match solve_clauses(clauses) {
            SolveOutcome::Sat(board) => {
                let count = board.count_live_cells() as usize;
                if count < hi {
                    hi = count;
                    best = board;
                    on_progress(&best);
                } else {
                    hi = mid;
                }
            }
            SolveOutcome::NotFound => {
                lo = mid + 1;
            }
            SolveOutcome::Error => return SearchOutcome::Error,
        }
    }

    SearchOutcome::Found(best)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board_from_live_cells(cells: &[(usize, usize)]) -> Board {
        let mut board = Board::new();
        for &(row, col) in cells {
            board.set_alive(row, col);
        }
        board
    }

    #[test]
    fn blinker_steps_to_vertical_phase() {
        let mut board = Board::new();
        board.add_pattern(Pattern::Blinker, 0);

        board.step();

        let expected = board_from_live_cells(&[(4, 5), (5, 5), (6, 5)]);
        assert_eq!(board, expected);
    }

    #[test]
    fn edge_neighbors_wrap_around() {
        let board = board_from_live_cells(&[(15, 15), (15, 0), (0, 15)]);

        assert_eq!(board.count_live_neighbors(0, 0), 3);
    }

    #[test]
    fn random_pattern_is_seed_deterministic() {
        let mut first = Board::new();
        let mut second = Board::new();

        first.add_pattern(Pattern::Random, 0x9e37_79b9);
        second.add_pattern(Pattern::Random, 0x9e37_79b9);

        assert_eq!(first, second);
    }

    #[test]
    fn at_most_zero_forbids_all_board_vars() {
        let mut clauses = Vec::new();

        add_atmost_k(&mut clauses, 0);

        assert_eq!(clauses.len(), H * W);
        assert_eq!(clauses[0], vec![-1]);
        assert_eq!(clauses[H * W - 1], vec![-((H * W) as i32)]);
    }
}
