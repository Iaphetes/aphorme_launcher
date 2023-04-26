#[cfg(feature = "iced-ui")]
pub mod iced_ui {

    use crate::apps::ApplicationManager;
    use crate::config::GuiCFG;
    use iced::widget::scrollable::{Properties, Scrollbar, Scroller};
    use iced::widget::{column, container, scrollable, text, Column};
    use iced::{executor, theme, window, Alignment, Color};
    use iced::{Application, Command, Element, Length, Settings, Theme};
    use once_cell::sync::Lazy;

    static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

    #[derive(Default)]
    pub struct State {
        pub gui_cfg: GuiCFG,
        pub application_manager: ApplicationManager,
    }
    pub fn launch_iced_ui(gui_cfg: GuiCFG, application_manager: ApplicationManager) {
        let _ = ScrollableDemo::run(Settings {
            flags: State {
                gui_cfg,
                application_manager,
            },
            window: window::Settings {
                size: (512, 512),
                decorations: false,
                always_on_top: true,
                resizable: false,
                ..Default::default()
            },
            ..Settings::default()
        });
    }
    pub struct ScrollableDemo {
        current_scroll_offset: scrollable::RelativeOffset,
        gui_cfg: GuiCFG,
        application_manager: ApplicationManager,
    }

    #[derive(Debug, Clone)]
    pub enum Message {
        Scrolled(scrollable::RelativeOffset),
    }

    impl Application for ScrollableDemo {
        type Executor = executor::Default;
        type Message = Message;
        type Theme = Theme;
        type Flags = State;

        fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
            (
                ScrollableDemo {
                    current_scroll_offset: scrollable::RelativeOffset::START,
                    gui_cfg: _flags.gui_cfg,
                    application_manager: _flags.application_manager,
                },
                Command::none(),
            )
        }

        fn title(&self) -> String {
            String::from("Scrollable - Iced")
        }

        fn update(&mut self, message: Message) -> Command<Message> {
            match message {
                Message::Scrolled(offset) => {
                    self.current_scroll_offset = offset;

                    Command::none()
                }
            }
        }

        fn view(&self) -> Element<Message> {
            let mut column = Column::new();
            for application in &self.application_manager.matches {
                column = column.push(container(text(application.0.name.clone())).style(
                    theme::Container::Custom(Box::new(ContainerNotSelectedStyle)),
                ));
            }
            let scrollable_content: Element<Message> = Element::from(
                scrollable(
                    column
                        .width(Length::Fill)
                        .align_items(Alignment::Center)
                        .padding([0, 0, 40, 0])
                        .spacing(0),
                )
                .height(Length::Fill)
                .vertical_scroll(Properties::new())
                .id(SCROLLABLE_ID.clone())
                .on_scroll(Message::Scrolled),
            );

            let content: Element<Message> = column![scrollable_content,]
                .width(Length::Fill)
                .height(Length::Fill)
                .align_items(Alignment::Center)
                .spacing(10)
                .into();

            Element::from(
                container(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(40)
                    .center_x()
                    .center_y(),
            )
        }

        fn theme(&self) -> Self::Theme {
            Theme::Dark
        }
    }

    struct ScrollbarCustomStyle;

    impl scrollable::StyleSheet for ScrollbarCustomStyle {
        type Style = Theme;

        fn active(&self, style: &Self::Style) -> Scrollbar {
            style.active(&theme::Scrollable::Default)
        }

        fn hovered(&self, style: &Self::Style, is_mouse_over_scrollbar: bool) -> Scrollbar {
            style.hovered(&theme::Scrollable::Default, is_mouse_over_scrollbar)
        }

        fn hovered_horizontal(
            &self,
            style: &Self::Style,
            is_mouse_over_scrollbar: bool,
        ) -> Scrollbar {
            if is_mouse_over_scrollbar {
                Scrollbar {
                    background: style.active(&theme::Scrollable::default()).background,
                    border_radius: 0.0,
                    border_width: 0.0,
                    border_color: Default::default(),
                    scroller: Scroller {
                        color: Color::from_rgb8(250, 85, 134),
                        border_radius: 0.0,
                        border_width: 0.0,
                        border_color: Default::default(),
                    },
                }
            } else {
                self.active(style)
            }
        }
    }
    struct ContainerSelectedStyle;
    impl container::StyleSheet for ContainerSelectedStyle {
        type Style = Theme;
        fn appearance(&self, style: &Self::Style) -> container::Appearance {
            container::Appearance {
                text_color: Some(style.extended_palette().background.strong.text),
                background: Some(style.extended_palette().background.strong.color.into()),
                ..Default::default()
            }
        }
    }
    struct ContainerNotSelectedStyle;
    impl container::StyleSheet for ContainerNotSelectedStyle {
        type Style = Theme;
        fn appearance(&self, style: &Self::Style) -> container::Appearance {
            container::Appearance {
                text_color: Some(style.extended_palette().background.strong.text),
                background: Some(style.extended_palette().background.base.color.into()),
                ..Default::default()
            }
        }
    }
    mod style {
        use iced::widget::container;
        use iced::Theme;
        pub fn selected(theme: &Theme) -> container::Appearance {
            let palette = theme.extended_palette();

            container::Appearance {
                text_color: Some(palette.background.strong.text),
                background: Some(palette.background.strong.color.into()),
                ..Default::default()
            }
        }

        pub fn not_selected(theme: &Theme) -> container::Appearance {
            let palette = theme.extended_palette();

            container::Appearance {
                text_color: Some(palette.primary.strong.text),
                background: Some(palette.primary.strong.color.into()),
                ..Default::default()
            }
        }
    }
}
