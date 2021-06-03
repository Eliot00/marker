use druid::widget::prelude::*;
use druid::widget::{Align, Flex, TextBox};
use druid::{
    commands, AppDelegate, AppLauncher, Command, Data, DelegateCtx, FileDialogOptions, FileSpec,
    Handled, Lens, LocalizedString, Menu, MenuItem, Target, Widget, WidgetExt, WindowDesc,
    WindowId,
};
use std::fs;

const TEXT_BOX_WIDTH: f64 = 300.0;
const TEXT_BOX_HEIGHT: f64 = 300.0;
const WINDOW_TITLE: LocalizedString<AppState> = LocalizedString::new("Hello World!");
const MARKDOWN_FILE: FileSpec = FileSpec::new("Markdown file", &["md"]);

fn main() {
    let main_window = WindowDesc::new(build_root_widget())
        .title(WINDOW_TITLE)
        .menu(make_menu)
        .window_size((400.0, 400.0));

    let initial_state = AppState { content: "".into() };

    AppLauncher::with_window(main_window)
        .delegate(Delegate)
        .log_to_console()
        .launch(initial_state)
        .expect("Failed to launch application");
}

#[derive(Clone, Data, Lens)]
struct AppState {
    content: String,
}

struct Delegate;

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppState,
        _env: &Env,
    ) -> Handled {
        if let Some(info) = cmd.get(commands::OPEN_FILE) {
            data.content = fs::read_to_string(info.path()).unwrap();
            Handled::Yes
        } else {
            Handled::No
        }
    }
}

fn build_root_widget() -> impl Widget<AppState> {
    let textbox = TextBox::multiline()
        .fix_width(TEXT_BOX_WIDTH)
        .fix_height(TEXT_BOX_HEIGHT)
        .lens(AppState::content);

    let layout = Flex::column().with_child(textbox);

    Align::centered(layout)
}

fn make_menu(_window_id: Option<WindowId>, _app_state: &AppState, _env: &Env) -> Menu<AppState> {
    Menu::new(LocalizedString::new("marker")).entry(
        Menu::new(LocalizedString::new("common-menu-file-menu")).entry(
            MenuItem::new(LocalizedString::new("common-menu-file-open")).on_activate(
                |ctx, _, _| {
                    ctx.submit_command(
                        commands::SHOW_OPEN_PANEL
                            .with(FileDialogOptions::new().allowed_types(vec![MARKDOWN_FILE])),
                    )
                },
            ),
        ),
    )
}
