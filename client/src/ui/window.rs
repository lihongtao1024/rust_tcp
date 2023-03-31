use crate::manager::UiEvent;
use crate::manager::StartBuilder;
use glium::Display;
use glium::glutin::dpi::PhysicalSize;
use imgui::*;
use std::sync::mpsc::Sender;

struct Background;

impl Background {
    fn draw(display: &Display, ui: &Ui){
        let PhysicalSize {
            width,
            height,
        } = display.gl_window().window().inner_size();
        let size = [width as f32, height as f32];

        ui
        .window("background window")
        .no_decoration()
        .no_inputs()
        .always_auto_resize(false)
        .menu_bar(false)
        .movable(false)
        .position([0.0, 0.0], Condition::FirstUseEver)
        .size(size, Condition::FirstUseEver)
        .build(|| {
            ui.set_cursor_pos([0.0, 70.0]);
            ui.separator();

            ui.set_cursor_pos([0.0, 170.0]);
            ui.separator();

            ui.set_cursor_pos([0.0, 300.0]);
            ui.separator();

            ui.set_cursor_pos([0.0, 375.0]);
            ui.separator();
        });
    }
}

struct GridLayout<'a> {
    left: Option<imgui::Window<'a, 'a, &'a str>>,
    right: Option<imgui::Window<'a, 'a, &'a str>>,
}

impl<'a> GridLayout<'a> {
    fn new() -> Self {
        Self {
            left: None,
            right: None,
        }
    }

    fn draw(mut self, display: &Display, ui: &'a Ui) -> Self {
        let PhysicalSize {
            width,
            height
        } = display.gl_window().window().inner_size();

        let width_separator = 130.0;
        let wnd_width = (width as f32 - width_separator) / 2.0;
        let height_separator = 10.0;
        let wnd_height = height as f32 - height_separator;

        self.left = Some(
            ui
            .window("left window")
            .no_decoration()
            .draw_background(false)
            .always_auto_resize(false)
            .menu_bar(false)
            .movable(false)
            .size([wnd_width, wnd_height], Condition::FirstUseEver)
            .position([0.0, 0.0], Condition::FirstUseEver)
        );

        self.right = Some(
            ui
            .window("right window")
            .no_decoration()
            .draw_background(false)
            .always_auto_resize(false)
            .menu_bar(false)
            .movable(false)
            .size([wnd_width, wnd_height], Condition::FirstUseEver)
            .position([width as f32 - wnd_width, 0.0], Condition::FirstUseEver)
        );

        self
    }

    fn into_left(&mut self) -> Option<imgui::Window<'a, 'a, &'a str>> {
        self.left.take().map(|wnd| wnd)
    }

    fn into_right(&mut self) -> Option<imgui::Window<'a, 'a, &'a str>> {
        self.right.take().map(|wnd| wnd)
    }
}

pub struct Window {
    tx: Sender<UiEvent>,
    ip: [i32; 4],
    port: i32,
    packet_size: i32,
    connect_limit: i32,
    connect_success: u64,
    connect_persecond: u64,
    connect_fail: u64,
    connect_avg_delay: u64,
    connect_cur_count: u64,
    send_packet_persecond: u64,
    recv_packet_persecond: u64,
    send_bytes_persecond: u64,
    recv_bytes_persecond: u64,
    send_packets_total: u64,
    recv_packets_total: u64,
    send_bytes_total: u64,
    recv_bytes_total: u64,
    recv_avg_delay: u64,
    recv_max_delay: u64,
    error_packet: u64,
    connect_pressed: bool,
}

impl Window {
    pub fn new(tx: Sender<UiEvent>) -> Self {
        Self {
            tx,
            ip: [127, 0, 0, 1],
            port: 6668,
            packet_size: 2048,
            connect_limit: 3000,
            connect_success: 0,
            connect_persecond: 0,
            connect_fail: 0,
            connect_avg_delay: 0,
            connect_cur_count: 0,
            send_packet_persecond: 0,
            recv_packet_persecond: 0,
            send_bytes_persecond: 0,
            recv_bytes_persecond: 0,
            send_packets_total: 0,
            recv_packets_total: 0,
            send_bytes_total: 0,
            recv_bytes_total: 0,
            recv_avg_delay: 0,
            recv_max_delay: 0,
            error_packet: 0,
            connect_pressed: false,
        }
    }

    fn draw_ip(&mut self, ui: &Ui) {
        let token = ui.begin_disabled(self.connect_pressed);
        ui.text("连 接 地 址:");
        ui.same_line();

        if ui
            .input_int4("##1", &mut self.ip)
            .build() {
                for i in self.ip.iter_mut() {
                    if *i < 0 {
                        *i = 0;
                        return;
                    }

                    if *i > 255 {
                        *i = 255;
                        return;
                    }
                }
        }
        token.end();
    }
    
    fn draw_port(&mut self, ui: &Ui) {
        let token = ui.begin_disabled(self.connect_pressed);
        ui.text("远 程 端 口:");
        ui.same_line();

        if ui
            .input_int("##2", &mut self.port)
            .build() {
                if self.port <= 0 || self.port >= 65536 {
                    self.port = 6666;
                    return;
                }
        }
        token.end();
    }

    fn draw_package_size(&mut self, ui: &Ui) {
        let token = ui.begin_disabled(self.connect_pressed);
        ui.text("网络包大小:");
        ui.same_line();

        if ui
            .input_int("##3", &mut self.packet_size)
            .build() {
                if self.packet_size < 0 {
                    self.packet_size = 0;
                }
        }
        token.end();
    }

    fn draw_connect_limit(&mut self, ui: &Ui) {
        let token = ui.begin_disabled(self.connect_pressed);
        ui.text("连 接 数 量:");
        ui.same_line();

        if ui
            .input_int("##4", &mut self.connect_limit)
            .build() {
                if self.connect_limit < 0 {
                    self.connect_limit = 1;
                }
        }
        token.end();
    }

    fn draw_connect_success(&mut self, ui: &Ui) {
        ui.set_cursor_pos([8.4, 80.0]);
        ui.text("连 接 成 功:");
        ui.same_line();

        ui
        .input_scalar("##5", &mut self.connect_success)
        .read_only(true)
        .build();
    }

    fn draw_connect_persecond(&mut self, ui: &Ui) {
        ui.set_cursor_pos([8.4, 80.0]);
        ui.text("每 秒 连 接:");
        ui.same_line();

        ui
        .input_scalar("##6", &mut self.connect_persecond)
        .read_only(true)
        .build();
    }

    fn draw_connect_fail(&mut self, ui: &Ui) {
        ui.text("连 接 失 败:");
        ui.same_line();

        ui
        .input_scalar("##7", &mut self.connect_fail)
        .read_only(true)
        .build();
    }

    fn draw_connect_avg_delay(&mut self, ui: &Ui) {
        ui.text("连 接 延 时:");
        ui.same_line();

        ui
        .input_scalar("##8", &mut self.connect_avg_delay)
        .read_only(true)
        .build();
    }

    fn draw_connect_cur_count(&mut self, ui: &Ui) {
        ui.text("当 前 连 接:");
        ui.same_line();

        ui
        .input_scalar("##9", &mut self.connect_cur_count)
        .read_only(true)
        .build();
    }

    fn draw_send_packet_persecond(&mut self, ui: &Ui) {
        ui.set_cursor_pos([8.4, 185.0]);
        ui.text("每 秒 发 包:");
        ui.same_line();

        ui
        .input_scalar("##10", &mut self.send_packet_persecond)
        .read_only(true)
        .build();
    }

    fn draw_recv_packet_persecond(&mut self, ui: &Ui) {
        ui.set_cursor_pos([8.4, 185.0]);
        ui.text("每 秒 收 包:");
        ui.same_line();

        ui
        .input_scalar("##11", &mut self.recv_packet_persecond)
        .read_only(true)
        .build();
    }

    fn draw_send_bytes_persecond(&mut self, ui: &Ui) {
        ui.text("每秒发字节:");
        ui.same_line();

        ui
        .input_scalar("##12", &mut self.send_bytes_persecond)
        .read_only(true)
        .build();
    }

    fn draw_recv_bytes_persecond(&mut self, ui: &Ui) {
        ui.text("每秒收字节:");
        ui.same_line();

        ui
        .input_scalar("##13", &mut self.recv_bytes_persecond)
        .read_only(true)
        .build();
    }

    fn draw_send_packets_total(&mut self, ui: &Ui) {
        ui.text("总 计 发 包:");
        ui.same_line();

        ui
        .input_scalar("##14", &mut self.send_packets_total)
        .read_only(true)
        .build();
    }

    fn draw_recv_packets_total(&mut self, ui: &Ui) {
        ui.text("总 计 收 包:");
        ui.same_line();

        ui
        .input_scalar("##15", &mut self.recv_packets_total)
        .read_only(true)
        .build();
    }

    fn draw_send_bytes_total(&mut self, ui: &Ui) {
        ui.text("总计发字节:");
        ui.same_line();

        ui
        .input_scalar("##16", &mut self.send_bytes_total)
        .read_only(true)
        .build();
    }

    fn draw_recv_bytes_total(&mut self, ui: &Ui) {
        ui.text("总计收字节:");
        ui.same_line();

        ui
        .input_scalar("##17", &mut self.recv_bytes_total)
        .read_only(true)
        .build();
    }

    fn draw_recv_avg_delay(&mut self, ui: &Ui) {
        ui.set_cursor_pos([8.4, 315.0]);
        ui.text("接 收 延 时:");
        ui.same_line();

        ui
        .input_scalar("##18", &mut self.recv_avg_delay)
        .read_only(true)
        .build();
    }

    fn draw_recv_max_delay(&mut self, ui: &Ui) {
        ui.set_cursor_pos([8.4, 315.0]);
        ui.text("最大收延时:");
        ui.same_line();

        ui
        .input_scalar("##19", &mut self.recv_max_delay)
        .read_only(true)
        .build();
    }

    fn draw_error_package(&mut self, ui: &Ui) {
        ui.text("总计错误包:");
        ui.same_line();

        ui
        .input_scalar("##20", &mut self.error_packet)
        .read_only(true)
        .build();
    }

    fn draw_connect_button(&mut self, ui: &Ui) {
        ui.set_cursor_pos([215.0, 390.0]);

        let token = ui.begin_disabled(self.connect_pressed);
        if ui.button("开 始") {
            self.connect_pressed = true;

            let addr = format!(
                "{}.{}.{}.{}:{}", 
                self.ip[0], 
                self.ip[1], 
                self.ip[2], 
                self.ip[3], 
                self.port,
            );
            if addr.parse().map(|addr| {
                let builder = StartBuilder {
                    addr,
                    size: self.packet_size as usize,
                    conn: self.connect_limit as usize,
                };
                let _ = self.tx.send(UiEvent::Start(builder));
            }).is_err() {
                
            }
        }
        token.end();
    }

    fn draw_disconnect_button(&mut self, ui: &Ui) {
        ui.set_cursor_pos([0.0, 390.0]);

        let token = ui.begin_disabled(!self.connect_pressed);
        if ui.button("结 束") {
            self.connect_pressed = false;
        }
        token.end();
    }

    pub fn show(&mut self, _: &mut bool, display: &mut Display, ui: &Ui) {
        Background::draw(display, ui);

        let mut grid = GridLayout::new().draw(display, ui);
        grid.into_left().map(|wnd| {
            wnd.build(|| {
                self.draw_ip(ui);
                self.draw_package_size(ui);
                self.draw_connect_success(ui);
                self.draw_connect_fail(ui);
                self.draw_connect_cur_count(ui);
                self.draw_send_packet_persecond(ui);
                self.draw_send_bytes_persecond(ui);
                self.draw_send_packets_total(ui);
                self.draw_send_bytes_total(ui);
                self.draw_recv_avg_delay(ui);
                self.draw_error_package(ui);
                self.draw_connect_button(ui);
            });
        });

        grid.into_right().map(|wnd| {
            wnd.build(|| {
                self.draw_port(ui);
                self.draw_connect_limit(ui);
                self.draw_connect_persecond(ui);
                self.draw_connect_avg_delay(ui);
                self.draw_recv_packet_persecond(ui);
                self.draw_recv_bytes_persecond(ui);
                self.draw_recv_packets_total(ui);
                self.draw_recv_bytes_total(ui);
                self.draw_recv_max_delay(ui);
                self.draw_disconnect_button(ui);
            });
        });
    }
}