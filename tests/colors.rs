#[cfg(test)]
mod tests {
    use nu_ansi_term::Color;
    use thag_rs::colors::{
        ColorSupport, MessageStyle, NuColor, NuThemeStyle, TermTheme, XtermColor, COLOR_SUPPORT,
    };
    use thag_rs::logging::Verbosity;
    use thag_rs::{log, nu_color_println, nu_resolve_style, MessageLevel};
    use supports_color::Stream;
    use termbg::Theme;

    // Set environment variables before running tests
    fn set_up() {
        std::env::set_var("TEST_ENV", "1");
        std::env::set_var("VISUAL", "cat");
        std::env::set_var("EDITOR", "cat");
    }

    fn convert_theme(theme1: &Theme) -> TermTheme {
        set_up();
        // Define how the equality is determined for `Theme`
        match theme1 {
            Theme::Light => TermTheme::Light,
            Theme::Dark => TermTheme::Dark,
        }
    }

    // #[ignore]
    #[test]
    // supports_color::on(Stream) causes rightward drift
    fn test_color_support() {
        set_up();
        let color_level = supports_color::on(Stream::Stdout);
        thag_rs::clear_screen();
        let color_support = match color_level {
            Some(color_level) => {
                if color_level.has_16m || color_level.has_256 {
                    Some(ColorSupport::Xterm256)
                } else {
                    Some(ColorSupport::Ansi16)
                }
            }
            None => None,
        };

        match color_support {
            Some(ColorSupport::Xterm256) => {
                assert!(color_level.unwrap().has_16m || color_level.unwrap().has_256);
            }
            Some(ColorSupport::Ansi16) => {
                assert!(!color_level.unwrap().has_16m && !color_level.unwrap().has_256);
            }
            Some(ColorSupport::None) => assert!(color_level.is_none()),
            None => {
                assert!(color_level.is_none());
            }
        }
    }

    #[test]
    fn test_term_theme() {
        // Test if TERM_THEME is set correctly
        set_up();
        // Example test using the manual comparison function
        // let theme =
        //     termbg::theme(std::time::Duration::from_millis(100)).expect("Error getting theme");
        for theme in &[Theme::Light, Theme::Dark] {
            match theme {
                Theme::Light => assert_eq!(convert_theme(theme), TermTheme::Light),
                Theme::Dark => assert_eq!(convert_theme(theme), TermTheme::Dark),
                // Add other cases here if needed
            }
        }
    }

    #[test]
    fn test_message_style_display() {
        // Test the Display trait for MessageStyle
        set_up();
        let style = MessageStyle::Ansi16LightError;
        assert_eq!(style.to_string(), "ansi16_light_error");

        let style = MessageStyle::Xterm256DarkEmphasis;
        assert_eq!(style.to_string(), "xterm256_dark_emphasis");
    }

    #[test]
    fn test_nu_color_get_color() {
        // Test the get_color method for XtermColor
        set_up();
        let color = XtermColor::GuardsmanRed;
        assert_eq!(color.get_color(), Color::Fixed(160));
    }

    #[test]
    fn test_nu_resolve_style() {
        set_up();
        // Test the nu_resolve_style function
        // Causes rightward drift of the test result printouts.
        let theme = termbg::theme(std::time::Duration::from_millis(100));
        // print!("{}[2J", 27 as char);
        thag_rs::clear_screen();

        let style = nu_resolve_style(MessageLevel::Warning);
        if let Some(color_support) = COLOR_SUPPORT.as_ref() {
            match theme {
                Ok(Theme::Light) => match color_support {
                    ColorSupport::Xterm256 => {
                        let expected_style = XtermColor::DarkPurplePizzazz.get_color().bold();
                        assert_eq!(style, expected_style);
                    }
                    ColorSupport::Ansi16 => {
                        let expected_style = Color::Magenta.bold();
                        assert_eq!(style, expected_style);
                    }
                    ColorSupport::None => assert_eq!(style, nu_ansi_term::Style::default()),
                },
                Ok(Theme::Dark) | Err(_) => match color_support {
                    ColorSupport::Xterm256 => {
                        let expected_style = XtermColor::DarkViolet.get_color().bold();
                        assert_eq!(style, expected_style);
                    }
                    ColorSupport::Ansi16 => {
                        let expected_style = Color::Magenta.bold();
                        assert_eq!(style, expected_style);
                    }
                    ColorSupport::None => assert_eq!(style, nu_ansi_term::Style::default()),
                },
            }
        } else {
            assert_eq!(style, nu_ansi_term::Style::default());
        }
    }

    #[test]
    fn test_message_style_get_style() {
        // Test the get_style method for MessageStyle
        set_up();
        let style = MessageStyle::Ansi16LightError.get_style();
        assert_eq!(style, Color::Red.bold());

        let style = MessageStyle::Xterm256DarkEmphasis.get_style();
        assert_eq!(style, XtermColor::Copperfield.get_color().bold());
    }

    #[test]
    fn test_nu_color_println_macro() {
        // Test the nu_color_println macro
        set_up();
        let content = "Test message";
        let output = format!("\u{1b}[1m{content}\u{1b}[0m");
        let style = nu_ansi_term::Style::new().bold();
        nu_color_println!(style, "{}", content);
        thag_rs::clear_screen();

        // Ensure the macro output is correctly styled
        assert_eq!(output, format!("{}", style.paint(content)));
    }
}
