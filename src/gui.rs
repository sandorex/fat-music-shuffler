use std::{cell::RefCell, rc::Rc};
use anyhow::Result;
use ply_engine::prelude::*;
use crate::util::DiskOrPartition;

#[derive(Debug)]
struct Theme {
    pub bg_pressed: Color,
    pub bg_hovered: Color,

    pub fg1: Color,
    pub fg2: Color,

    pub fg_disabled: Color,

    pub bg1: Color,

    pub font_size: u16,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bg_pressed: Color::from(0x524640),
            bg_hovered: Color::from(0x3A3434),

            fg1: Color::from(0xE8E0DC),
            fg2: Color::from(0xCBBEB7),

            fg_disabled: Color::from(0x79716C),

            bg1: Color::from(0x1E1B1B),

            font_size: 18,
        }
    }
}

thread_local! {
    static TARGET_SELECTED: RefCell<Option<DiskOrPartition>> = RefCell::new(None);
    static TARGETS: RefCell<Vec<DiskOrPartition>> = RefCell::new(vec![]);
    static SELECTOR_SHOWN: RefCell<bool> = RefCell::new(false);
}

fn window_conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: miniquad::conf::Conf {
            window_title: concat!("Fat Music Shuffler (v", env!("CARGO_PKG_VERSION"), ")").to_owned(),
            window_width: 600,
            window_height: 300,
            high_dpi: true,
            sample_count: 4,
            platform: miniquad::conf::Platform {
                webgl_version: miniquad::conf::WebGLVersion::WebGL2,
                ..Default::default()
            },
            window_resizable: false,
            ..Default::default()
        },
        draw_call_vertex_capacity: 100000,
        draw_call_index_capacity: 100000,
        ..Default::default()
    }
}

fn update_targets() -> Result<()> {
    // update devices
    TARGETS.with_borrow_mut(|targets| -> Result<()> {
        *targets = crate::lsblk::query_all_block_devices()?;
        Ok(())
    })
}

pub fn main() -> Result<()> {
    use clap::Parser;

    // if there are arguments parse them
    if std::env::args().count() > 1 {
        let args = crate::cli::Cli::parse();
        crate::handle_cli(args)?;

        return Ok(());
    }

    update_targets()?;

    let result: Rc<RefCell<Result<()>>> = Rc::new(RefCell::new(Ok(())));

    // TODO this looks awful but idk how to do it properly
    // TODO this is not recommended? but how do i avoid the macro?
    // basically return the result using a pointer
    {
        let result = Rc::clone(&result);

        macroquad::Window::from_config(
            window_conf(),
            async move {
                match gui_main().await {
                    Ok(_) => {},
                    Err(err) => {
                        *result.borrow_mut() = Err(err);
                    },
                }
            }
            // gui_main()
        );
    }

    Rc::try_unwrap(result)
        .expect("could not unwrap result rc")
        .into_inner()
}

fn dropdown_item(ui: &mut Ui, item: &DiskOrPartition, item_index: usize) {
    ui.element()
        .on_press(move |_, _| {
            // hide the dropdown
            SELECTOR_SHOWN.with_borrow_mut(|x| *x = false);

            TARGETS.with_borrow(|targets| {
                TARGET_SELECTED.with_borrow_mut(|selected| {
                    *selected = Some(targets.get(item_index).cloned().unwrap());
                });
            });
        })
        .width(grow!())
        .children(|ui| {
            let bg = if ui.pressed() {
                0xB91414
            } else if ui.hovered() || ui.focused() {
                0xFF654D
            } else {
                0xE8E0DC
            };

            match item {
                DiskOrPartition::Disk(disk) => {
                    // left aligned stuff
                    ui.element()
                        .width(grow!())
                        .layout(|l| l.gap(10_u16).padding((0, 10, 0, 0)))
                        .children(|ui| {
                            if let Some(model) = &disk.model {
                                ui.text(model, |t| t.font_size(18).color(bg));
                            }
                        });

                    // right aligned stuff
                    ui.element()
                        .width(fit!())
                        .layout(|l| l.gap(10_u16))
                        .children(|ui| {
                            ui.text(&disk.size, |t| t.font_size(18).color(bg));
                            ui.text(&disk.path, |t| t.font_size(18).color(bg));
                        });
                }
                DiskOrPartition::Partition(part) => {
                    // left aligned stuff
                    ui.element()
                        .width(grow!())
                        .layout(|l| l.gap(10_u16))
                        .children(|ui| {
                            if let Some(label) = &part.label {
                                ui.text(label, |t| t.font_size(18).color(bg));
                            }
                        });

                    // right aligned
                    ui.element()
                        .width(fit!())
                        .layout(|l| l.gap(10_u16))
                        .children(|ui| {
                            ui.text(&part.size, |t| t.font_size(18).color(bg));
                            ui.text(&part.path, |t| t.font_size(18).color(bg));
                        });
                }
            }
        });
}

// TODO press
fn button<F>(ui: &mut Ui, theme: &Theme, text: &str, enabled: bool, on_press: F)
where
    F: FnMut(Id, ply_engine::engine::PointerData) + 'static,
{
    let mut e = ui.element()
        .width(fit!())
        .height(fit!());

    // enable on_press only when enabled
    if enabled {
        e = e.on_press(on_press);
    }

    e.children(|ui| {
        let fg = if !enabled {
            Color::from(theme.fg_disabled)
        } else {
            theme.fg1
        };

        let bg = if enabled {
            if ui.pressed() {
                theme.bg_pressed
            } else if ui.hovered() || ui.focused() {
                theme.bg_hovered
            } else {
                theme.bg1
            }
        } else {
            theme.bg1
        };

        ui.element()
            .background_color(bg)
            .layout(|l| l.padding((6, 18, 6, 18)).gap(10).align(CenterX, CenterY))
            .corner_radius(6.0)
            .border(|b| b.all(2).color(fg))
            .children(|ui| {
                ui.text(text, |f| f.font_size(theme.font_size).color(fg));
            });
    });
}

async fn gui_main() -> Result<()> {
    static DEFAULT_FONT: FontAsset = FontAsset::Bytes {
        file_name: "lexend.ttf",
        data: include_bytes!("../assets/fonts/lexend.ttf")
    };

    let mut ply = Ply::<()>::new(&DEFAULT_FONT).await;

    // let mut result: Result<()> = Ok(());
    let theme = Theme::default();

    loop {
        clear_background(theme.bg1.into());

        let mut ui = ply.begin();
        // ui.set_debug_mode(true);

        ui.element()
            .width(grow!())
            .height(grow!())
            .layout(|l| l.direction(TopToBottom).gap(20).padding(20_u16))
            .children(|ui| {
                ui.element()
                    .width(grow!())
                    .height(fit!())
                    .children(|ui| {
                        let bg = if ui.pressed() {
                            theme.bg_pressed
                        } else if ui.hovered() || ui.focused() {
                            theme.bg_hovered
                        } else {
                            theme.bg1
                        };

                        ui.element()
                            .width(grow!())
                            .height(fit!())
                            .border(|b| b.all(2).color(theme.fg1))
                            .layout(|l| l.padding(12))
                            .background_color(bg)
                            .corner_radius(8.0)
                            .children(|ui| {
                                ui.text("Target:", |f| f.font_size(theme.font_size).color(theme.fg2));

                                TARGET_SELECTED.with_borrow(|target| {
                                    ui.element()
                                        .width(grow!())
                                        .height(fit!())
                                        .layout(|l| l.align(CenterX, CenterY).gap(5))
                                        .children(|ui| {
                                            if let Some(target) = target {
                                                ui.text(if target.is_partition() { "Partition" } else { "Disk" }, |f| f.font_size(theme.font_size).color(theme.fg2));

                                                ui.text(
                                                    &format!("{} {:?} ({})", target.size(), target.model_or_label().unwrap_or_default(), target.path()),
                                                    |f| f.font_size(theme.font_size).color(theme.fg1)
                                                )
                                            } else {
                                                ui.text("Click to select target", |f| f.font_size(theme.font_size).color(theme.fg2))
                                            }
                                        });
                                })
                            });
                    });

                // buttons
                ui.element()
                    .width(grow!())
                    .height(fit!())
                    .layout(|l| l.gap(14))
                    .children(|ui| {
                        button(ui, &theme, "Format", true, |_, _| {
                            jobs::spawn("format", async || -> Result<()> {
                                Ok(())
                            }, |x| {
                                match x {
                                    Ok(_) => println!("format job completed successfully"),
                                    Err(err) => println!("format job error: {err}"),
                                }
                            }).expect("could not start format job");
                            // crate::commands::format(target, interactive)
                        });
                        ui.element().width(grow!()).empty();

                        button(ui, &theme, "Clean", true, |_, _| {});
                        ui.element().width(grow!()).empty();

                        // TODO import is for later
                        button(ui, &theme, "Import", false, |_, _| {});
                        ui.element().width(grow!()).empty();

                        button(ui, &theme, "Shuffle", true, |_, _| {});
                        ui.element().width(grow!()).empty();

                        // TODO help
                        button(ui, &theme, "?", false, |_, _| {});
                    });
            });

        // // TODO replace selector with modal fullscreen dialog, would also make it possible to make
        // // nicer choosing with multiline information about disk/partition
        // ui.element()
        //     .width(grow!())
        //     .height(fit!())
        //     .background_color(0x1E1B1B)
        //     .layout(|l| l.align(Left, Top).padding(24_u16))
        //     .children(|ui| {
        //         // TODO do "Device:" <selected device> V "Refresh Button" where V is the icon for
        //         // dropdown menu
        //         ui.element()
        //             .width(grow!())
        //             .background_color(0x660000)
        //             .corner_radius(6.0)
        //             .layout(|l| l.padding(5).gap(16).align(Left, CenterY))
        //             .children(|ui| {
        //                 ui.text("Target:", |t| t.font_size(18).color(0xE8E0DC));
        //
        //                 ui.element()
        //                     .id("selector")
        //                     .width(grow!())
        //                     .on_press(|_, _| {
        //                         // TODO remove unwrap
        //                         update_targets().unwrap();
        //
        //                         // toggle it
        //                         SELECTOR_SHOWN.with_borrow_mut(|f| *f = !*f);
        //                     })
        //                     .children(|ui| {
        //                         TARGET_SELECTED.with_borrow(|selected| {
        //                             if let Some(selected) = selected.as_ref() {
        //                                 ui.text(&format!("{}", selected), |t| t.font_size(18).color(0xE8E0DC).wrap_mode(WrapMode::Words));
        //                             } else {
        //                                 ui.text("Select disk/partition", |t| t.font_size(18).color(0xE8E0DC).wrap_mode(WrapMode::Words));
        //                             }
        //                         });
        //                     });
        //
        //                 // // TODO refresh button icon
        //                 // ui.element()
        //                 //     .on_press(|_, _| {
        //                 //         // hide the selector just in case
        //                 //         SELECTOR_SHOWN.with_borrow_mut(|f| *f = false);
        //                 //
        //                 //         // TODO cannot use anyhow here!
        //                 //         update_targets().unwrap();
        //                 //     })
        //                 //     .aspect_ratio(1.0)
        //                 //     .width(fixed!(24.0))
        //                 //     .background_color(0x006600)
        //                 //     .corner_radius(4.0)
        //                 //     .layout(|l| l.align(CenterX, CenterY))
        //                 //     .children(|ui| {
        //                 //         ui.text("R", |t| t.font_size(16).color(0x000000));
        //                 //     });
        //             });
        //     });
        //
        // SELECTOR_SHOWN.with_borrow(|f| if *f {
        //     ui.element()
        //         .floating(|f| f.attach_id("selector").anchor((CenterX, Top), (CenterX, Bottom)).offset((0.0, 15.0)))
        //         // .background_color(0x000066)
        //         .border(|b| b.color(0xFF0000).all(2))
        //         .layout(|l| l.direction(TopToBottom).padding(10).gap(10_u16))
        //         .width(fit!())
        //         .children(|ui| {
        //             TARGETS.with(|targets| {
        //                 for (index, item) in targets.borrow().iter().enumerate() {
        //                     dropdown_item(ui, item, index);
        //                 }
        //             });
        //         });
        //
        //     // if ui.p
        // });

        // TARGET_SELECTED.with_borrow(|selected| match selected {
        //     Some(DiskOrPartition::Disk(_)) => {
        //         ui.element()
        //             .children(|ui| {
        //                 ui.text("Format disk", |f| f.font_size(18));
        //             });
        //     }
        //
        //     Some(DiskOrPartition::Partition(_)) => {
        //         ui.element()
        //             .children(|ui| {
        //                 ui.text("Format", |f| f.font_size(18).color(0xE8E0DC));
        //             });
        //
        //         ui.element()
        //             .children(|ui| {
        //                 ui.text("Clean", |f| f.font_size(18).color(0xE8E0DC));
        //             });
        //
        //         ui.element()
        //             .children(|ui| {
        //                 ui.text("Import", |f| f.font_size(18).color(0xE8E0DC));
        //             });
        //
        //         ui.element()
        //             .children(|ui| {
        //                 ui.text("Shuffle", |f| f.font_size(18).color(0xE8E0DC));
        //             });
        //     }
        //
        //     None => {
        //
        //     }
        // });

        ui.show(|_| {}).await;
        next_frame().await;
    }
}
