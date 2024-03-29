// Filepaths
pub const NPC_DIRECTORY: &str = "assets/specs/npcs/";
pub const MAP_DIRECTORY: &str = "assets/specs/maps/";

// Game Parameters
pub const DEFAULT_BODY_SIZE: u8 = 40;
pub const DEFAULT_HEAD_SIZE: u8 = 10;
pub const DEFAULT_ARM_SIZE: u8 = 15;
pub const DEFAULT_LEG_SIZE: u8 = 20;
pub const DEFAULT_HAND_SIZE: u8 = 5;
pub const DEFAULT_FOOT_SIZE: u8 = 5;
pub const MOVEMENT_TICKS: u8 = 10;
pub const AI_SPEAK_TICKS: u8 = 10;

// UI
pub const DEFAULT_FONT_SIZE: f32 = 24.;
pub const MENU_TO_SCREEN_RATIO: f32 = 0.8;
pub const MENU_INDENTATION: &str = "    ";
pub const LOG_WINDOW_SIZE: (f32, f32) = (540., 200.);
pub const LOG_TEXT_SIZE: f32 = 14.;
pub const KEY_HOLD_DELAY_IN_MILLIS: u64 = 400;
pub const TOAST_MESSAGE_TIME_IN_SECONDS: f32 = 3.;

// Map
pub const DEFAULT_MAP_WIDTH_IN_TILES: usize = 50;
pub const DEFAULT_MAP_HEIGHT_IN_TILES: usize = 50;
pub const TILE_WIDTH: usize = 32;
pub const TILE_HEIGHT: usize = 32;
pub const ASCII_TILE_FONT_SIZE: f32 = 40.;
pub const CAMERA_ZOOM_LOG_BASE: f32 = 2.0;
pub const CAMERA_ZOOM_SPEED: f32 = 0.2;
pub const CAMERA_MOVE_SPEED: f32 = 10.;

// Colors
pub const BLUEPRINT_BLUE: (f32, f32, f32) = (0.25, 0.25, 0.75);

// Ascii
pub const MIDDLE_DOT: char = '\u{00B7}';
pub const EM_DASH: char = '\u{2014}';

// Greek Letters
pub const ALPHA_LOWER: char = 'α';
pub const ALPHA_UPPER: char = 'Α';
pub const BETA_LOWER: char = 'β';
pub const BETA_UPPER: char = 'Β';
pub const GAMMA_LOWER: char = 'γ';
pub const GAMMA_UPPER: char = 'Γ';
pub const DELTA_LOWER: char = 'δ';
pub const DELTA_UPPER: char = 'Δ';
pub const EPSILON_LOWER: char = 'ε';
pub const EPSILON_UPPER: char = 'Ε';
pub const ZETA_LOWER: char = 'ζ';
pub const ZETA_UPPER: char = 'Ζ';
pub const ETA_LOWER: char = 'η';
pub const ETA_UPPER: char = 'Η';
pub const THETA_LOWER: char = 'θ';
pub const THETA_UPPER: char = 'Θ';
pub const IOTA_LOWER: char = 'ι';
pub const IOTA_UPPER: char = 'Ι';
pub const KAPPA_LOWER: char = 'κ';
pub const KAPPA_UPPER: char = 'Κ';
pub const LAMBDA_LOWER: char = 'λ';
pub const LAMBDA_UPPER: char = 'Λ';
pub const MU_LOWER: char = 'μ';
pub const MU_UPPER: char = 'Μ';
pub const NU_LOWER: char = 'ν';
pub const NU_UPPER: char = 'Ν';
pub const XI_LOWER: char = 'ξ';
pub const XI_UPPER: char = 'Ξ';
pub const OMICRON_LOWER: char = 'ο';
pub const OMICRON_UPPER: char = 'Ο';
pub const PI_LOWER: char = 'π';
pub const PI_UPPER: char = 'Π';
pub const RHO_LOWER: char = 'ρ';
pub const RHO_UPPER: char = 'Ρ';
pub const SIGMA_LOWER: char = 'σ';
pub const SIGMA_UPPER: char = 'Σ';
pub const TAU_LOWER: char = 'τ';
pub const TAU_UPPER: char = 'Τ';
pub const UPSILON_LOWER: char = 'υ';
pub const UPSILON_UPPER: char = 'Υ';
pub const PHI_LOWER: char = 'φ';
pub const PHI_UPPER: char = 'Φ';
pub const CHI_LOWER: char = 'χ';
pub const CHI_UPPER: char = 'Χ';
pub const PSI_LOWER: char = 'ψ';
pub const PSI_UPPER: char = 'Ψ';
pub const OMEGA_LOWER: char = 'ω';
pub const OMEGA_UPPER: char = 'Ω';

pub const GREEK_ALPHABET: &'static [char] = &[
    ALPHA_LOWER,
    BETA_LOWER,
    GAMMA_LOWER,
    DELTA_LOWER,
    EPSILON_LOWER,
    ZETA_LOWER,
    ETA_LOWER,
    THETA_LOWER,
    IOTA_LOWER,
    KAPPA_LOWER,
    LAMBDA_LOWER,
    MU_LOWER,
    NU_LOWER,
    XI_LOWER,
    OMICRON_LOWER,
    PI_LOWER,
    RHO_LOWER,
    SIGMA_LOWER,
    TAU_LOWER,
    UPSILON_LOWER,
    PHI_LOWER,
    CHI_LOWER,
    PSI_LOWER,
    OMEGA_LOWER,
    ALPHA_UPPER,
    BETA_UPPER,
    GAMMA_UPPER,
    DELTA_UPPER,
    EPSILON_UPPER,
    ZETA_UPPER,
    ETA_UPPER,
    THETA_UPPER,
    IOTA_UPPER,
    KAPPA_UPPER,
    LAMBDA_UPPER,
    MU_UPPER,
    NU_UPPER,
    XI_UPPER,
    OMICRON_UPPER,
    PI_UPPER,
    RHO_UPPER,
    SIGMA_UPPER,
    TAU_UPPER,
    UPSILON_UPPER,
    PHI_UPPER,
    CHI_UPPER,
    PSI_UPPER,
    OMEGA_UPPER,
];
