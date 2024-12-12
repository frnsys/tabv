use color_eyre::Result;
use glob::glob;
use tabv::{App, TableFile};

fn find_csvs() -> Vec<TableFile> {
    glob("./**/*.csv")
        .expect("Failed to read glob pattern")
        .filter_map(|path| {
            path.ok().map(|path| {
                let name = path
                    .file_stem()
                    .expect("No valid file stem")
                    .to_string_lossy()
                    .to_string();
                TableFile {
                    name,
                    path,
                    records: None,
                }
            })
        })
        .collect()
}

fn main() -> Result<()> {
    let files = find_csvs();

    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new(files).run(terminal);
    ratatui::restore();
    app_result
}
