use eframe::egui;
use std::time::{Duration, Instant};
use rodio::{OutputStream, source::SineWave, Sink};

struct PomodoroApp {
    start_time: Option<Instant>,
    paused_duration: Duration, // Track the total duration for which the timer was paused
    duration: Duration,
    timer_running: bool,
    timer_ended: bool,
    sink: Option<Sink>,
    _stream: Option<OutputStream>, // Keep the stream alive
}

impl PomodoroApp {
    fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        Self {
            start_time: None,
            paused_duration: Duration::new(0, 0),
            duration: Duration::new(1500, 0), // 25 minutes for Pomodoro
            timer_running: false,
            timer_ended: false,
            sink: Some(sink),
            _stream: Some(_stream), // Keep the stream alive
        }
    }

    fn play_end_sound(&mut self) {
        if let Some(sink) = &self.sink {
            if sink.empty() {  // Check if sink is empty
                sink.append(SineWave::new(440.0)); // Append sound
                sink.play(); // Ensure the sink is playing
                println!("Playing sound..."); // Debug print
            }
        } else {
            println!("Sink is None, cannot play sound."); // Debug print
        }
    }
}

impl eframe::App for PomodoroApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let mut style: egui::Style = (*ctx.style()).clone();
        style.text_styles.get_mut(&egui::TextStyle::Body).unwrap().size = 60.0;
        style.text_styles.get_mut(&egui::TextStyle::Heading).unwrap().size = 80.0;
        style.text_styles.get_mut(&egui::TextStyle::Button).unwrap().size = 40.0;

        style.visuals = egui::Visuals {
            dark_mode: true,
            widgets: egui::style::Widgets {
                inactive: egui::style::WidgetVisuals {
                    bg_fill: egui::Color32::from_gray(40),
                    bg_stroke: egui::Stroke::new(1.0, egui::Color32::WHITE),
                    fg_stroke: egui::Stroke::new(1.5, egui::Color32::WHITE),
                    rounding: egui::Rounding::same(5.0),
                    weak_bg_fill: egui::Color32::TRANSPARENT,
                    expansion: 0.0,
                },
                active: egui::style::WidgetVisuals {
                    bg_fill: egui::Color32::from_gray(70),
                    bg_stroke: egui::Stroke::new(2.0, egui::Color32::WHITE),
                    fg_stroke: egui::Stroke::new(2.0, egui::Color32::WHITE),
                    rounding: egui::Rounding::same(5.0),
                    weak_bg_fill: egui::Color32::TRANSPARENT,
                    expansion: 0.0,
                },
                hovered: egui::style::WidgetVisuals {
                    bg_fill: egui::Color32::from_gray(50),
                    bg_stroke: egui::Stroke::new(1.5, egui::Color32::WHITE),
                    fg_stroke: egui::Stroke::new(1.5, egui::Color32::WHITE),
                    rounding: egui::Rounding::same(5.0),
                    weak_bg_fill: egui::Color32::TRANSPARENT,
                    expansion: 0.0,
                },
                noninteractive: egui::style::WidgetVisuals {
                    bg_fill: egui::Color32::from_gray(30),
                    bg_stroke: egui::Stroke::new(1.0, egui::Color32::WHITE),
                    fg_stroke: egui::Stroke::new(1.0, egui::Color32::WHITE),
                    rounding: egui::Rounding::same(5.0),
                    weak_bg_fill: egui::Color32::TRANSPARENT,
                    expansion: 0.0,
                },
                open: egui::style::WidgetVisuals {
                    bg_fill: egui::Color32::from_gray(60),
                    bg_stroke: egui::Stroke::new(1.0, egui::Color32::WHITE),
                    fg_stroke: egui::Stroke::new(1.0, egui::Color32::WHITE),
                    rounding: egui::Rounding::same(5.0),
                    weak_bg_fill: egui::Color32::TRANSPARENT,
                    expansion: 0.0,
                },
            },
            ..egui::Visuals::default()
        };
        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                // Timer display
                let (minutes, seconds) = if self.timer_running {
                    if let Some(start_time) = self.start_time {
                        let elapsed = start_time.elapsed();
                        let total_elapsed = elapsed + self.paused_duration;
                        let remaining = if self.duration > total_elapsed {
                            self.duration - total_elapsed
                        } else {
                            Duration::new(0, 0)
                        };
                        (remaining.as_secs() / 60, remaining.as_secs() % 60)
                    } else {
                        (0, 0)
                    }
                } else {
                    let total_elapsed = self.paused_duration;
                    let remaining = if self.duration > total_elapsed {
                        self.duration - total_elapsed
                    } else {
                        Duration::new(0, 0)
                    };
                    (remaining.as_secs() / 60, remaining.as_secs() % 60)
                };

                ui.heading(format!("{:02}:{:02}", minutes, seconds));
                ui.add_space(20.0);

                // Center the buttons using vertical_centered and horizontal layouts
                ui.vertical_centered(|ui| {
                    if ui.button(if self.timer_running { "Pause" } else { "Start" }).clicked() {
                        if self.timer_running {
                            self.timer_running = false;
                            self.paused_duration += self.start_time.unwrap_or_else(Instant::now).elapsed();
                            self.start_time = None;
                        } else {
                            self.timer_running = true;
                            self.start_time = Some(Instant::now());
                            self.timer_ended = false;
                        }
                    }

                    ui.add_space(10.0);

                    if ui.button("Reset").clicked() {
                        self.timer_running = false;
                        self.start_time = None;
                        self.paused_duration = Duration::new(0, 0);
                        self.timer_ended = false;
                    }
                });

                ui.add_space(20.0);

                // Display a progress bar
                let total_elapsed = if self.timer_running {
                    self.start_time.unwrap_or_else(Instant::now).elapsed() + self.paused_duration
                } else {
                    self.paused_duration
                };
                let remaining = if self.duration > total_elapsed {
                    self.duration - total_elapsed
                } else {
                    Duration::new(0, 0)
                };
                let progress = if self.duration.as_secs() > 0 {
                    1.0 - remaining.as_secs_f32() / self.duration.as_secs_f32()
                } else {
                    0.0
                };

                ui.add(egui::ProgressBar::new(progress).desired_width(300.0));

                if self.timer_ended {
                    ui.colored_label(egui::Color32::RED, "Timer Ended");
                    self.play_end_sound();
                }

                ui.add_space(20.0);
            });
        });

        ctx.request_repaint();
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 350.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Pomodoro Timer",
        options,
        Box::new(|_cc| Ok(Box::new(PomodoroApp::new()))),
    )
}
