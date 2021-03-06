#![windows_subsystem = "windows"]
mod amoled_image;

use amoled_image::AmoledImageConverter;

use std::path::PathBuf;

use iced::{
    button, slider, text_input, window, Align, Button, Column, Container, Element,
    HorizontalAlignment, Length, Row, Sandbox, Settings, Slider, Text, TextInput,
};

use rfd::FileDialog;

pub fn main() -> iced::Result {
    Amoled::run(Settings {
        window: (window::Settings {
            size: (600, 400),
            min_size: Some((600, 400)),
            resizable: true,
            ..window::Settings::default()
        }),
        ..Settings::default()
    })
}

#[derive(Default)]
struct Amoled {
    black_point: u8,
    black_point_slider: slider::State,
    black_point_input: text_input::State,
    path_input: text_input::State,
    path_input_value: Option<PathBuf>,
    file_open_button: button::State,
    file_save_button: button::State,
    image: Option<AmoledImageConverter>,
}

#[derive(Debug, Clone)]
pub enum Message {
    PathChanged(String),
    BlackPointChanged(u8),
    BlackPointInputChanged(String),
    FileButtonpressed,
    SaveButtonPressed,
}

impl Amoled {
    fn handle_image_change(&mut self) {
        if let Some(path) = &self.path_input_value {
            println!("making new image");
            self.image = AmoledImageConverter::from_path(&path, self.black_point).ok();
        }
    }
}

impl Sandbox for Amoled {
    type Message = Message;

    fn new() -> Amoled {
        Amoled::default()
    }

    fn title(&self) -> String {
        let file_name = self
            .path_input_value
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|file_name| file_name.to_str());

        match file_name {
            Some(name) => format!("Amoled Maker - {}", name),
            None => "Amoled Maker".to_string(),
        }
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::BlackPointChanged(bp) => {
                self.black_point = bp;

                if let Some(img) = self.image.as_mut() {
                    img.set_black_point(bp);
                }
            }
            Message::PathChanged(path_string) => {
                self.path_input_value = Some(PathBuf::from(path_string));
                self.handle_image_change();
            }
            Message::FileButtonpressed => {
                self.path_input_value = match FileDialog::new()
                    .add_filter("image", &["png", "jpg", "jpeg"])
                    .pick_file()
                {
                    Some(new_path) => Some(new_path),
                    None => self.path_input_value.to_owned(),
                };
                self.handle_image_change();
            }
            Message::BlackPointInputChanged(bp_string) => {
                // Only update the text input if it is a u8 value or set it to 0 if it is empty.
                if bp_string.eq("") {
                    self.update(Message::BlackPointChanged(0));
                } else if let Ok(bp) = bp_string.parse::<u8>() {
                    self.update(Message::BlackPointChanged(bp));
                }
            }
            Message::SaveButtonPressed => {
                println!("Test");
                let potential_save_path = FileDialog::new().add_filter("png", &["png"]).save_file();
                if let Some(save_path) = potential_save_path {
                    if let Some(image) = self.image.as_mut() {
                        // todo: handle the error here instead of unwrapping
                        println!("Saving file");
                        image::save_buffer(
                            save_path,
                            &image.as_rgba_image(),
                            image.get_width(),
                            image.get_height(),
                            image::ColorType::Rgba8,
                        ).unwrap();
                    }
                }
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let title = Text::new("amoled maker")
            .size(75)
            .color([0.6, 0.6, 0.6])
            .horizontal_alignment(HorizontalAlignment::Center)
            .width(Length::Fill);

        let path_input = Row::new()
            .spacing(10)
            .push(
                TextInput::new(
                    &mut self.path_input,
                    "Please select a file",
                    match &mut self.path_input_value {
                        Some(path) => path.to_str().unwrap_or(""),
                        None => "",
                    },
                    Message::PathChanged,
                )
                .padding(10)
                .width(Length::Fill)
                .size(20),
            )
            .push(
                Button::new(&mut self.file_open_button, Text::new("Open"))
                    .padding(10)
                    .on_press(Message::FileButtonpressed),
            )
            .max_width(500);

        let top_container = Container::new(
            Column::new()
                .spacing(20)
                .push(title)
                .push(path_input)
                .push(
                    Row::new()
                        .push(
                            Slider::new(
                                &mut self.black_point_slider,
                                0..=255,
                                self.black_point,
                                Message::BlackPointChanged,
                            )
                            .width(Length::Units(450)),
                        )
                        .push(
                            TextInput::new(
                                &mut self.black_point_input,
                                "0",
                                &self.black_point.to_string(),
                                Message::BlackPointInputChanged,
                            )
                            .width(Length::Units(40))
                            .size(20),
                        )
                        .spacing(20)
                        .padding(5),
                )
                .align_items(Align::Center),
        );

        let content = Column::new().push(top_container).align_items(Align::Center);

        if let Some(img) = self.image.as_mut() {
            let mut content = content.push(
                Column::new()
                    .push(img.view())
                    .push(
                        Button::new(&mut self.file_save_button, Text::new("Save"))
                            .on_press(Message::SaveButtonPressed),
                    )
                    .align_items(Align::Center)
                    .spacing(20)
                    .padding(20),
            );
            content = content.push(Row::new().height(Length::Fill));
            return Container::new(content)
                .center_x()
                .center_y()
                .into();
        }

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
