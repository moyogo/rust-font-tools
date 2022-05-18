//! A fonticulously fast variable font builder
mod basictables;
mod buildbasic;
mod fontinfo;
mod glyph;
mod kerning;
mod notdef;
mod utils;

use buildbasic::build_font;
use clap::Parser;

/// A fonticulusly fast font builder
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Only convert the given glyphs (for testing only)
    #[clap(short, long)]
    subset: Option<String>,

    /// Don't make a variable font, make a static font for each master
    #[clap(long)]
    masters: bool,

    /// Increase logging
    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,

    input: String,
    output: Option<String>,
}

// use rayon::prelude::*;
use std::collections::HashSet;
use std::io;
use std::path::PathBuf;

/*
    OK, here is the basic plan:

    1) This function handles command line stuff, uses babelfont-rs to load
       the source file(s) into memory, and calls into buildbasic::build_font.
    2) The build_font function in buildbasic.rs coordinates the build.
    3) basictables.rs creates the non-glyph, non-layout, non-variable metadata tables
       (that is: head, hhea, maxp, OS/2, hmtx, cmap, glyf, name, post, loca).
    3a) fontinfo.rs works out what some of the stuff in those tables should be.
    4) glyph.rs handles Babelfont->OT glyph conversion, creating the glyf and gvar
       table entries for each glyph.
    5) babelfont-rs creates the variable metadata tables (fvar,avar).
    6) We come back here and save the files at the end.
*/

fn main() {
    // Command line handling
    let args = Args::parse();

    env_logger::init_from_env(env_logger::Env::default().filter_or(
        env_logger::DEFAULT_FILTER_ENV,
        match args.verbose {
            0 => "warn",
            1 => "info",
            _ => "debug",
        },
    ));

    // If we are only handling a subset of the glyphs (usually for debugging
    // purposes), split that into a set here.
    let subset: Option<HashSet<String>> = args
        .subset
        .as_ref()
        .map(|x| x.split(',').map(|y| y.to_string()).collect());

    let mut in_font = load_with_babelfont(&args.input);

    // --masters means we produce a TTF for each master and don't do interpolation
    if args.masters {
        create_ttf_per_master(&mut in_font, subset);
    } else {
        create_variable_font(&mut in_font, subset, &args.output);
    }
}

fn load_with_babelfont(filename: &str) -> babelfont::Font {
    if filename.ends_with(".designspace") {
        babelfont::convertors::designspace::load(PathBuf::from(filename))
            .expect("Couldn't load source")
    } else if filename.ends_with(".ufo") {
        babelfont::convertors::ufo::load(PathBuf::from(filename)).expect("Couldn't load source")
    } else if filename.ends_with(".glyphs") {
        babelfont::convertors::glyphs3::load(PathBuf::from(filename)).expect("Couldn't load source")
    } else {
        panic!("Unknown file type {:?}", filename);
    }
}

fn create_ttf_per_master(in_font: &mut babelfont::Font, subset: Option<HashSet<String>>) {
    let family_name = in_font
        .names
        .family_name
        .default()
        .unwrap_or_else(|| "New Font".to_string());
    let master_names: Vec<String> = in_font
        .masters
        .iter()
        .enumerate()
        .map(|(ix, master)| {
            let master_name = master
                .name
                .default()
                .unwrap_or_else(|| format!("Master{}", ix));
            if master_name == "Unnamed master" {
                format!("Master{}", ix)
            } else {
                master_name
            }
        })
        .collect();
    for (ix, master_name) in master_names.iter().enumerate() {
        let mut out_font = build_font(in_font, &subset, Some(ix));
        log::info!("Building {}", master_name);
        out_font
            .save(format!("{}-{}.ttf", family_name, master_name))
            .expect("Could not write font");
    }
}

fn create_variable_font(
    in_font: &mut babelfont::Font,
    subset: Option<HashSet<String>>,
    output: &Option<String>,
) {
    let mut out_font;
    if in_font.masters.len() > 1 {
        out_font = build_font(in_font, &subset, None);
        // Ask babelfont to make fvar/avar
        in_font
            .add_variation_tables(&mut out_font)
            .expect("Couldn't add variation tables")
    } else {
        out_font = build_font(in_font, &subset, Some(0));
    }

    match output {
        Some(filename) => out_font.save(filename).expect("Could not write font"),
        None => out_font.write(io::stdout()).expect("Could not write font"),
    }
}
