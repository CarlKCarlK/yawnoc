use serde::Serialize;
use std::sync::mpsc::{self, Receiver};
use std::sync::Mutex;
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

#[derive(Clone, Copy, Debug)]
enum Status {
    Ok,
    Paused,
    Searching,
    Found,
    NotFound,
    SolverError,
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
            Self::SolverError => "solver_error",
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

enum SearchMessage {
    Progress(Board),
    Done(SearchOutcome),
}

struct Conway {
    board: Board,
    pattern_index: usize,
    search_rx: Option<Receiver<SearchMessage>>,
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
            search_rx: None,
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
        if self.search_rx.is_some() {
            self.search_rx = None;
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
                    self.start_search();
                    Status::Searching
                } else {
                    Status::Off
                }
            }
            "cancel" => {
                self.search_rx = None;
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

        if self.search_rx.is_some() {
            self.poll_search();
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
        self.search_rx = None;
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
            let alive_color = if matches!(self.status, Status::Searching) {
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

    fn start_search(&mut self) {
        let target = self.board;
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let outcome = sat_predecessor_with_progress(&target, |board| {
                let _ = tx.send(SearchMessage::Progress(*board));
            });
            let _ = tx.send(SearchMessage::Done(outcome));
        });
        self.search_rx = Some(rx);
    }

    fn poll_search(&mut self) {
        while let Some(rx) = self.search_rx.as_ref() {
            match rx.try_recv() {
                Ok(SearchMessage::Progress(board)) => {
                    self.board = board;
                    self.status = Status::Searching;
                }
                Ok(SearchMessage::Done(SearchOutcome::Found(board))) => {
                    self.board = board;
                    self.search_rx = None;
                    self.stasis_tracker = (0, 0);
                    self.empty_tracker = 0;
                    self.status = Status::Found;
                    return;
                }
                Ok(SearchMessage::Done(SearchOutcome::NotFound)) => {
                    self.search_rx = None;
                    self.status = Status::NotFound;
                    return;
                }
                Ok(SearchMessage::Done(SearchOutcome::Error)) => {
                    self.search_rx = None;
                    self.status = Status::SolverError;
                    return;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    self.status = Status::Searching;
                    return;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.search_rx = None;
                    self.status = Status::SolverError;
                    return;
                }
            }
        }
    }
}

#[tauri::command]
fn toggle_cell(row: usize, col: usize, state: tauri::State<'_, AppState>) -> FrameDto {
    let mut conway = state.conway.lock().expect("conway mutex poisoned");
    if row < H && col < W {
        conway.board.cells[row][col] ^= true;
    }
    conway.frame()
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
        .invoke_handler(tauri::generate_handler![
            frame,
            tick,
            press_key,
            toggle_cell
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
