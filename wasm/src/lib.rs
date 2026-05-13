use js_sys::Function;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use yawnoc_core::{sat_predecessor_with_progress, Board, Pattern, SearchOutcome, H, PATTERNS, W};

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

#[derive(Serialize, Deserialize)]
struct Conway {
    board: Board,
    pattern_index: usize,
    paused: bool,
    display_power_on: bool,
    speed_mode: SpeedMode,
    color_index: usize,
    status: Status,
    #[serde(default)]
    is_searching: bool,
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
pub fn find_predecessor_json_with_progress(board_json: &str, on_progress: &Function) -> String {
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
