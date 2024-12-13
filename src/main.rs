use std::path::{Path, PathBuf};

use clap::{Parser, ValueHint};
use color_eyre::Result;
use glob::glob;
use tabv::{App, TableFile};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(value_hint = ValueHint::FilePath)]
    path: Option<PathBuf>,
}

fn find_csvs(path: &Path) -> Vec<TableFile> {
    let pattern = path.join("**/*.csv?").display().to_string();
    glob(&pattern)
        .expect("Failed to read glob pattern")
        .filter_map(|path| path.ok().map(TableFile::new))
        .collect()
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = args.path.unwrap_or_else(|| PathBuf::from("."));

    let files = if path.is_dir() {
        find_csvs(&path)
    } else {
        vec![TableFile::new(path)]
    };

    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new(files).run(terminal);
    ratatui::restore();
    app_result
}
