use std::fs;
use std::env;
use std::fs::File;
use std::process::{Command, Stdio};
use std::io::{Write};
use std::path::{Path, PathBuf};
use fltk::{app::*, button::*, frame::*, window::*, prelude::*, group::Scroll, enums::Color, image::SharedImage};
use fltk_theme::{WidgetTheme, ThemeType};

static CURSORS_DIR: &'static str = "./cursors";

#[derive(Clone)]
enum Message {
    SetCategory(PathBuf)
}

fn main() {
    // let scheme = WidgetScheme::new(SchemeType::Gleam);
    // scheme.apply();
    let theme = WidgetTheme::new(ThemeType::Aero);
    theme.apply();
    
    platform_check();
    let app = App::default();
    set_background_color(176, 205, 221);
    let (_, r) = channel::<Message>();
    
    let mut wind = Window::new(100, 100, 400, 480, "CMRedux");
    let mut title = Frame::new(10, 6, 140, 50, "CMRedux");
    title.set_label_size(32);
    let mut scroll = Scroll::new(10, 60, 120, 360, None);
    let mut cursor_scroll = Scroll::new(140, 60, 250, 360, None);
    scroll.set_color(Color::White);
    cursor_scroll.set_color(Color::White);
    wind.add(&cursor_scroll);
    
    init_categories(&mut scroll);

    wind.end();
    wind.show();

    while app.wait() {
        if let Some(msg) = r.recv() {
            match msg {
                Message::SetCategory(cat) => show_cursors(&mut cursor_scroll, cat),
            }
        }
    }
}

fn init_categories(scroll: &mut Scroll) {
    fn scrub_dir(dir: PathBuf, scroll: &mut Scroll, i: usize) -> usize {
        let paths = fs::read_dir(dir).unwrap();
        let nested = i != 0;
        let mut i = i;
        for path in paths {
            let path = path.unwrap();
            let metadata = path.metadata().unwrap();
            if metadata.is_dir() {
                let (s, _) = channel::<Message>();
                
                let butlabel = path.file_name().into_string().unwrap();
                
                let words: Vec<&str> = butlabel.split(" ").collect();
                let mut out: Vec<String> = Vec::new();
                for word in &words {
                    let lower = word.to_lowercase();
                    let mut c = lower.chars();
                    if let Some(f) = c.next() {
                        let titled = f.to_uppercase().chain(c).collect::<String>();
                        out.push(titled)
                    } else {
                        out.push(String::new())
                    }
                }
                let result = out.join(" ").split(" - ").collect::<Vec<&str>>().join("/");
                
                let x = if nested { 22 } else { 12 };
                let width = if nested { 90 } else { 100 };
                let mut but = Button::new(x, (i as i32 * 25) + 14, width, 20, string_to_str(result));
                but.emit(s, Message::SetCategory(path.path()));
                scroll.add(&but);
                
                i = scrub_dir(path.path(), scroll, i + 1);
            }
        }
        return i
    }
    scrub_dir(PathBuf::from(CURSORS_DIR), scroll, 0);
    scroll.scroll_to(0, -48);
}

fn show_cursors(scroll: &mut Scroll, category: PathBuf) {
    scroll.clear();
    fn scrub_dir(dir: PathBuf, scroll: &mut Scroll, i: usize) -> usize {
        let mut i = i;
        for path in fs::read_dir(dir).unwrap() {
            let path = path.unwrap();
            let metadata = path.metadata().unwrap();
            if metadata.is_file() {
                let mut but = Button::new(((i as i32 % 5) * 46) + 150, ((i as i32 / 5) * 46) + 20, 32, 32, None); //string_to_str(path.file_name().into_string().unwrap())
                let image = SharedImage::load(path.path()).expect("Failed to load");
                but.set_image(Some(image));
                but.set_label_size(11);
                but.set_callback(move |_| set_cursor(path.path()));
                scroll.add(&but);
                scroll.redraw();
                i += 1
            } else if metadata.is_dir() {
                // i = scrub_dir(path.path(), scroll, i)
            }
        }
        return i
    }
    scrub_dir(category, scroll, 0);
    // scroll.scroll_to(0, -48);
}

fn string_to_str(s: String) -> &'static str {
  Box::leak(s.into_boxed_str())
}

fn throw_popup(width: i32, height: i32, text: String) {
    let app = App::default();
    
    let mut wind = Window::new(100, 100, width, height, "Warning!");
    Frame::new(175, 45, 0, 0, string_to_str(text));
    let mut but = Button::new(135, 70, 80, 30, "Close");
    but.set_callback(move |_| {app.quit();});
    
    wind.end();
    wind.show();
    app.run().unwrap();
}

#[cfg(target_os = "linux")]
fn platform_check() {
    if None == find_command("xcursorgen") {
        throw_popup(350, 125, "xcursorgen could not be found!\nCursors will not be applicable!".to_string())
    }
    if None == find_command("convert") {
        throw_popup(350, 125, "ImageMagick's \"convert\" could not be found!\nCursors will not be applicable!".to_string())
    }
}

fn update_cursor_theme() {
    let mut de: Option<&str> = None;
    if let Some(var_de) = env::var_os("DESKTOP_SESSION") {
        de = Some(string_to_str(var_de.into_string().unwrap()));
    }
    
    match de {
        Some("xfce") => {
            let setcur = |name: String| {
                Command::new("xfconf-query")
                    .args([
                        "--channel", "xsettings", 
                        "--property", "/Gtk/CursorThemeName",
                        "--set", &name
                    ])
                    .spawn().unwrap().wait().unwrap();
            };
            setcur("default".to_string());
            setcur("cmcursor".to_string());
        },
        Some(_) | None => throw_popup(350, 125, "Unknown desktop environment!\nChange your cursor theme to 'cmcursor' manually.".to_string())
    }
}

#[cfg(target_os = "linux")]
fn set_cursor(path_buf: PathBuf) {
    if let Some(homedir) = home::home_dir() {
        let working_dir = homedir.clone().join(".local/share/icons/cmcursor");
        fs::create_dir_all(&working_dir.join("cursors")).expect("failed to create dir");
        let path = &path_buf.clone().into_os_string().into_string().unwrap();
        
        let input = File::open(&path).unwrap();
        let mut options = gif::DecodeOptions::new();
        options.set_color_output(gif::ColorOutput::RGBA);
        let mut decoder = options.read_info(input).unwrap();
        let mut cfg = String::new();
        
        Command::new("convert")
            .arg(&path_buf.clone().into_os_string().into_string().unwrap())
            .arg(&working_dir.join("cursor.png"))
            .spawn().unwrap().wait();
        
        let mut i = 0;
        let mut cfg_push = "32 0 0 cursor.png\n".to_string();
        while let Some(frame) = decoder.read_next_frame().unwrap() {
            cfg_push = format!(
                "{} 0 0 cursor-{}.png {}\n", 
                frame.width, 
                i, 
                frame.delay * 10
            );
            cfg.push_str(cfg_push.as_str());
            i += 1;
        }
        if i == 1 {
            let mut chunks: Vec<&str> = cfg_push.split_whitespace().collect();
            chunks[3] = "cursor.png";
            chunks.pop();
            cfg = chunks.join(" ");
        }
        
        let mut cursorgen = Command::new("xcursorgen")
            .arg("-p")
            .arg(&working_dir)
            .arg("-")
            .arg(&working_dir.join("cursors/pointer"))
            .stdin(Stdio::piped())
            .spawn().unwrap();
            
        let mut cursorgen_stdin = cursorgen.stdin.take().unwrap();
        cursorgen_stdin.write_all(cfg.as_bytes()).unwrap();
        drop(cursorgen_stdin);
        
        cursorgen.wait();
        
        if !Path::exists(&working_dir.join("cursors/wait")) {
            let links = ["00008160000006810000408080010102", "028006030e0e7ebffc7f7070c0600140", "03b6e0fcb3499374a867c041f52298f0", "08e8e1c95fe2fc01f976f1e063a24ccd", "1081e37283d90000800003c07f3ef6bf", "14fef782d02440884392942c11205230", "2870a09082c103050810ffdffffe0204", "3085a0e285430894940527032f8b26df", "3ecb610c1bf2410f44200f48c40d3599", "4498f0e0c1937ffe01fd06f973665830", "5c6cd98b3f3ebcb1f9c7f1c204630408", "6407b0e94181790501fd1e167b474872", "640fb0e74195791501fd1ed57b41487f", "9081237383d90e509aa00f00170e968f", "9d800788f1b08800ae810202380a0822", "alias", "all-scroll", "arrow", "bd_double_arrow", "bottom_side", "bottom_tee", "cell", "circle", "context-menu", "copy", "cross", "crossed_circle", "crosshair", "cross_reverse", "d9ce0ab605698f320427677b458ad60b", "default", "diamond_cross", "dnd-ask", "dnd-copy", "dnd-link", "dnd-move", "dnd-no-drop", "dnd-none", "dotbox", "dot_box_mask", "double_arrow", "draft_large", "draft_small", "draped_box", "e29285e634086352946a0e7090d73106", "fd_double_arrow", "fleur", "grab", "grabbing", "hand", "hand1", "hand2", "h_double_arrow", "help", "icon", "left_ptr", "left_ptr_help", "left_ptr_watch", "left_side", "left_tee", "link", "ll_angle", "lr_angle", "move", "no-drop", "not-allowed", "pencil", "pirate", "plus", "pointer-move", "progress", "question_arrow", "right_ptr", "right_side", "right_tee", "sb_down_arrow", "sb_h_double_arrow", "sb_left_arrow", "sb_right_arrow", "sb_up_arrow", "sb_v_double_arrow", "target", "tcross", "text", "top_left_arrow", "top_side", "top_tee", "ul_angle", "ur_angle", "v_double_arrow", "vertical-text", "wait", "watch", "X_cursor", "xterm", "zoom-in", "zoom-out"];
            for val in links.iter() {
                std::os::unix::fs::symlink(
                    &working_dir.join("cursors/pointer"), 
                    &working_dir.join("cursors/").join(val)
                );
            }
        }
        if !Path::exists(&working_dir.join("index.theme")) {
            let mut file = File::create(&working_dir.join("index.theme"))
                .expect("Could not create index.theme");
            
            file.write_all(b"[Icon Theme]\nName=CMRedux Cursor\nExample=left_ptr\nInherits=core")
                .expect("Error while writing to index.theme");
        }
        update_cursor_theme();
        for path in fs::read_dir(working_dir).unwrap() {
            let path = path.unwrap().path();
            if let Some(extension) = path.extension() && extension == std::ffi::OsStr::new("png") {
                fs::remove_file(path).unwrap();
            }
        }
    }
}

fn find_command<P>(exe_name: P) -> Option<PathBuf>
    where P: AsRef<Path>,
{
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths).filter_map(|dir| {
            let full_path = dir.join(&exe_name);
            if full_path.is_file() {
                Some(full_path)
            } else {
                None
            }
        }).next()
    })
}
