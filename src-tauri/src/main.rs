use serde::Serialize;
use std::sync::Mutex;

const H: usize = 16;
const W: usize = 16;
const SEARCH_ITERATIONS_PER_STEP: u32 = 256;
const MAX_SEARCH_ITERATIONS: u32 = 500_000;
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

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
enum Status {
    Ok,
    Paused,
    Searching,
    Found,
    NotFound,
    Cancelled,
    Off,
    Unknown,
}

impl Status {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Paused => "paused",
            Self::Searching => "searching",
            Self::Found => "found",
            Self::NotFound => "not_found",
            Self::Cancelled => "cancelled",
            Self::Off => "off",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy, PartialEq, Eq)]
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

    fn evolves_to(&self, target: &Self) -> bool {
        let mut next = *self;
        next.step();
        next == *target
    }

    fn predecessor_search_mask(&self, radius: usize) -> [[bool; W]; H] {
        let mut mask = [[false; W]; H];
        let radius = radius as isize;
        for row in 0..H {
            for col in 0..W {
                if self.cells[row][col] {
                    for row_delta in -radius..=radius {
                        for col_delta in -radius..=radius {
                            let mask_row =
                                ((row as isize + row_delta).rem_euclid(H as isize)) as usize;
                            let mask_col =
                                ((col as isize + col_delta).rem_euclid(W as isize)) as usize;
                            mask[mask_row][mask_col] = true;
                        }
                    }
                }
            }
        }
        mask
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

struct Conway {
    board: Board,
    pattern_index: usize,
    search: Option<PredecessorSearch>,
    paused: bool,
    display_power_on: bool,
    speed_mode: SpeedMode,
    color_index: usize,
    status: Status,
    stasis_tracker: (u8, u16),
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
            search: None,
            paused: false,
            display_power_on: true,
            speed_mode: SpeedMode::Medium,
            color_index: 1,
            status: Status::Ok,
            stasis_tracker: (0, 0),
            empty_tracker: 0,
            random_seed,
        }
    }

    fn command(&mut self, key: &str) -> Status {
        if self.search.is_some() {
            self.search = None;
            if matches!(key, "prev" | "cancel") {
                self.status = Status::Cancelled;
                return self.status;
            }
        }

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
                    self.search = Some(PredecessorSearch::new(self.board));
                    Status::Searching
                } else {
                    Status::Off
                }
            }
            "cancel" => {
                self.search = None;
                Status::Cancelled
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

        if let Some(search) = &mut self.search {
            self.status = match search.advance(SEARCH_ITERATIONS_PER_STEP) {
                SearchStep::Progress => Status::Searching,
                SearchStep::Found(predecessor) => {
                    self.board = predecessor;
                    self.search = None;
                    self.stasis_tracker = (0, 0);
                    self.empty_tracker = 0;
                    Status::Found
                }
                SearchStep::NotFound => {
                    self.search = None;
                    Status::NotFound
                }
            };
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
        self.search = None;
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
        } else if let Some(search) = &self.search {
            let (candidate, assigned, target) = search.progress();
            for row in 0..H {
                for col in 0..W {
                    let color = if assigned[row][col] {
                        if candidate.cells[row][col] {
                            Some(Rgb::new(255, 0, 0).css())
                        } else {
                            Some(Rgb::new(0, 0, 12).css())
                        }
                    } else if target.cells[row][col] {
                        Some(Rgb::new(0, 10, 0).css())
                    } else {
                        None
                    };
                    cells.push(color);
                }
            }
        } else {
            let alive_color = ALIVE_COLORS[self.color_index].css();
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

enum SearchStep {
    Progress,
    Found(Board),
    NotFound,
}

struct PredecessorSearch {
    target: Board,
    candidate: Board,
    search_mask: [[bool; W]; H],
    choices: [[u8; W]; H],
    assigned: [[bool; W]; H],
    depth: usize,
    active_count: usize,
    iteration: u32,
}

impl PredecessorSearch {
    fn new(target: Board) -> Self {
        let search_mask = target.predecessor_search_mask(1);
        let mut assigned = [[true; W]; H];
        let active_count = count_search_cells(&search_mask);
        for row in 0..H {
            for col in 0..W {
                assigned[row][col] = !search_mask[row][col];
            }
        }

        Self {
            target,
            candidate: Board::new(),
            search_mask,
            choices: [[0; W]; H],
            assigned,
            depth: 0,
            active_count,
            iteration: 0,
        }
    }

    fn progress(&self) -> (Board, [[bool; W]; H], Board) {
        (self.candidate, self.assigned, self.target)
    }

    fn advance(&mut self, budget: u32) -> SearchStep {
        for _ in 0..budget {
            if let Some(step) = self.advance_once() {
                return step;
            }
        }
        SearchStep::Progress
    }

    fn advance_once(&mut self) -> Option<SearchStep> {
        if self.depth == self.active_count {
            if self.candidate.evolves_to(&self.target) {
                return Some(SearchStep::Found(self.candidate));
            }
            if self.depth == 0 {
                return Some(SearchStep::NotFound);
            }
            self.depth -= 1;
            if let Some((row, col)) = search_cell_at(&self.search_mask, self.depth) {
                self.assigned[row][col] = false;
            }
            return None;
        }

        let Some((row, col)) = search_cell_at(&self.search_mask, self.depth) else {
            return Some(SearchStep::NotFound);
        };

        let try_value = match self.choices[row][col] {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        };

        if let Some(value) = try_value {
            self.choices[row][col] += 1;
            self.candidate.cells[row][col] = value;
            self.assigned[row][col] = true;

            if check_search_constraints(&self.candidate, &self.assigned, &self.target, row, col) {
                self.depth += 1;
            } else {
                self.assigned[row][col] = false;
            }
        } else {
            self.choices[row][col] = 0;
            self.assigned[row][col] = false;
            if self.depth == 0 {
                return Some(SearchStep::NotFound);
            }
            self.depth -= 1;
            if let Some((prev_row, prev_col)) = search_cell_at(&self.search_mask, self.depth) {
                self.assigned[prev_row][prev_col] = false;
            }
        }

        self.iteration += 1;
        if self.iteration >= MAX_SEARCH_ITERATIONS {
            return Some(SearchStep::NotFound);
        }

        None
    }
}

fn check_search_constraints(
    candidate: &Board,
    assigned: &[[bool; W]; H],
    target: &Board,
    changed_row: usize,
    changed_col: usize,
) -> bool {
    for row_delta in [-1, 0, 1] {
        for col_delta in [-1, 0, 1] {
            let row = ((changed_row as isize + row_delta).rem_euclid(H as isize)) as usize;
            let col = ((changed_col as isize + col_delta).rem_euclid(W as isize)) as usize;

            let mut complete = true;
            'neighborhood: for neighbor_row_offset in [-1, 0, 1] {
                for neighbor_col_offset in [-1, 0, 1] {
                    let neighbor_row =
                        ((row as isize + neighbor_row_offset).rem_euclid(H as isize)) as usize;
                    let neighbor_col =
                        ((col as isize + neighbor_col_offset).rem_euclid(W as isize)) as usize;
                    if !assigned[neighbor_row][neighbor_col] {
                        complete = false;
                        break 'neighborhood;
                    }
                }
            }

            if complete {
                let live_neighbors = candidate.count_live_neighbors(row, col);
                let alive = candidate.cells[row][col];
                let next_alive =
                    matches!((alive, live_neighbors), (true, 2) | (true, 3) | (false, 3));
                if next_alive != target.cells[row][col] {
                    return false;
                }
            }
        }
    }
    true
}

fn count_search_cells(search_mask: &[[bool; W]; H]) -> usize {
    search_mask.iter().flatten().filter(|&&cell| cell).count()
}

fn search_cell_at(search_mask: &[[bool; W]; H], target_index: usize) -> Option<(usize, usize)> {
    let mut index = 0;
    for (row, line) in search_mask.iter().enumerate() {
        for (col, &is_search_cell) in line.iter().enumerate() {
            if is_search_cell {
                if index == target_index {
                    return Some((row, col));
                }
                index += 1;
            }
        }
    }
    None
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

struct AppState {
    conway: Mutex<Conway>,
}

#[tauri::command]
fn frame(state: tauri::State<'_, AppState>) -> FrameDto {
    state.conway.lock().expect("conway mutex poisoned").frame()
}

#[tauri::command]
fn tick(state: tauri::State<'_, AppState>) -> FrameDto {
    let mut conway = state.conway.lock().expect("conway mutex poisoned");
    conway.tick();
    conway.frame()
}

#[tauri::command]
fn press_key(key: String, state: tauri::State<'_, AppState>) -> FrameDto {
    let mut conway = state.conway.lock().expect("conway mutex poisoned");
    conway.command(&key);
    conway.frame()
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            conway: Mutex::new(Conway::new(0x9e37_79b9)),
        })
        .invoke_handler(tauri::generate_handler![frame, tick, press_key])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
