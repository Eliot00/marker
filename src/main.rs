use druid::widget::{Align, Flex, TextBox};
use druid::{AppLauncher, Data, Lens, LocalizedString, Widget, WidgetExt, WindowDesc};

const TEXT_BOX_WIDTH: f64 = 300.0;
const TEXT_BOX_HEIGHT: f64 = 300.0;
const WINDOW_TITLE: LocalizedString<EditorState> = LocalizedString::new("Hello World!");

#[derive(Clone, Data, Lens)]
struct EditorState {
    content: String,
}

fn main() {
    let main_window = WindowDesc::new(build_root_widget())
        .title(WINDOW_TITLE)
        .window_size((400.0, 400.0));

    let initial_state = EditorState { content: "".into() };

    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn build_root_widget() -> impl Widget<EditorState> {
    let textbox = TextBox::multiline()
        .fix_width(TEXT_BOX_WIDTH)
        .fix_height(TEXT_BOX_HEIGHT)
        .lens(EditorState::content);

    let layout = Flex::column().with_child(textbox);

    Align::centered(layout)
}
