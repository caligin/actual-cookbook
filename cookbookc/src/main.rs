extern crate handlebars;
extern crate serde;
extern crate serde_yaml;
extern crate serde_json;
extern crate clap;

use clap::{App, AppSettings, SubCommand};
use handlebars::Handlebars;
use regex::Regex;
use serde::{Serialize, Deserialize};
use serde_yaml::Value;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::read_dir;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;


fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("cookbookc")
                          .version("1.0")
                          .author("0xf00 <foo@anima.tech>")
                          .about("Cookbook build tool")
                          .setting(AppSettings::ArgRequiredElseHelp)
                          .subcommand(SubCommand::with_name("apply")
                                      .about("generates md from yml and writes that to disk, overwriting existing files"))
                          .subcommand(SubCommand::with_name("export")
                                      .about("generates single-file json from yml and writes that to disk, overwriting if existing"))
                          .get_matches();

    match matches.subcommand() {
        ("apply", _) => apply(),
        ("export", _) => export(),
        _ => unreachable!()
    }
}

fn apply() -> Result<(), Box<dyn Error>> {
    // TODO validate that they have all required fields and structure
    // TODO find a clever way to use tags ?
    let renderer = Renderer::new();
    let recipes = load_data()?;
    for recipe in recipes.iter() {
        let filename = filename_for(&recipe);
        let mut out_file = ensure_file(&filename);
        let recipe_md = renderer.render_recipe(recipe)?;
        out_file.write_all(recipe_md.as_bytes())?;
    }
    Ok(())

}

fn export() -> Result<(), Box<dyn Error>> {
    let recipes = load_data()?;
    let cookbook = Cookbook::new(recipes);
    let filename = Path::new("../cookbook.json");
    let mut out_file = ensure_file(&filename);
    out_file.write_all(&serde_json::to_vec(&cookbook)?)?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Cookbook {
    recipes: Vec<Value>,
}

impl Cookbook {
    fn new(recipes: Vec<Value>) -> Cookbook {
        Cookbook{ recipes }
    }
}

struct Renderer {
    templates: Handlebars<'static>,
}

impl Renderer {
    fn new() -> Renderer {
        let mut reg = Handlebars::new();
        reg.register_template_string("recipe", include_str!("recipe.md.tpl"))
            .unwrap();
        Renderer {templates: reg}
    }

    fn render_recipe(&self, recipe: &Value) -> Result<String, Box<dyn Error>> {
        Ok(format!("{}", self.templates.render("recipe", recipe)?))
    }
}


fn filename_for(recipe: &Value) -> PathBuf {
    let filename = translate_filename_component(recipe["title"].as_str().unwrap()) + ".md";
    Path::new("../").join(translate_filename_component(recipe["section"].as_str().unwrap())).join(filename)
}

fn translate_filename_component(component: &str) -> String {
    let multidash = Regex::new("-+").unwrap();
    let trimdash = Regex::new("^-*(.+?)-*$").unwrap();
    let component = component
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric(), "-");
    trimdash.replace_all(&multidash.replace_all(&component, "-"), "$1").into_owned()
}

fn ensure_file(path: &Path) -> File {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    File::create(path).unwrap()
}

#[derive(Debug)]
struct LoadError {
    path: Box<Path>,
    source: serde_yaml::Error,
}

impl LoadError {
    fn new(path: &Path, source: serde_yaml::Error) -> LoadError {
        LoadError{ path: Box::from(path), source }
    }
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "error loading file {}: ", self.path.display())?;
        self.source.fmt(f)?;
        Ok(())
    }
}

impl Error for LoadError {}

fn load_data() -> Result<Vec<Value>, LoadError> {
    read_dir(Path::new("../src"))
        .unwrap()
        .map(|r| r.unwrap())
        .filter(|e| e.path().extension() == Some(OsStr::new("yml")))
        .map(|entry| serde_yaml::from_reader(BufReader::new(File::open(entry.path()).unwrap()))
            .map_err(|err| LoadError::new(&entry.path(), err)))
        .collect::<Result<Vec<Value>, LoadError>>()
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_yaml::from_str;

    // FIXME: this stuff is more like an integration test tbh, should go in /test not /sr
    // FIXME: the tests in this form is actually needed because we're using a loose type like a value in the template
    #[test]
    fn recipe_renders_fully_when_all_recipe_fields_specified() {
        let data: Value = from_str(include_str!("../fixtures/test-drink.yml")).unwrap();
        let renderer = Renderer::new();
        let got = renderer.render_recipe(&data).unwrap();
        assert_eq!(include_str!("../fixtures/test-drink.md"), got);
    }

    #[test]
    fn filename_for_yields_path_based_on_section_and_title() {
        let recipe: Value = from_str(r#"
            title: test
            section: taste
            "#).unwrap();
        let got = filename_for(&recipe);
        assert_eq!(PathBuf::new().join("../taste/test.md"), got)
    }

    #[test]
    fn filename_for_yields_path_downcased_and_kebabcased() {
        let recipe: Value = from_str(r#"
            title: te ST
            section: TaS tE
            "#).unwrap();
        let got = filename_for(&recipe);
        assert_eq!(PathBuf::new().join("../tas-te/te-st.md"), got)
    }

    #[test]
    fn filename_for_yields_path_translating_slashes_to_dashes() {
        let recipe: Value = from_str(r#"
            title: te/ST
            section: TaS tE
            "#).unwrap();
        let got = filename_for(&recipe);
        assert_eq!(PathBuf::new().join("../tas-te/te-st.md"), got)
    }
 
    #[test]
    fn filename_for_yields_path_translating_punctuation_to_dashes() {
        let recipe: Value = from_str(r#"
            title: te<ST
            section: TaS>tE
            "#).unwrap();
        let got = filename_for(&recipe);
        assert_eq!(PathBuf::new().join("../tas-te/te-st.md"), got)
    }

    #[test]
    fn filename_for_yields_path_with_nonrepeating_dashes() {
        let recipe: Value = from_str(r#"
            title: te<>/.ST
            section: taste
            "#).unwrap();
        let got = filename_for(&recipe);
        assert_eq!(PathBuf::new().join("../taste/te-st.md"), got)
    }
    #[test]
    fn filename_for_yields_path_without_edge_dashes() {
        let recipe: Value = from_str(r#"
            title: --<<test>>--
            section: --<<taste>>--
            "#).unwrap();
        let got = filename_for(&recipe);
        assert_eq!(PathBuf::new().join("../taste/test.md"), got)
    }
}