use druid::widget::prelude::*;
use druid::widget::{Align, Flex, TextBox};
use druid::{
    commands, platform_menus, AppDelegate, AppLauncher, Command, Data, DelegateCtx,
    FileDialogOptions, FileSpec, Handled, Lens, LocalizedString, Menu, MenuItem, SysMods, Target,
    Widget, WidgetExt, WindowDesc, WindowId,
};
use std::fs;
use std::path::Path;

const TEXT_BOX_WIDTH: f64 = 300.0;
const TEXT_BOX_HEIGHT: f64 = 300.0;
const WINDOW_TITLE: LocalizedString<AppState> = LocalizedString::new("Hello World!");
const MARKDOWN_FILE: FileSpec = FileSpec::new("Markdown file", &["md"]);

fn main() {
    let main_window = WindowDesc::new(build_root_widget())
        .title(WINDOW_TITLE)
        .menu(make_menu)
        .window_size((400.0, 400.0));

    let initial_state = AppState {
        content: "".into(),
        path: None,
    };

    AppLauncher::with_window(main_window)
        .delegate(Delegate)
        .log_to_console()
        .launch(initial_state)
        .expect("Failed to launch application");
}

#[derive(Clone, Data, Lens)]
struct AppState {
    content: String,
    path: Option<String>,
}

impl AppState {
    fn save(&self) {
        self.save_as(self.path.as_ref().unwrap());
    }

    fn save_as<P: AsRef<Path>>(&self, path: P) {
        fs::write(path, &self.content).expect("Unable to write");
    }
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
            data.path = Some(info.path().to_str().unwrap().to_string());
            Handled::Yes
        } else if cmd.is(commands::SAVE_FILE) {
            data.save();
            Handled::Yes
        } else if let Some(info) = cmd.get(commands::SAVE_FILE_AS) {
            data.save_as(info.path());
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
    let menu = if cfg!(target_os = "macos") {
        Menu::empty().entry(platform_menus::mac::application::default())
    } else {
        Menu::empty()
    };
    println!("test");
    menu.entry(file_menu())
}

fn file_menu() -> Menu<AppState> {
    Menu::new(LocalizedString::new("common-menu-file-menu"))
        .entry(platform_menus::mac::file::new_file().enabled(false))
        .entry(
            MenuItem::new(LocalizedString::new("common-menu-file-open")).on_activate(
                |ctx, _, _| {
                    ctx.submit_command(
                        commands::SHOW_OPEN_PANEL
                            .with(FileDialogOptions::new().allowed_types(vec![MARKDOWN_FILE])),
                    )
                },
            ),
        )
        .entry(
            platform_menus::mac::file::save()
                .enabled_if(|data: &AppState, _env: &Env| data.path.is_some()),
        )
        .entry(
            MenuItem::new(LocalizedString::new("common-menu-file-save-as"))
                .on_activate(|ctx, _, _| {
                    ctx.submit_command(
                        commands::SHOW_SAVE_PANEL
                            .with(FileDialogOptions::new().allowed_types(vec![MARKDOWN_FILE])),
                    )
                })
                .hotkey(SysMods::CmdShift, "S"),
        )
}
