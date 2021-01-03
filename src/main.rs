use ini::Ini;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::DirEntry;
use walkdir::WalkDir;

fn find_desktop_entires(paths: Vec<impl AsRef<Path>>) -> impl Iterator<Item = DirEntry> {
    paths.into_iter().flat_map(|path| {
        WalkDir::new(path)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|path| {
                path.path()
                    .extension()
                    .map_or(false, |ext| ext == "desktop")
            })
    })
}

fn process_desktop_entries(files: impl Iterator<Item = DirEntry>) -> HashMap<String, Vec<String>> {
    let mut associations = HashMap::new();
    for file in files {
        let filename = file.path().to_string_lossy().to_string();
        debug!("Found desktop file {}", filename);
        let i = match Ini::load_from_file(&file.path()) {
            Ok(i) => i,
            Err(err) => {
                warn!("Failed to parse {}, {:#?}", &filename, err);
                continue;
            }
        };
        for (section, property) in i.iter() {
            if !section.map_or(false, |section| section == "Desktop Entry") {
                if let Some(section) = section {
                    warn!(
                        "Unrecognized section \"{}\" in desktop file {}",
                        section, &filename
                    );
                } else {
                    warn!("Missing section in desktop file {}", &filename);
                }
                continue;
            }
            let mimes = if let Some(mimes) = property.get("MimeType") {
                mimes
            } else {
                warn!("Missing MimeType in desktop file {}", &filename);
                continue;
            };
            for mime in mimes.split(';').filter(|m| !m.is_empty()) {
                debug!("xdg-mime default '{}' '{}'", &filename, &mime);
                associations
                    .entry(mime.to_owned())
                    .or_insert(Vec::new())
                    .push(filename.clone());
            }
        }
    }
    associations
}

fn do_association(mimetype: &str, programs: &[String]) {
    let choice = match programs.len() {
        0 => {
            error!(
                "There's an empty list of programs for the type \"{}\"!",
                mimetype
            );
            return;
        }
        1 => {
            eprintln!(
                "Magically selecting the only option ({}) for type \"{}\".",
                programs[0], mimetype
            );
            Some(&programs[0])
        }
        _ => {
            let index =
                match dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .items(programs)
                    .with_prompt(format!(
                        "Select the handler for \"{}\". (ESC or q to skip)",
                        mimetype
                    ))
                    .interact_opt()
                {
                    Ok(i) => i,
                    Err(err) => {
                        // Something went wrong reading the input, so just fail
                        error!("{}", err);
                        std::process::exit(1);
                    }
                };
            index.map(|i| &programs[i])
        }
    };
    if let Some(choice) = choice {
        println!("Running xdg-mime default '{}' '{}'", choice, mimetype);
        match Command::new("xdg-mime")
            .arg("default")
            .arg(choice)
            .arg(mimetype)
            .status()
            .expect("Failed to run xdg-mime!")
            .code()
        {
            Some(0) => (),
            Some(i) => {
                error!("xdg-mime exited with non-zero code {}!", i)
            }
            None => {
                error!("xdg-mime was terminated by signal, aborting.");
                std::process::exit(1);
            }
        };
    } else {
        info!("Skipping type {}", mimetype);
    }
}

fn main() {
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();
    if !dialoguer::console::Term::stderr().features().is_attended() {
        error!("This application requires that stderr is a terminal!");
        std::process::exit(1);
    }
    if let Err(err) = ctrlc::set_handler(move || {
        let term = dialoguer::console::Term::stderr();
        let _ = term.show_cursor();
    }) {
        error!(
            "Failed to set signal handler to restore cursor, continuing anyway.\n{}",
            err
        );
    }
    let mut desktop_file_locations: Vec<PathBuf> = vec![PathBuf::from("/usr/share/applications/")];
    if let Some(home) = dirs::home_dir() {
        desktop_file_locations.push(home.join(".local/share/applications/"))
    } else {
        warn!("Failed to find home directory!")
    }
    let desktop_files = find_desktop_entires(desktop_file_locations);
    let associations = process_desktop_entries(desktop_files);
    info!(
        "Found {} total options for {} total MIME types.",
        associations.values().flatten().count(),
        associations.keys().count()
    );
    for (mimetype, programs) in &associations {
        do_association(mimetype, programs);
    }
}
