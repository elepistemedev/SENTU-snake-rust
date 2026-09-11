//! Theme configuration and palette definitions for Snake AI
//!
//! Provides customizable visual themes for the snake (head & body) and the food/apple,
//! with safe JSON persistence to disk (`theme_config.json`).

use macroquad::prelude::Color;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const THEME_CONFIG_FILE: &str = "theme_config.json";

/// Available visual themes for the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GameTheme {
    /// Classic retro style: bright green head, darker green body, white food.
    #[default]
    Retro,
    /// Cyber/Synthwave neon arcade style: electric cyan head, cobalt blue body, bright neon pink food.
    Arcade,
    /// Soft, harmonious modern palette: mint emerald head, sage teal body, warm coral apple.
    Pleasant,
    /// Natural illustrated style: meadow green with dark outline, vivid red apple.
    Meadow,
}

/// Resolved color palette for rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeColors {
    pub head: Color,
    pub body: Color,
    pub food: Color,
    pub outline: Option<Color>,
}

impl GameTheme {
    /// Human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            GameTheme::Retro => "Retro",
            GameTheme::Arcade => "Arcade",
            GameTheme::Pleasant => "Agradable",
            GameTheme::Meadow => "Pradera",
        }
    }

    /// Short description of the theme.
    pub fn description(&self) -> &'static str {
        match self {
            GameTheme::Retro => "Verde clásico + Manzana blanca",
            GameTheme::Arcade => "Cian neón + Manzana rosa brillante",
            GameTheme::Pleasant => "Menta suave + Manzana coral cálido",
            GameTheme::Meadow => "Verde natural con contorno + Manzana roja",
        }
    }

    /// Returns the active colors for this theme.
    pub fn colors(&self) -> ThemeColors {
        match self {
            GameTheme::Retro => ThemeColors {
                head: Color::new(0.3, 0.9, 0.3, 1.0),
                body: Color::new(0.2, 0.7, 0.2, 1.0),
                food: Color::new(1.0, 1.0, 1.0, 1.0),
                outline: None,
            },
            GameTheme::Arcade => ThemeColors {
                head: Color::new(0.0, 0.95, 1.0, 1.0),
                body: Color::new(0.1, 0.5, 0.9, 1.0),
                food: Color::new(1.0, 0.1, 0.6, 1.0),
                outline: None,
            },
            GameTheme::Pleasant => ThemeColors {
                head: Color::new(0.28, 0.82, 0.65, 1.0),
                body: Color::new(0.20, 0.65, 0.55, 1.0),
                food: Color::new(0.98, 0.42, 0.42, 1.0),
                outline: None,
            },
            GameTheme::Meadow => ThemeColors {
                head: Color::new(0.32, 0.65, 0.40, 1.0),
                body: Color::new(0.35, 0.68, 0.42, 1.0),
                food: Color::new(0.88, 0.22, 0.28, 1.0),
                outline: Some(Color::new(0.18, 0.41, 0.24, 1.0)),
            },
        }
    }

    /// All themes in order.
    pub fn all() -> [GameTheme; 4] {
        [GameTheme::Retro, GameTheme::Arcade, GameTheme::Pleasant, GameTheme::Meadow]
    }
}

/// Persisted configuration format.
#[derive(Serialize, Deserialize)]
struct StoredTheme {
    theme: GameTheme,
}

/// Load the persisted theme from disk, or return default `Retro` on error/missing.
pub fn load_theme() -> GameTheme {
    load_theme_from_path(THEME_CONFIG_FILE)
}

/// Helper for loading theme from an arbitrary path (unit-testable).
pub fn load_theme_from_path<P: AsRef<Path>>(path: P) -> GameTheme {
    if path.as_ref().exists() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(stored) = serde_json::from_str::<StoredTheme>(&content) {
                return stored.theme;
            }
        }
    }
    GameTheme::Retro
}

/// Save the selected theme to disk.
pub fn save_theme(theme: GameTheme) -> std::io::Result<()> {
    save_theme_to_path(THEME_CONFIG_FILE, theme)
}

/// Helper for saving theme to an arbitrary path (unit-testable).
pub fn save_theme_to_path<P: AsRef<Path>>(path: P, theme: GameTheme) -> std::io::Result<()> {
    let stored = StoredTheme { theme };
    let json = serde_json::to_string_pretty(&stored)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(path, json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_is_retro() {
        assert_eq!(GameTheme::default(), GameTheme::Retro);
        assert_eq!(load_theme_from_path("non_existent_file_123.json"), GameTheme::Retro);
    }

    #[test]
    fn theme_names_and_descriptions() {
        for theme in GameTheme::all() {
            assert!(!theme.name().is_empty());
            assert!(!theme.description().is_empty());
        }
    }

    #[test]
    fn theme_colors_are_unique_per_theme() {
        assert_eq!(GameTheme::all().len(), 4);
        let retro = GameTheme::Retro.colors();
        let arcade = GameTheme::Arcade.colors();
        let pleasant = GameTheme::Pleasant.colors();
        let meadow = GameTheme::Meadow.colors();

        assert_ne!(retro, arcade);
        assert_ne!(retro, pleasant);
        assert_ne!(retro, meadow);
        assert_ne!(arcade, pleasant);
        assert_ne!(arcade, meadow);
        assert_ne!(pleasant, meadow);
        assert!(meadow.outline.is_some());
    }

    #[test]
    fn theme_save_and_load_round_trip() {
        let test_file = "test_theme_roundtrip.json";
        std::fs::remove_file(test_file).ok();

        for theme in GameTheme::all() {
            save_theme_to_path(test_file, theme).expect("save must succeed");
            let loaded = load_theme_from_path(test_file);
            assert_eq!(loaded, theme);
        }

        std::fs::remove_file(test_file).ok();
    }

    #[test]
    fn corrupt_theme_file_falls_back_to_retro() {
        let test_file = "test_theme_corrupt.json";
        fs::write(test_file, "{ not valid json").expect("write must succeed");
        assert_eq!(load_theme_from_path(test_file), GameTheme::Retro);
        std::fs::remove_file(test_file).ok();
    }
}
