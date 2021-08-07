// use iced::{button, Color};

// fn color_from_rgb(r: i32, g: i32, b: i32) -> Color {
//     Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
// }

// const BLACK: Color = color_from_rgb(0x24, 0x23, 0x31);
// const ACTIVE: Color = color_from_rgb(0x79, 0x9F, 0xBC);
// const WHITE: Color = color_from_rgb(0xF4, 0xEC, 0xD6);

// pub struct Button;

// impl From<Theme> for Box<dyn button::StyleSheet> {
//     fn from(theme: Theme) -> Self {
//         self::Button.into()
//     }
// }

// impl button::StyleSheet for Button {
//     fn active(&self) -> button::Style {
//         button::Style {
//             background: ACTIVE.into(),
//             border_radius: 3.0,
//             text_color: WHITE,
//             ..button::Style::default()
//         }
//     }
// }
