use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;

use gtk4::gdk::{Key, ModifierType};
use gtk4::{
    Application, ApplicationWindow, CssProvider, prelude::*, style_context_add_provider_for_display,
};
use gtk4::{Label, glib};
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};

#[derive(Default, Debug)]
struct AppState {
    expr: String,
    result: Option<f64>,
}
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Inputs {
    #[arg(long, short, default_value_t = String::from("Calculator"))]
    title: String,
    #[arg(long, short, default_value_t = String::new())]
    prompt: String,
    #[arg(long, short = 'W', default_value_t = 450)]
    width: i32,
    #[arg(long, short = 'H', default_value_t = 100)]
    height: i32,
    #[arg(long, short)]
    opacity: Option<f32>,
    #[arg(long, short)]
    background_color: Option<String>,
    #[arg(long, short)]
    input_color: Option<String>,
    #[arg(long, short)]
    result_color: Option<String>,
    #[arg(long, short = 'R', default_value_t = 12)]
    border_radius: u32,
    #[arg(long, short, default_value_t = 16)]
    font_size: u32,
    #[arg(long, short, default_value_t = false)]
    center_align: bool,
}

static INPUT: OnceLock<Inputs> = OnceLock::new();

fn provider() -> CssProvider {
    let css = format!(
        "
            window {{
                border-radius: {}px;
                background-color: {};
                opacity: {};
            }}
            
            label {{
                font-size: {}px;
                margin: 10px;
                padding-left: 32px;
            }}
            .expr {{
                color: {};
            }}
            .result {{
                color: {};
            }}
        ",
        INPUT.wait().border_radius,
        INPUT.wait().background_color.clone().unwrap_or_default(),
        INPUT.wait().opacity.unwrap_or(1.0),
        INPUT.wait().font_size,
        INPUT.wait().input_color.clone().unwrap_or_default(),
        INPUT.wait().result_color.clone().unwrap_or_default(),
    );

    let provider = CssProvider::new();
    provider.load_from_data(css.as_str());
    provider

    // Apply CSS globally to all windows
}

fn main() -> glib::ExitCode {
    let _ = INPUT.set(Inputs::parse());

    let app = Application::builder()
        .application_id("dev.merle.calcu")
        .build();

    app.connect_activate(|app| {
        let app_state = Rc::new(RefCell::new(AppState::default()));
        let expr_label = Rc::new(RefCell::new(Label::new(Some(&INPUT.wait().prompt))));
        let result_label = Rc::new(RefCell::new(Label::new(Some(&""))));

        style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().unwrap(),
            &provider(),
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(INPUT.wait().width)
            .default_height(INPUT.wait().height)
            .title(INPUT.wait().title.clone())
            .build();

        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(KeyboardMode::Exclusive);
        window.set_exclusive_zone(-1);
        // window.set_decorated(false);
        if INPUT.wait().background_color.is_some() {
            window.remove_css_class("background");
        }

        window.set_resizable(false);
        window.style_context().add_class("window");
        window.auto_exclusive_zone_enable();

        let result_label_clone = result_label.clone();
        let expr_labe_clone = expr_label.clone();

        let key_controller = gtk4::EventControllerKey::new();
        key_controller.connect_key_pressed(move |_, key, _, modif| {
            let result = key_event(key, modif, app_state.clone());
            let expr_label = expr_labe_clone.borrow_mut();
            expr_label.set_text(&format!(
                "{}{}",
                INPUT.wait().prompt,
                app_state.borrow().expr
            ));
            app_state
                .borrow()
                .result
                .map(|num| {
                    result_label_clone
                        .borrow_mut()
                        .set_text(&format!("= {num}"))
                })
                .or_else(|| {
                    result_label_clone.borrow_mut().set_text(&"");
                    None
                });
            result
        });
        window.add_controller(key_controller);

        let vbox = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .build();

        expr_label.borrow().add_css_class(&"expr");
        result_label.borrow().add_css_class(&"result");
        if !INPUT.wait().center_align {
            expr_label.borrow().set_halign(gtk4::Align::Start);
            result_label.borrow().set_halign(gtk4::Align::Start);
        }
        vbox.append(&*expr_label.borrow());
        vbox.append(&*result_label.borrow());

        window.set_child(Some(&vbox));

        window.present();
    });

    app.run_with_args(&[] as &[&str])
}

fn key_event(key: Key, _: ModifierType, state: Rc<RefCell<AppState>>) -> glib::Propagation {
    let mut state = state.borrow_mut();
    match key {
        Key::Escape => std::process::exit(0),
        k if k
            .to_unicode()
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || "+-/*^%!.()".contains(ch)) =>
        {
            let ch = k.to_unicode().unwrap();

            state.expr.push(ch);
            state.result = expr_solver::eval(&state.expr).ok();
        }
        Key::BackSpace | Key::Delete => {
            state.expr.pop();
            state.result = expr_solver::eval(&state.expr).ok();
        }
        _ => (),
    }
    if state.expr.is_empty() {
        state.result = None;
    }
    glib::Propagation::Proceed
}
