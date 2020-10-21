#![allow(dead_code, unused_variables)]

pub mod ast;
pub mod exec;
pub mod parser;
pub mod result;
pub mod types;
mod utils;

use std::{path::Path, path::PathBuf};

use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    drawer::DrawerBuilder,
    light::LightProps,
    processes::{pipe_to_magick, wait_for_magick},
    utils as gfxutils, PPMImg,
};

use self::{
    ast::{Command, VaryInfo},
    exec::{exec_no_animation, exec_once_with_animation},
    parser::{parse_file, SymTable},
    result::EngineResult,
};

/// MDL Interpreter for a single file
pub struct Interpreter {
    filename: PathBuf,
}

/// Config for interpreter to exec script
pub enum ExecContext {
    Animation {
        script: Vec<String>,
        cmd_list: Vec<(usize, Command)>,
        basename: String,
        frames: u32,
        vary_list: Vec<(usize, VaryInfo)>,
        light_props: SymTable<LightProps>,
    },
    NoAnimation {
        script: Vec<String>,
        cmd_list: Vec<(usize, Command)>,
        basename: String,
        light_props: SymTable<LightProps>,
    },
}

impl Interpreter {
    pub fn new<T: AsRef<Path>>(path: T) -> Self {
        Self {
            filename: path.as_ref().to_path_buf(),
        }
    }

    pub fn run(&self) -> EngineResult<()> {
        let pgbar = ProgressBar::new_spinner().with_style(gfxutils::shark_spinner_style());
        pgbar.set_message("Parsing file");
        pgbar.enable_steady_tick(120);

        match parse_file(&self.filename)? {
            ExecContext::Animation {
                cmd_list,
                basename,
                frames,
                vary_list,
                script,
                light_props,
            } => {
                pgbar.set_message("Computing animation knobs");
                // second pass, compute all knob values for each frame
                let mut knob_states: Vec<SymTable<f64>> = vec![];

                for cur_frame in 0..frames {
                    let mut table: SymTable<f64> = SymTable::new();
                    for (line, v) in vary_list.iter() {
                        if v.start_frame <= cur_frame && cur_frame <= v.end_frame {
                            let val = (v.end_val - v.start_val)
                                / (v.end_frame as f64 - v.start_frame as f64)
                                * (cur_frame - v.start_frame) as f64
                                + v.start_val;

                            // override previous vary command if they overlap in frame numbers
                            if table.insert(v.knob.to_owned(), val).is_some() {
                                eprintln!(
                                    "Vary commands cannot overlap. Using vary on line {}: {}",
                                    line,
                                    script.get(*line).unwrap().as_str()
                                );
                            }
                        }
                    }
                    knob_states.push(table);
                }

                let fout_name = format!("{}.gif", basename);

                let mut magick = pipe_to_magick(vec!["-delay", "1.7", "ppm:-", &fout_name]);
                let writer = magick.stdin.take().unwrap();
                let mut drawer = DrawerBuilder::new(PPMImg::new(500, 500, 255))
                    .with_writer(Box::new(writer))
                    .build();

                pgbar.finish_and_clear();
                let render_pg = ProgressBar::new(knob_states.len() as u64).with_style(
                    ProgressStyle::default_bar().template(
                        "\t[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
                    ),
                );
                render_pg.set_message("Rendering frames");

                for knob_state in knob_states.iter() {
                    exec_once_with_animation(
                        &cmd_list,
                        &script,
                        knob_state,
                        &mut drawer,
                        &light_props,
                    )?;
                    drawer.flush()?;
                    drawer.reset_stack();
                    drawer.clear();
                    render_pg.inc(1);
                }

                drawer.finish()?;

                render_pg.finish_and_clear();
                let magick_pg = ProgressBar::new_spinner()
                    .with_style(ProgressStyle::default_spinner().template(""));
                magick_pg.enable_steady_tick(120);
                // magick_pg.tick();

                magick_pg.set_style(gfxutils::shark_spinner_style());
                magick_pg.set_message("Waiting for magick gif generation");
                let status = wait_for_magick(magick);
                magick_pg.finish_with_message(&format!(
                    "Done. `magick` {}. Animation saved as \"{}\"",
                    status, fout_name
                ));
            }
            ExecContext::NoAnimation {
                script,
                cmd_list,
                basename,
                light_props,
            } => {
                let mut drawer = DrawerBuilder::new(PPMImg::new(500, 500, 255)).build();
                pgbar.println("\tAnimation not detected. Rendering still image.");
                // pgbar.set_message("Drawing image");
                exec_no_animation(cmd_list, &script, &light_props, &mut drawer, &pgbar)?;
                pgbar.finish_with_message("Done.");
            }
        }

        Ok(())
    }
}
