use eframe::egui;
use std::time::{Duration, Instant};
use rodio::{OutputStream, source::SineWave, Sink};

struct PomodoroApp {
    start_time: Option<Instant>,
    work_duration: Duration,   // Duration for concentration (work) period
    pause_duration: Duration,  // Duration for break (pause) period
    current_duration: Duration, // The duration for the current interval (work or break)
    timer_running: bool,
    is_work_period: bool,      // Flag to track if it's a work period or break period
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
            work_duration: Duration::new(25 * 60, 0),   // 25 minutes for work
            pause_duration: Duration::new(5 * 60, 0),   // 5 minutes for break
            current_duration: Duration::new(25 * 60, 0), // Initially set to work duration
            timer_running: false,
            is_work_period: true,   // Start with work period
            timer_ended: false,
            sink: Some(sink),
            _stream: Some(_stream), // Keep the stream alive
        }
    }

    fn play_end_sound(&mut self) {
        if let Some(sink) = &self.sink {
            if sink.empty() {
                sink.append(SineWave::new(440.0)); // Append a sound at 440 Hz
                sink.play();
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
                        let remaining = if self.current_duration > elapsed {
                            self.current_duration - elapsed
                        } else {
                            Duration::new(0, 0)
                        };

                        if remaining.as_secs() == 0 {
                            // Timer has ended
                            self.timer_running = false;
                            self.timer_ended = true;

                            // Switch between work and break intervals
                            if self.is_work_period {
                                self.current_duration = self.pause_duration; // Switch to break
                                self.is_work_period = false;
                            } else {
                                self.current_duration = self.work_duration; // Switch to work
                                self.is_work_period = true;
                            }

                            // Restart the timer after switching periods
                            self.start_time = Some(Instant::now());
                        }

                        (remaining.as_secs() / 60, remaining.as_secs() % 60)
                    } else {
                        (0, 0)
                    }
                } else {
                    // If timer is paused or not running, show the remaining time
                    let remaining = self.current_duration;
                    (remaining.as_secs() / 60, remaining.as_secs() % 60)
                };

                ui.heading(format!("{:02}:{:02}", minutes, seconds));
                ui.add_space(20.0);

                // Start/Pause button
                if ui.button(if self.timer_running { "Pause" } else { "Start" }).clicked() {
                    if self.timer_running {
                        // Pausing the timer
                        self.timer_running = false;
                    } else {
                        // Starting the timer
                        self.timer_running = true;
                        self.start_time = Some(Instant::now());
                        self.timer_ended = false;
                    }
                }

                ui.add_space(10.0);

                // Reset button
                if ui.button("Reset").clicked() {
                    self.timer_running = false;
                    self.start_time = None;
                    self.current_duration = self.work_duration;
                    self.timer_ended = false;
                }

                ui.add_space(20.0);

                // Display a progress bar
                let total_elapsed = if self.timer_running {
                    self.start_time.unwrap_or_else(Instant::now).elapsed()
                } else {
                    Duration::new(0, 0)
                };
                let remaining = if self.current_duration > total_elapsed {
                    self.current_duration - total_elapsed
                } else {
                    Duration::new(0, 0)
                };
                let progress = if self.current_duration.as_secs() > 0 {
                    1.0 - remaining.as_secs_f32() / self.current_duration.as_secs_f32()
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

