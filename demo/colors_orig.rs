/*[toml]
[dependencies]
crossterm = "0.27.0"
log = "0.4.21"
owo-colors = { version = "4.0.0", features = ["supports-colors"] }
rs-script = { path = "/Users/donf/projects/rs-script" }

strum = { version = "0.26.2", features = ["derive"] }
supports-color= "3.0.0"
termbg = "0.5.0"
*/

use log::debug;
use owo_ansi::xterm as owo_xterm;
use owo_ansi::{Blue, Cyan, Green, Red, White, Yellow};
use owo_colors::colors::{self as owo_ansi, Magenta};
use owo_colors::{AnsiColors, Style, XtermColors};
use owo_xterm::Black;
use rs_script::{clear_screen, debug_log};
use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};
use supports_color::Stream;
use termbg::{Error, Theme};

/// Original prototype of `rs-script`'s `colors` module to style messages according
/// to their type. I only dropped `owo-colors` because I switched from `rustyline` to
/// `reedline`, which was already using `nu_ansi_term`.
///
/// Format: `color_println`!(style: Option<Style>, "Lorem ipsum dolor {} amet", content: &str);
#[macro_export]
macro_rules! color_println {
    ($style:expr, $($arg:tt)*) => {{
        let content = format!("{}", format_args!($($arg)*));
        if let Some(style) = $style {
                // Qualified form to avoid imports in calling code.
                println!("{}", owo_colors::Style::style(&style, content));
        } else {
            println!("{}", content);
        }
    }};
}

pub trait ThemeStyle {
    fn get_style(&self) -> Option<Style>;
    fn to_string(&self) -> String;
}

// Enum for light and dark theme styles
#[derive(Clone, Copy, EnumIter, IntoStaticStr)]
pub enum YinYangStyle {
    Error,
    Warning,
    Emphasis,
    OuterPrompt,
    InnerPrompt,
    Info,
    Debug,
}

impl ThemeStyle for YinYangStyle {
    // Get the corresponding color style for the message type
    fn get_style(&self) -> Option<Style> {
        let theme_result = get_theme();
        if let Ok(theme) = theme_result {
            let style = match theme {
                Theme::Light => match *self {
                    YinYangStyle::Error => Style::new().fg::<Red>().bold(),
                    YinYangStyle::Warning => Style::new().fg::<Magenta>().bold(),
                    YinYangStyle::Emphasis => Style::new().fg::<Yellow>().bold(),
                    YinYangStyle::OuterPrompt => Style::new().fg::<Blue>().bold(),
                    YinYangStyle::InnerPrompt => Style::new().fg::<Cyan>().bold(),
                    YinYangStyle::Info => Style::new().fg::<Black>(),
                    YinYangStyle::Debug => Style::new().fg::<Cyan>(),
                },
                Theme::Dark => match *self {
                    YinYangStyle::Error => Style::new().fg::<Red>().bold(),
                    YinYangStyle::Warning => Style::new().fg::<Magenta>().bold(),
                    YinYangStyle::Emphasis => Style::new().fg::<Yellow>().bold(),
                    YinYangStyle::OuterPrompt => Style::new().fg::<Blue>().bold(),
                    YinYangStyle::InnerPrompt => Style::new().fg::<Green>().bold(),
                    YinYangStyle::Info => Style::new().fg::<White>(),
                    YinYangStyle::Debug => Style::new().fg::<Cyan>(),
                },
            };
            Some(style)
        } else if let Some(_support) = supports_color::on(Stream::Stdout) {
            // If supports colour, default to dark theme - safer
            let style: Style = match *self {
                YinYangStyle::Error => Style::new().fg::<Red>().bold(),
                YinYangStyle::Warning => Style::new().fg::<Magenta>().bold(),
                YinYangStyle::Emphasis => Style::new().fg::<Yellow>().bold(),
                YinYangStyle::OuterPrompt => Style::new().fg::<Blue>().bold(),
                YinYangStyle::InnerPrompt => Style::new().fg::<Green>().bold(),
                YinYangStyle::Info => Style::new().fg::<White>(),
                YinYangStyle::Debug => Style::new().fg::<Cyan>(),
            };
            Some(style)
        } else {
            None
        }
    }

    fn to_string(&self) -> String {
        match *self {
            YinYangStyle::Error => String::from("error"),
            YinYangStyle::Warning => String::from("warning"),
            YinYangStyle::Emphasis => String::from("emphasis"),
            YinYangStyle::OuterPrompt => String::from("outer_prompt"),
            YinYangStyle::InnerPrompt => String::from("inner_prompt"),
            YinYangStyle::Info => String::from("info"),
            YinYangStyle::Debug => String::from("debug"),
        }
    }
}

#[allow(dead_code)]
fn main() {
    let term = termbg::terminal();
    clear_screen();
    debug!("  Term : {:?}", term);

    let theme = get_theme();

    // Get the appropriate style based on the theme
    let borrowed_theme = theme.as_ref();

    if let Ok(theme_ref) = borrowed_theme {
        for variant in YinYangStyle::iter() {
            let level: &str = &variant.to_string();
            // let borrowed_theme = theme.as_ref();
            // if let Ok(theme_ref) = borrowed_theme {
            color_println!(
                variant.get_style(),
                "My {theme_ref:?} style {} message",
                level
            );
        }
    } else if let Some(_support) = supports_color::on(Stream::Stdout) {
        // if support.has_16m {
        //     println!("16 million (RGB) colors are supported");
        // } else if support.has_256 {
        //     println!("256 colors are supported.");
        // } else if support.has_basic {
        //     println!("Only basic ANSI colors are supported.");
        // }
        for variant in YinYangStyle::iter() {
            let level: &str = &variant.to_string();
            color_println!(
                variant.get_style(),
                "My unknown theme (defaulting to Dark) style {} message",
                level
            );
        }
    } else {
        println!("My warning message - no color support");
    }

    println!();

    if let Some(support) = supports_color::on(Stream::Stdout) {
        if support.has_16m || support.has_256 {
            print_xterm_colors();
        } else if support.has_basic {
            print_ansi_colors();
        }
    } else {
        println!("No color support.");
    }
}

#[allow(clippy::too_many_lines)]
fn print_xterm_colors() {
    let style = Style::new();
    for variant in &[
        XtermColors::UserBlack,
        XtermColors::UserRed,
        XtermColors::UserGreen,
        XtermColors::UserYellow,
        XtermColors::UserBlue,
        XtermColors::UserMagenta,
        XtermColors::UserCyan,
        XtermColors::UserWhite,
        XtermColors::UserBrightBlack,
        XtermColors::UserBrightRed,
        XtermColors::UserBrightGreen,
        XtermColors::UserBrightYellow,
        XtermColors::UserBrightBlue,
        XtermColors::UserBrightMagenta,
        XtermColors::UserBrightCyan,
        XtermColors::UserBrightWhite,
        XtermColors::Black,
        XtermColors::StratosBlue,
        XtermColors::NavyBlue,
        XtermColors::MidnightBlue,
        XtermColors::DarkBlue,
        XtermColors::Blue,
        XtermColors::CamaroneGreen,
        XtermColors::BlueStone,
        XtermColors::OrientBlue,
        XtermColors::EndeavourBlue,
        XtermColors::ScienceBlue,
        XtermColors::BlueRibbon,
        XtermColors::JapaneseLaurel,
        XtermColors::DeepSeaGreen,
        XtermColors::Teal,
        XtermColors::DeepCerulean,
        XtermColors::LochmaraBlue,
        XtermColors::AzureRadiance,
        XtermColors::LightJapaneseLaurel,
        XtermColors::Jade,
        XtermColors::PersianGreen,
        XtermColors::BondiBlue,
        XtermColors::Cerulean,
        XtermColors::LightAzureRadiance,
        XtermColors::DarkGreen,
        XtermColors::Malachite,
        XtermColors::CaribbeanGreen,
        XtermColors::LightCaribbeanGreen,
        XtermColors::RobinEggBlue,
        XtermColors::Aqua,
        XtermColors::Green,
        XtermColors::DarkSpringGreen,
        XtermColors::SpringGreen,
        XtermColors::LightSpringGreen,
        XtermColors::BrightTurquoise,
        XtermColors::Cyan,
        XtermColors::Rosewood,
        XtermColors::PompadourMagenta,
        XtermColors::PigmentIndigo,
        XtermColors::DarkPurple,
        XtermColors::ElectricIndigo,
        XtermColors::ElectricPurple,
        XtermColors::VerdunGreen,
        XtermColors::ScorpionOlive,
        XtermColors::Lilac,
        XtermColors::ScampiIndigo,
        XtermColors::Indigo,
        XtermColors::DarkCornflowerBlue,
        XtermColors::DarkLimeade,
        XtermColors::GladeGreen,
        XtermColors::JuniperGreen,
        XtermColors::HippieBlue,
        XtermColors::HavelockBlue,
        XtermColors::CornflowerBlue,
        XtermColors::Limeade,
        XtermColors::FernGreen,
        XtermColors::SilverTree,
        XtermColors::Tradewind,
        XtermColors::ShakespeareBlue,
        XtermColors::DarkMalibuBlue,
        XtermColors::DarkBrightGreen,
        XtermColors::DarkPastelGreen,
        XtermColors::PastelGreen,
        XtermColors::DownyTeal,
        XtermColors::Viking,
        XtermColors::MalibuBlue,
        XtermColors::BrightGreen,
        XtermColors::DarkScreaminGreen,
        XtermColors::ScreaminGreen,
        XtermColors::DarkAquamarine,
        XtermColors::Aquamarine,
        XtermColors::LightAquamarine,
        XtermColors::Maroon,
        XtermColors::DarkFreshEggplant,
        XtermColors::LightFreshEggplant,
        XtermColors::Purple,
        XtermColors::ElectricViolet,
        XtermColors::LightElectricViolet,
        XtermColors::Brown,
        XtermColors::CopperRose,
        XtermColors::StrikemasterPurple,
        XtermColors::DelugePurple,
        XtermColors::DarkMediumPurple,
        XtermColors::DarkHeliotropePurple,
        XtermColors::Olive,
        XtermColors::ClayCreekOlive,
        XtermColors::DarkGray,
        XtermColors::WildBlueYonder,
        XtermColors::ChetwodeBlue,
        XtermColors::SlateBlue,
        XtermColors::LightLimeade,
        XtermColors::ChelseaCucumber,
        XtermColors::BayLeaf,
        XtermColors::GulfStream,
        XtermColors::PoloBlue,
        XtermColors::LightMalibuBlue,
        XtermColors::Pistachio,
        XtermColors::LightPastelGreen,
        XtermColors::DarkFeijoaGreen,
        XtermColors::VistaBlue,
        XtermColors::Bermuda,
        XtermColors::DarkAnakiwaBlue,
        XtermColors::ChartreuseGreen,
        XtermColors::LightScreaminGreen,
        XtermColors::DarkMintGreen,
        XtermColors::MintGreen,
        XtermColors::LighterAquamarine,
        XtermColors::AnakiwaBlue,
        XtermColors::AeroBlue,
        XtermColors::BrightRed,
        XtermColors::DarkFlirt,
        XtermColors::Flirt,
        XtermColors::LightFlirt,
        XtermColors::DarkViolet,
        XtermColors::BrightElectricViolet,
        XtermColors::RoseofSharonOrange,
        XtermColors::MatrixPink,
        XtermColors::TapestryPink,
        XtermColors::FuchsiaPink,
        XtermColors::MediumPurple,
        XtermColors::Heliotrope,
        XtermColors::PirateGold,
        XtermColors::MuesliOrange,
        XtermColors::PharlapPink,
        XtermColors::Bouquet,
        XtermColors::Lavender,
        XtermColors::LightHeliotrope,
        XtermColors::BuddhaGold,
        XtermColors::OliveGreen,
        XtermColors::HillaryOlive,
        XtermColors::SilverChalice,
        XtermColors::WistfulLilac,
        XtermColors::MelroseLilac,
        XtermColors::RioGrandeGreen,
        XtermColors::ConiferGreen,
        XtermColors::Feijoa,
        XtermColors::PixieGreen,
        XtermColors::JungleMist,
        XtermColors::LightAnakiwaBlue,
        XtermColors::Lime,
        XtermColors::GreenYellow,
        XtermColors::LightMintGreen,
        XtermColors::Celadon,
        XtermColors::FrenchPassLightBlue,
        XtermColors::GuardsmanRed,
        XtermColors::RazzmatazzCerise,
        XtermColors::MediumVioletRed,
        XtermColors::HollywoodCerise,
        XtermColors::DarkPurplePizzazz,
        XtermColors::BrighterElectricViolet,
        XtermColors::TennOrange,
        XtermColors::RomanOrange,
        XtermColors::CranberryPink,
        XtermColors::HopbushPink,
        XtermColors::Orchid,
        XtermColors::LighterHeliotrope,
        XtermColors::MangoTango,
        XtermColors::Copperfield,
        XtermColors::SeaPink,
        XtermColors::CanCanPink,
        XtermColors::LightOrchid,
        XtermColors::BrightHeliotrope,
        XtermColors::DarkCorn,
        XtermColors::DarkTachaOrange,
        XtermColors::TanBeige,
        XtermColors::ClamShell,
        XtermColors::ThistlePink,
        XtermColors::Mauve,
        XtermColors::Corn,
        XtermColors::TachaOrange,
        XtermColors::DecoOrange,
        XtermColors::PaleGoldenrod,
        XtermColors::AltoBeige,
        XtermColors::FogPink,
        XtermColors::ChartreuseYellow,
        XtermColors::Canary,
        XtermColors::Honeysuckle,
        XtermColors::ReefPaleYellow,
        XtermColors::SnowyMint,
        XtermColors::OysterBay,
        XtermColors::Red,
        XtermColors::DarkRose,
        XtermColors::Rose,
        XtermColors::LightHollywoodCerise,
        XtermColors::PurplePizzazz,
        XtermColors::Fuchsia,
        XtermColors::BlazeOrange,
        XtermColors::BittersweetOrange,
        XtermColors::WildWatermelon,
        XtermColors::DarkHotPink,
        XtermColors::HotPink,
        XtermColors::PinkFlamingo,
        XtermColors::FlushOrange,
        XtermColors::Salmon,
        XtermColors::VividTangerine,
        XtermColors::PinkSalmon,
        XtermColors::DarkLavenderRose,
        XtermColors::BlushPink,
        XtermColors::YellowSea,
        XtermColors::TexasRose,
        XtermColors::Tacao,
        XtermColors::Sundown,
        XtermColors::CottonCandy,
        XtermColors::LavenderRose,
        XtermColors::Gold,
        XtermColors::Dandelion,
        XtermColors::GrandisCaramel,
        XtermColors::Caramel,
        XtermColors::CosmosSalmon,
        XtermColors::PinkLace,
        XtermColors::Yellow,
        XtermColors::LaserLemon,
        XtermColors::DollyYellow,
        XtermColors::PortafinoYellow,
        XtermColors::Cumulus,
        XtermColors::White,
        XtermColors::DarkCodGray,
        XtermColors::CodGray,
        XtermColors::LightCodGray,
        XtermColors::DarkMineShaft,
        XtermColors::MineShaft,
        XtermColors::LightMineShaft,
        XtermColors::DarkTundora,
        XtermColors::Tundora,
        XtermColors::ScorpionGray,
        XtermColors::DarkDoveGray,
        XtermColors::DoveGray,
        XtermColors::Boulder,
        XtermColors::Gray,
        XtermColors::LightGray,
        XtermColors::DustyGray,
        XtermColors::NobelGray,
        XtermColors::DarkSilverChalice,
        XtermColors::LightSilverChalice,
        XtermColors::DarkSilver,
        XtermColors::Silver,
        XtermColors::DarkAlto,
        XtermColors::Alto,
        XtermColors::Mercury,
        XtermColors::GalleryGray,
    ] {
        let style = style.color(*variant);

        debug_log!("style={style:?}");
        color_println!(Some(style), "My Xterm {variant:?} style message");
        let style = style.color(*variant).bold();
        debug_log!("style={style:?}");
        color_println!(Some(style), "My bold Xterm {variant:?} style message");
    }
}

fn print_ansi_colors() {
    let style = Style::new();
    for variant in &[
        AnsiColors::Black,
        AnsiColors::Red,
        AnsiColors::Green,
        AnsiColors::Yellow,
        AnsiColors::Magenta,
        AnsiColors::Blue,
        AnsiColors::Cyan,
        AnsiColors::White,
        AnsiColors::Default,
        AnsiColors::BrightBlack,
        AnsiColors::BrightRed,
        AnsiColors::BrightGreen,
        AnsiColors::BrightYellow,
        AnsiColors::BrightBlue,
        AnsiColors::BrightMagenta,
        AnsiColors::BrightCyan,
        AnsiColors::BrightWhite,
    ] {
        let style = style.color(*variant);

        debug_log!("style={style:?}");
        color_println!(Some(style), "My Ansi {variant:?} style message");
        let style = style.color(*variant).bold();
        debug_log!("style={style:?}");
        color_println!(Some(style), "My bold Ansi {variant:?} style message");
    }
}

fn get_theme() -> Result<Theme, Error> {
    let timeout = std::time::Duration::from_millis(100);

    debug_log!("Check terminal background color");
    let theme: Result<Theme, Error> = termbg::theme(timeout);
    clear_screen();
    theme
}
