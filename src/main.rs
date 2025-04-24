use freedesktop_entry_parser::parse_entry;
use std::{collections::HashMap, env, path::Path};
use walkdir::WalkDir;

#[derive(Debug)]
struct Application {
    name: String,
    xdg_type: String,
    exec: String,
    icon: String,
    categories: Vec<String>,
    no_display: bool,
    path: String,
}

enum AppError {
    IO(std::io::Error),
    MissingData,
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::IO(value)
    }
}

fn get_application(path: &Path) -> Result<Application, AppError> {
    let entry = parse_entry(path)?;
    Ok(Application {
        name: entry
            .section("Desktop Entry")
            .attr("Name")
            .ok_or(AppError::MissingData)?
            .to_owned(),
        exec: entry
            .section("Desktop Entry")
            .attr("Exec")
            .ok_or(AppError::MissingData)?
            .to_owned(),
        xdg_type: entry
            .section("Desktop Entry")
            .attr("Type")
            .unwrap_or("No Type")
            .to_owned(),
        icon: entry
            .section("Desktop Entry")
            .attr("Icon")
            .unwrap_or("")
            .to_owned(),
        categories: entry
            .section("Desktop Entry")
            .attr("Categories")
            .unwrap_or("None")
            .to_owned()
            .split(";")
            .map(str::to_owned)
            .filter(|s| !s.is_empty())
            .collect(),
        path: path.to_str().unwrap_or("No path?").to_owned(),
        no_display: entry
            .section("Desktop Entry")
            .attr("NoDisplay")
            .eq(&Some("true")),
    })
}

fn get_desktop_entries(path: &Path) -> impl Iterator<Item = Application> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .map(|de| de.path().to_owned())
        .map(|p| get_application(&p))
        .filter_map(Result::ok)
}

fn main() {
    let x_include = vec!["X-Wine"];
    let alias = HashMap::from([
        (
            "Graphics",
            vec![
                "2DGraphics",
                "3DGraphics",
                "RasterGraphics",
                "VectorGraphics",
            ],
        ),
        ("Video", vec!["Player", "TV"]),
        (
            "Office",
            vec!["Presentation", "Spreadsheet", "WordProcessor"],
        ),
    ]);
    let skip = vec![
        "Player",
        "TV",
        "2DGraphics",
        "3DGraphics",
        "RasterGraphics",
        "VectorGraphics",
        "Presentation",
        "Spreadsheet",
        "WordProcessor",
    ];
    let exclusive = vec!["Game"];
    let rename = HashMap::from([("X-Wine", "Wine")]);

    let data_dirs = env::var("XDG_DATA_DIRS").unwrap_or("/usr/share".to_owned());
    let entries: Vec<Application> = data_dirs
        .split(":")
        .map(|path| {
            let mut full = path.to_owned();
            full.push_str("/applications");
            full
        })
        .flat_map(|p| get_desktop_entries(Path::new(&p)))
        .filter(|app| !app.no_display)
        .filter(|app| app.xdg_type == "Application")
        .collect();
    let mut cats: Vec<String> = vec![];
    for entry in entries.iter() {
        let ex = entry
            .categories
            .iter()
            .find(|cat| exclusive.contains(&cat.as_str()));

        let test_cats = match ex {
            Some(cat) => &vec![cat.clone()],
            None => &entry.categories,
        };
        for cat in test_cats.iter() {
            if !cats.contains(&cat) {
                if cat.starts_with("X-") && !x_include.contains(&cat.as_str()) {
                    continue;
                }
                if skip.contains(&cat.as_str()) {
                    continue;
                }
                cats.push(cat.clone());
            }
        }
    }
    cats.sort();
    for cat in cats {
        let name: &str = if rename.contains_key(&cat.as_str()) {
            rename.get(&cat.as_str()).unwrap().to_owned()
        } else {
            &cat
        };
        println!("<menu id=\"{name}\" label=\"{name}\">", name=name);

        let aliases = alias.get(&cat.as_str());
        for app in entries.iter().filter(|a| {
            a.categories.contains(&cat)
                || a.categories
                    .iter()
                    .any(|c| aliases.map(|al| al.contains(&c.as_str())).unwrap_or(false))
        }) {
            // println!(" - {}", app.name);
            println!("\t<item label=\"{name}\" icon=\"{icon}>\"", name=app.name, icon=app.icon);
            println!("\t\t<action name=\"Execute\">");
            println!("\t\t\t<command>{}</command>", app.exec);
            println!("\t\t</action>");
            println!("\t</item>");
        }
        println!("</menu>");
    }
}
