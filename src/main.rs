// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

use druid::text::{AttributesAdder, RichText, RichTextBuilder};
use druid::widget::prelude::*;
use druid::widget::{Container, Controller, LineBreaking, RawLabel, Scroll, Split, TextBox};
use druid::{
    commands, platform_menus, AppDelegate, AppLauncher, Color, Command, Data, DelegateCtx,
    FileDialogOptions, FileSpec, FontFamily, FontStyle, FontWeight, Handled, Lens, LocalizedString,
    Menu, MenuItem, Selector, SysMods, Target, Widget, WidgetExt, WindowDesc, WindowId,
};
use pulldown_cmark::{Event as ParseEvent, HeadingLevel, Options, Parser, Tag};
use std::fs;
use std::path::Path;

const WINDOW_TITLE: LocalizedString<AppState> = LocalizedString::new("Marker");
const MARKDOWN_FILE: FileSpec = FileSpec::new("Markdown file", &["md"]);
const BLOCKQUOTE_COLOR: Color = Color::grey8(0x88);
const LINK_COLOR: Color = Color::rgb8(0, 0, 0xEE);
const OPEN_LINK: Selector<String> = Selector::new("druid-example.open-link");

fn main() {
    let main_window = WindowDesc::new(build_root_widget())
        .title(WINDOW_TITLE)
        .menu(make_menu)
        .window_size((1000.0, 900.0));

    let initial_state = AppState {
        raw_text: "".into(),
        path: None,
        rendered: rebuild_rendered_text(""),
    };

    AppLauncher::with_window(main_window)
        .delegate(Delegate)
        .log_to_console()
        .launch(initial_state)
        .expect("Failed to launch application");
}

#[derive(Clone, Data, Lens)]
struct AppState {
    raw_text: String,
    rendered: RichText,
    path: Option<String>,
}

impl AppState {
    fn save(&self) {
        self.save_as(self.path.as_ref().unwrap());
    }

    fn save_as<P: AsRef<Path>>(&self, path: P) {
        fs::write(path, &self.raw_text).expect("Unable to write");
    }
}

/// A controller that rebuilds the preview when edits occur
struct RichTextRebuilder;

impl<W: Widget<AppState>> Controller<AppState, W> for RichTextRebuilder {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        let pre_data = data.raw_text.to_owned();
        child.event(ctx, event, data, env);
        if !data.raw_text.same(&pre_data) {
            data.rendered = rebuild_rendered_text(&data.raw_text);
        }
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
            data.raw_text = fs::read_to_string(info.path()).unwrap();
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
        .lens(AppState::raw_text)
        .controller(RichTextRebuilder);

    let rendered = Scroll::new(
        RawLabel::new()
            .with_text_color(Color::BLACK)
            .with_line_break_mode(LineBreaking::WordWrap)
            .lens(AppState::rendered)
            .expand_width(),
    )
    .vertical()
    .background(Color::grey8(222))
    .expand();

    Container::new(
        Split::columns(textbox, rendered)
            .split_point(0.5)
            .draggable(true)
            .solid_bar(true)
            .min_size(300.0, 300.0),
    )
}

/// Parse a markdown string and generate a `RichText` object with
/// the appropriate attributes.
fn rebuild_rendered_text(text: &str) -> RichText {
    let mut current_pos = 0;
    let mut builder = RichTextBuilder::new();
    let mut tag_stack = Vec::new();

    let parser = Parser::new_ext(text, Options::ENABLE_STRIKETHROUGH);
    for event in parser {
        match event {
            ParseEvent::Start(tag) => {
                tag_stack.push((current_pos, tag));
            }
            ParseEvent::Text(txt) => {
                builder.push(&txt);
                current_pos += txt.len();
            }
            ParseEvent::End(end_tag) => {
                let (start_off, tag) = tag_stack
                    .pop()
                    .expect("parser does not return unbalanced tags");
                assert_eq!(end_tag, tag, "mismatched tags?");
                add_attribute_for_tag(
                    &tag,
                    builder.add_attributes_for_range(start_off..current_pos),
                );
                if add_newline_after_tag(&tag) {
                    builder.push("\n\n");
                    current_pos += 2;
                }
            }
            ParseEvent::Code(txt) => {
                builder.push(&txt).font_family(FontFamily::MONOSPACE);
                current_pos += txt.len();
            }
            ParseEvent::Html(txt) => {
                builder
                    .push(&txt)
                    .font_family(FontFamily::MONOSPACE)
                    .text_color(BLOCKQUOTE_COLOR);
                current_pos += txt.len();
            }
            ParseEvent::HardBreak => {
                builder.push("\n\n");
                current_pos += 2;
            }
            _ => (),
        }
    }
    builder.build()
}

fn add_newline_after_tag(tag: &Tag) -> bool {
    !matches!(
        tag,
        Tag::Emphasis | Tag::Strong | Tag::Strikethrough | Tag::Link(..)
    )
}

fn add_attribute_for_tag(tag: &Tag, mut attrs: AttributesAdder) {
    match tag {
        Tag::Heading(lvl, ..) => {
            let font_size = match lvl {
                HeadingLevel::H1 => 38.,
                HeadingLevel::H2 => 32.0,
                HeadingLevel::H3 => 26.0,
                HeadingLevel::H4 => 20.0,
                HeadingLevel::H5 => 16.0,
                HeadingLevel::H6 => 12.0,
            };
            attrs.size(font_size).weight(FontWeight::BOLD);
        }
        Tag::BlockQuote => {
            attrs.style(FontStyle::Italic).text_color(BLOCKQUOTE_COLOR);
        }
        Tag::CodeBlock(_) => {
            attrs.font_family(FontFamily::MONOSPACE);
        }
        Tag::Emphasis => {
            attrs.style(FontStyle::Italic);
        }
        Tag::Strong => {
            attrs.weight(FontWeight::BOLD);
        }
        Tag::Strikethrough => {
            attrs.strikethrough(true);
        }
        Tag::Link(_link_ty, target, _title) => {
            attrs
                .underline(true)
                .text_color(LINK_COLOR)
                .link(OPEN_LINK.with(target.to_string()));
        }
        // ignore other tags for now
        _ => (),
    }
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
