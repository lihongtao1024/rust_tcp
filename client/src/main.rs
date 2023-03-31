#![windows_subsystem = "windows"]
use client::Application;
use client::manager;
use client::Window;
use std::sync::mpsc;

fn main() {
    let app = Application::new(17.0);

    let (tx, rx) = mpsc::channel();
    let mut wnd = Window::new(tx);    
    manager::start(rx);

    app.show_window(
        "网络性能测试客户端",
        640.0,
        430.0,
        false,
        move |run, display, ui| {
            wnd.show(run, display, ui);
        }
    );
}