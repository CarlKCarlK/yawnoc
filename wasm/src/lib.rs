use js_sys::Function;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

const H: usize = 16;
const W: usize = 16;
const STASIS_RESET_GENERATIONS: u8 = 15;

const ALIVE_COLORS: [Rgb; 6] = [
    Rgb::new(0, 255, 0),
    Rgb::new(0, 255, 255),
    Rgb::new(255, 0, 255),
    Rgb::new(255, 165, 0),
    Rgb::new(255, 255, 0),
    Rgb::new(255, 255, 255),
];

#[derive(Clone, Copy)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    fn css(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
enum Pattern {
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

const PATTERNS: [Pattern; 10] = [
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
enum Status {
    Ok,
    Paused,
    Found,
    NotFound,
    SolverError,
    Cancelled,
    Searching,
    Off,
    Unknown,
}

impl Status {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Paused => "paused",
            Self::Found => "found",
            Self::NotFound => "not_found",
            Self::SolverError => "solver_error",
            Self::Cancelled => "cancelled",
            Self::Searching => "searching",
            Self::Off => "off",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum SpeedMode {
    Slow,
    Medium,
    Fast,
}

impl SpeedMode {
    const fn slower(self) -> Self {
        match self {
            Self::Slow => Self::Slow,
            Self::Medium => Self::Slow,
            Self::Fast => Self::Medium,
        }
    }

    const fn faster(self) -> Self {
        match self {
            Self::Slow => Self::Medium,
            Self::Medium => Self::Fast,
            Self::Fast => Self::Fast,
        }
    }

    const fn interval_ms(self) -> u32 {
        match self {
            Self::Slow => 500,
            Self::Medium => 160,
            Self::Fast => 50,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Slow => "slow",
            Self::Medium => "medium",
            Self::Fast => "fast",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct Board {
    cells: [[bool; W]; H],
}

impl Board {
    const fn new() -> Self {
        Self {
            cells: [[false; W]; H],
        }
    }

    fn step(&mut self) {
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

    fn count_live_neighbors(&self, row: usize, col: usize) -> u8 {
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

    fn count_live_cells(&self) -> u16 {
        self.cells.iter().flatten().filter(|&&alive| alive).count() as u16
    }

    fn set_alive(&mut self, row: usize, col: usize) {
        self.cells[row % H][col % W] = true;
    }

    fn add_pattern(&mut self, pattern: Pattern, random_seed: u32) {
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

#[derive(Serialize, Deserialize)]
struct Conway {
    board: Board,
    pattern_index: usize,
    paused: bool,
    display_power_on: bool,
    speed_mode: SpeedMode,
    color_index: usize,
    status: Status,    #[serde(default)]
    is_searching: bool,    stasis_tracker: (u8, u16),
    empty_tracker: u8,
    random_seed: u32,
}

impl Conway {
    fn new(random_seed: u32) -> Self {
        let mut board = Board::new();
        board.add_pattern(PATTERNS[1], random_seed);
        Self {
            board,
            pattern_index: 1,
            paused: false,
            display_power_on: true,
            speed_mode: SpeedMode::Medium,
            color_index: 1,
            status: Status::Ok,
            is_searching: false,
            stasis_tracker: (0, 0),
            empty_tracker: 0,
            random_seed,
        }
    }

    fn command(&mut self, key: &str) -> Status {
        self.status = match key {
            "play_pause" => {
                self.paused = !self.paused;
                if self.paused {
                    Status::Paused
                } else {
                    Status::Ok
                }
            }
            "next" => {
                if self.display_power_on && self.paused {
                    self.board.step();
                    self.evaluate_auto_reset();
                }
                Status::Ok
            }
            "prev" => {
                if self.display_power_on {
                    self.is_searching = true;
                    Status::Searching
                } else {
                    Status::Off
                }
            }
            "cancel" => {
                self.is_searching = false;
                Status::Cancelled
            }
            "search_not_found" => {
                self.is_searching = false;
                Status::NotFound
            }
            "search_solver_error" => {
                self.is_searching = false;
                Status::SolverError
            }
            "mode" => {
                self.color_index = (self.color_index + 1) % ALIVE_COLORS.len();
                Status::Ok
            }
            "speed_down" => {
                self.speed_mode = self.speed_mode.slower();
                Status::Ok
            }
            "speed_up" => {
                self.speed_mode = self.speed_mode.faster();
                Status::Ok
            }
            digit if digit.len() == 1 && digit.as_bytes()[0].is_ascii_digit() => {
                let pattern_index = (digit.as_bytes()[0] - b'0') as usize;
                self.pattern_index = pattern_index;
                self.is_searching = false;
                self.reset_board_for_pattern();
                Status::Ok
            }
            _ => Status::Unknown,
        };
        self.status
    }

    fn tick(&mut self) -> Status {
        if !self.display_power_on {
            self.status = Status::Off;
            return self.status;
        }

        if self.is_searching {
            self.status = Status::Searching;
            return self.status;
        }

        if self.paused {
            self.status = Status::Paused;
            return self.status;
        }

        self.board.step();
        self.evaluate_auto_reset();
        self.status = Status::Ok;
        self.status
    }

    fn reset_board_for_pattern(&mut self) {
        self.board = Board::new();
        let random_seed = self.next_random_seed();
        self.board
            .add_pattern(PATTERNS[self.pattern_index], random_seed);
        self.stasis_tracker = (0, 0);
        self.empty_tracker = 0;
    }

    fn evaluate_auto_reset(&mut self) {
        let live_cell_count = self.board.count_live_cells();
        let current_pattern = PATTERNS[self.pattern_index];

        if matches!(current_pattern, Pattern::Random | Pattern::Cross) {
            let (unchanged_count, last_live_count) = self.stasis_tracker;
            if live_cell_count == last_live_count {
                let new_unchanged_count = unchanged_count + 1;
                self.stasis_tracker = (new_unchanged_count, live_cell_count);
                if new_unchanged_count >= STASIS_RESET_GENERATIONS {
                    self.reset_same_pattern();
                }
            } else {
                self.stasis_tracker = (1, live_cell_count);
            }
        } else if live_cell_count == 0 {
            self.empty_tracker += 1;
            if self.empty_tracker >= STASIS_RESET_GENERATIONS {
                self.reset_same_pattern();
            }
        } else {
            self.empty_tracker = 0;
        }
    }

    fn reset_same_pattern(&mut self) {
        self.board = Board::new();
        let random_seed = self.next_random_seed();
        self.board
            .add_pattern(PATTERNS[self.pattern_index], random_seed);
        self.stasis_tracker = (0, 0);
        self.empty_tracker = 0;
    }

    fn next_random_seed(&mut self) -> u32 {
        self.random_seed = self
            .random_seed
            .wrapping_mul(1664525)
            .wrapping_add(1013904223);
        self.random_seed
    }

    fn frame(&self) -> FrameDto {
        let mut cells = Vec::with_capacity(H * W);
        if !self.display_power_on {
            cells.resize(H * W, None);
        } else {
            let alive_color = if self.is_searching {
                Rgb::new(220, 30, 30).css()
            } else {
                ALIVE_COLORS[self.color_index].css()
            };
            for row in 0..H {
                for col in 0..W {
                    cells.push(self.board.cells[row][col].then(|| alive_color.clone()));
                }
            }
        }

        FrameDto {
            width: W,
            height: H,
            cells,
            status: self.status.as_str(),
            live_cells: self.board.count_live_cells(),
            tick_interval_ms: self.speed_mode.interval_ms(),
            speed: self.speed_mode.as_str(),
        }
    }
}

fn build_life_clauses(target: &Board) -> Vec<Vec<i32>> {
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
                            if (bits >> j) & 1 == 1 { -var } else { var }
                        })
                        .collect();
                    clauses.push(clause);
                }
            }
        }
    }
    clauses
}

fn add_atmost_k(clauses: &mut Vec<Vec<i32>>, k: usize) {
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

enum SolveOutcome {
    Sat(Board),
    NotFound,
    Error,
}

enum SearchOutcome {
    Found(Board),
    NotFound,
    Error,
}

fn solve_clauses(clauses: Vec<Vec<i32>>) -> SolveOutcome {
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

fn sat_predecessor_with_progress(target: &Board, on_progress: impl Fn(&Board)) -> SearchOutcome {
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

#[derive(Serialize)]
struct FrameDto {
    width: usize,
    height: usize,
    cells: Vec<Option<String>>,
    status: &'static str,
    live_cells: u16,
    tick_interval_ms: u32,
    speed: &'static str,
}

#[wasm_bindgen]
pub struct WasmApp {
    conway: Conway,
}

#[wasm_bindgen]
impl WasmApp {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32) -> Self {
        Self {
            conway: Conway::new(seed),
        }
    }

    pub fn frame_json(&self) -> String {
        serde_json::to_string(&self.conway.frame()).expect("failed to serialize frame")
    }

    pub fn tick_json(&mut self) -> String {
        self.conway.tick();
        serde_json::to_string(&self.conway.frame()).expect("failed to serialize frame")
    }

    pub fn press_key_json(&mut self, key: &str) -> String {
        self.conway.command(key);
        serde_json::to_string(&self.conway.frame()).expect("failed to serialize frame")
    }

    pub fn toggle_cell_json(&mut self, row: usize, col: usize) -> String {
        if row < H && col < W {
            self.conway.board.cells[row][col] ^= true;
        }
        serde_json::to_string(&self.conway.frame()).expect("failed to serialize frame")
    }

    pub fn state_json(&self) -> String {
        serde_json::to_string(&self.conway).expect("failed to serialize app state")
    }

    pub fn replace_state_json(&mut self, json: &str) -> bool {
        match serde_json::from_str::<Conway>(json) {
            Ok(state) => {
                self.conway = state;
                true
            }
            Err(_) => false,
        }
    }

    pub fn show_progress_json(&mut self, board_json: &str) {
        if let Ok(board) = serde_json::from_str::<Board>(board_json) {
            self.conway.board = board;
            // keep is_searching = true so the tick loop renders in red
        }
    }

    pub fn board_json(&self) -> String {
        serde_json::to_string(&self.conway.board).expect("failed to serialize board")
    }

    pub fn apply_predecessor_json(&mut self, board_json: &str) -> String {
        if let Ok(board) = serde_json::from_str::<Board>(board_json) {
            self.conway.board = board;
            self.conway.is_searching = false;
            self.conway.stasis_tracker = (0, 0);
            self.conway.empty_tracker = 0;
            self.conway.status = Status::Found;
        }
        serde_json::to_string(&self.conway.frame()).expect("failed to serialize frame")
    }

    pub fn search_not_found_json(&mut self) -> String {
        self.conway.is_searching = false;
        self.conway.status = Status::NotFound;
        serde_json::to_string(&self.conway.frame()).expect("failed to serialize frame")
    }

    pub fn search_solver_error_json(&mut self) -> String {
        self.conway.is_searching = false;
        self.conway.status = Status::SolverError;
        serde_json::to_string(&self.conway.frame()).expect("failed to serialize frame")
    }
}

/// Runs the SAT predecessor search, calling `on_progress` with each
/// intermediate improvement (as board JSON) before returning the final result.
#[wasm_bindgen]
pub fn find_predecessor_json_with_progress(
    board_json: &str,
    on_progress: &Function,
) -> String {
    #[derive(Serialize)]
    struct SearchResponse {
        kind: &'static str,
        result: Option<String>,
    }

    let outcome = match serde_json::from_str::<Board>(board_json) {
        Ok(board) => sat_predecessor_with_progress(&board, |b| {
            if let Ok(json) = serde_json::to_string(b) {
                let _ = on_progress.call1(&JsValue::NULL, &JsValue::from_str(&json));
            }
        }),
        Err(_) => SearchOutcome::Error,
    };

    let response = match outcome {
        SearchOutcome::Found(pred) => SearchResponse {
            kind: "found",
            result: serde_json::to_string(&pred).ok(),
        },
        SearchOutcome::NotFound => SearchResponse {
            kind: "not_found",
            result: None,
        },
        SearchOutcome::Error => SearchResponse {
            kind: "solver_error",
            result: None,
        },
    };

    serde_json::to_string(&response).expect("failed to serialize search response")
}
