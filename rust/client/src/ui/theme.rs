use iced::{application, Background, Color, overlay, Renderer, Theme};
use iced::theme::{Palette, palette};
use iced::widget::{button, checkbox, container, pick_list, rule, scrollable, text, text_input};
use iced_aw::date_picker::Appearance;
use iced_aw::style::date_picker;

pub type GauntletRenderer = Renderer<GauntletTheme>;

pub type Element<'a, Message> = iced::Element<'a, Message, GauntletRenderer>;

#[derive(Default)]
pub struct GauntletTheme {
    theme: Theme,
}

impl GauntletTheme {
    pub fn new() -> Self {
        Self {
            theme: Theme::custom(Palette {
                background: iced::color!(0x2A373E),
                text: iced::color!(0xCFE7DF),
                primary: iced::color!(0x77BCBF),
                success: iced::color!(0x659B5E),
                danger: iced::color!(0x6C1B1B),
            })
        }
    }

    pub fn palette(&self) -> Palette {
        self.theme.palette()
    }

    pub fn extended_palette(&self) -> &palette::Extended {
        self.theme.extended_palette()
    }
}

impl application::StyleSheet for GauntletTheme {
    type Style = ();

    fn appearance(&self, _: &Self::Style) -> application::Appearance {
        let palette = self.extended_palette();

        application::Appearance {
            background_color: Color::TRANSPARENT,
            text_color: palette.background.base.text,
        }
    }
}

impl text_input::StyleSheet for GauntletTheme {
    type Style = ();

    fn active(&self, _: &Self::Style) -> text_input::Appearance {
        let palette = self.extended_palette();

        text_input::Appearance {
            background: palette.background.base.color.into(),
            border_radius: 2.0.into(),
            border_width: 1.0,
            border_color: Color::TRANSPARENT,
            icon_color: palette.background.weak.text,
        }
    }

    fn focused(&self, _: &Self::Style) -> text_input::Appearance {
        let palette = self.extended_palette();

        text_input::Appearance {
            background: palette.background.base.color.into(),
            border_radius: 2.0.into(),
            border_width: 1.0,
            border_color: Color::TRANSPARENT,
            icon_color: palette.background.weak.text,
        }
    }

    fn placeholder_color(&self, _: &Self::Style) -> Color {
        let palette = self.extended_palette();

        palette.background.strong.color
    }

    fn value_color(&self, _: &Self::Style) -> Color {
        let palette = self.extended_palette();

        palette.background.base.text
    }

    fn disabled_color(&self, _: &Self::Style) -> Color {
        self.placeholder_color(&())
    }

    fn selection_color(&self, _: &Self::Style) -> Color {
        let palette = self.extended_palette();

        palette.primary.weak.color
    }

    fn hovered(&self, _: &Self::Style) -> text_input::Appearance {
        let palette = self.extended_palette();

        text_input::Appearance {
            background: palette.background.base.color.into(),
            border_radius: 2.0.into(),
            border_width: 1.0,
            border_color: Color::TRANSPARENT,
            icon_color: palette.background.weak.text,
        }
    }

    fn disabled(&self, _: &Self::Style) -> text_input::Appearance {
        let palette = self.extended_palette();

        text_input::Appearance {
            background: palette.background.weak.color.into(),
            border_radius: 2.0.into(),
            border_width: 1.0,
            border_color: Color::TRANSPARENT,
            icon_color: palette.background.strong.color,
        }
    }
}

impl scrollable::StyleSheet for GauntletTheme {
    type Style = ();

    fn active(&self, _: &Self::Style) -> scrollable::Scrollbar {
        let palette = self.extended_palette();

        scrollable::Scrollbar {
            background: Some(palette.background.weak.color.into()),
            border_radius: 2.0.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            scroller: scrollable::Scroller {
                color: palette.background.strong.color,
                border_radius: 2.0.into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }

    fn hovered(&self, _: &Self::Style, is_mouse_over_scrollbar: bool) -> scrollable::Scrollbar {
        if is_mouse_over_scrollbar {
            let palette = self.extended_palette();

            scrollable::Scrollbar {
                background: Some(palette.background.weak.color.into()),
                border_radius: 2.0.into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
                scroller: scrollable::Scroller {
                    color: palette.primary.strong.color,
                    border_radius: 2.0.into(),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
            }
        } else {
            self.active(&())
        }
    }
}


#[derive(Default)]
pub enum ContainerStyle {
    #[default]
    Transparent,
    Background,
    Code,
}

impl container::StyleSheet for GauntletTheme {
    type Style = ContainerStyle;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        match style {
            ContainerStyle::Transparent => Default::default(),
            ContainerStyle::Background => {
                let palette = self.extended_palette();

                container::Appearance {
                    text_color: None,
                    background: Some(palette.background.base.color.into()),
                    border_radius: 10.0.into(),
                    border_width: 1.0,
                    border_color: palette.background.weak.color,
                }
            }
            ContainerStyle::Code => {
                let palette = self.extended_palette();

                container::Appearance {
                    text_color: None,
                    background: Some(palette.background.weak.color.into()),
                    border_radius: 4.0.into(),
                    border_width: 1.0,
                    border_color: palette.background.weak.color,
                }
            }
        }
    }
}

impl rule::StyleSheet for GauntletTheme {
    type Style = ();

    fn appearance(&self, _: &Self::Style) -> rule::Appearance {
        let palette = self.extended_palette();

        rule::Appearance {
            color: palette.background.strong.color,
            width: 1,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
        }
    }
}

#[derive(Default, Clone)]
pub enum TextStyle {
    #[default]
    Default,
    Subtext,
}


impl text::StyleSheet for GauntletTheme {
    type Style = TextStyle;

    fn appearance(&self, style: Self::Style) -> text::Appearance {
        match style {
            TextStyle::Default => Default::default(),
            TextStyle::Subtext => {
                let palette = self.extended_palette();

                let color = palette.background.base.text;

                text::Appearance {
                    color: Some(Color::new(color.r, color.g, color.b, 0.4)),
                }
            }
        }
    }
}

impl date_picker::StyleSheet for GauntletTheme {
    type Style = ();

    fn active(&self, _: &Self::Style) -> Appearance {
        let palette = self.extended_palette();
        let foreground = self.palette();

        Appearance {
            background: palette.background.base.color.into(),
            border_radius: 15.0,
            border_width: 1.0,
            border_color: foreground.text,
            text_color: foreground.text,
            text_attenuated_color: Color {
                a: foreground.text.a * 0.5,
                ..foreground.text
            },
            day_background: palette.background.base.color.into(),
        }
    }

    fn selected(&self, style: &Self::Style) -> Appearance {
        let palette = self.extended_palette();

        Appearance {
            day_background: palette.primary.strong.color.into(),
            text_color: palette.primary.strong.text,
            ..self.active(style)
        }
    }

    fn hovered(&self, style: &Self::Style) -> Appearance {
        let palette = self.extended_palette();

        Appearance {
            day_background: palette.primary.weak.color.into(),
            text_color: palette.primary.weak.text,
            ..self.active(style)
        }
    }

    fn focused(&self, style: &Self::Style) -> Appearance {
        Appearance {
            border_color: Color::from_rgb(0.5, 0.5, 0.5),
            ..self.active(style)
        }
    }
}

impl checkbox::StyleSheet for GauntletTheme {
    type Style = ();

    fn active(&self, _: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        let palette = self.extended_palette();

        checkbox_appearance(
            palette.primary.strong.text,
            palette.background.base,
            palette.primary.strong,
            is_checked,
        )
    }

    fn hovered(&self, _: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        let palette = self.extended_palette();

        checkbox_appearance(
            palette.primary.strong.text,
            palette.background.weak,
            palette.primary.base,
            is_checked,
        )
    }
}

fn checkbox_appearance(
    icon_color: Color,
    base: palette::Pair,
    accent: palette::Pair,
    is_checked: bool,
) -> checkbox::Appearance {
    checkbox::Appearance {
        background: Background::Color(if is_checked {
            accent.color
        } else {
            base.color
        }),
        icon_color,
        border_radius: 2.0.into(),
        border_width: 1.0,
        border_color: accent.color,
        text_color: None,
    }
}

impl pick_list::StyleSheet for GauntletTheme {
    type Style = ();

    fn active(&self, _: &Self::Style) -> pick_list::Appearance {
        let palette = self.extended_palette();

        pick_list::Appearance {
            text_color: palette.background.weak.text,
            background: palette.background.weak.color.into(),
            placeholder_color: palette.background.strong.color,
            handle_color: palette.background.weak.text,
            border_radius: 2.0.into(),
            border_width: 1.0,
            border_color: palette.background.strong.color,
        }
    }

    fn hovered(&self, _: &Self::Style) -> pick_list::Appearance {
        let palette = self.extended_palette();

        pick_list::Appearance {
            text_color: palette.background.weak.text,
            background: palette.background.weak.color.into(),
            placeholder_color: palette.background.strong.color,
            handle_color: palette.background.weak.text,
            border_radius: 2.0.into(),
            border_width: 1.0,
            border_color: palette.primary.strong.color,
        }
    }
}

impl overlay::menu::StyleSheet for GauntletTheme {
    type Style = ();

    fn appearance(&self, _: &Self::Style) -> overlay::menu::Appearance {
        let palette = self.extended_palette();

        overlay::menu::Appearance {
            text_color: palette.background.weak.text,
            background: palette.background.weak.color.into(),
            border_width: 1.0,
            border_radius: 0.0.into(),
            border_color: palette.background.strong.color,
            selected_text_color: palette.primary.strong.text,
            selected_background: palette.primary.strong.color.into(),
        }
    }
}


#[derive(Default)]
pub enum ButtonStyle {
    #[default]
    Primary,
    Secondary,
    Positive,
    Destructive,
    Link,
    EntrypointItem,
}

impl button::StyleSheet for GauntletTheme {
    type Style = ButtonStyle;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let palette = self.extended_palette();

        let appearance = button::Appearance {
            border_radius: 2.0.into(),
            ..button::Appearance::default()
        };

        let from_pair = |pair: palette::Pair| button::Appearance {
            background: Some(pair.color.into()),
            text_color: pair.text,
            ..appearance
        };

        match style {
            ButtonStyle::Primary => from_pair(palette.primary.strong),
            ButtonStyle::Secondary => from_pair(palette.secondary.base),
            ButtonStyle::Positive => from_pair(palette.success.base),
            ButtonStyle::Destructive => from_pair(palette.danger.base),
            ButtonStyle::Link => button::Appearance {
                text_color: palette.background.weak.text,
                ..appearance
            },
            ButtonStyle::EntrypointItem => button::Appearance {
                background: None,
                text_color: palette.background.base.text,
                ..appearance
            }
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let palette = self.extended_palette();

        let appearance = button::Appearance {
            border_radius: 2.0.into(),
            ..button::Appearance::default()
        };

        match style {
            ButtonStyle::EntrypointItem => button::Appearance {
                background: Some(palette.background.weak.color.into()),
                text_color: palette.secondary.base.text,
                ..appearance
            },
            _ => self.active(style)
        }
    }
}
