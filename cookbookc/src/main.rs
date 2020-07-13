extern crate handlebars;
extern crate serde_yaml;
extern crate clap;

use clap::{App, AppSettings, SubCommand};
use handlebars::Handlebars;
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
                                      .about("generates md form yml and writes that to disk, overwriting existing files"))
                          .get_matches();

    match matches.subcommand() {
        ("apply", _) => apply(),
        _ => unreachable!()
    }
}

fn apply() -> Result<(), Box<dyn Error>> {
    let reg  = load_templates()?;
    let recipes = load_data()?;
    // TODO validate that they have all required fields and structure
    // TODO find a clever way to use tags ?
    render_recipes(&reg, &recipes)
}

fn load_templates() -> Result<Handlebars<'static>, Box<dyn Error>> {
    let mut reg = Handlebars::new();
    reg.register_template_string("recipe", include_str!("recipe.md.tpl"))?;
    Ok(reg)
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

fn filename_for(recipe: &Value) -> PathBuf {
    let filename = recipe["title"].as_str().unwrap().to_lowercase().replace(" ", "-") + ".md";
    Path::new("../").join(recipe["section"].as_str().unwrap().to_lowercase().replace(" ", "-")).join(filename)
}

fn out_file_for(recipe: &Value) -> File {
    File::create(filename_for(recipe)).unwrap()
}

// FIXME renderer should really be a struct that carries the templates with it as an impl detail...
fn render_recipes(templates: &Handlebars, data: &Vec<Value>) -> Result<(), Box<dyn Error>> {
    for recipe in data.iter() {
        let mut out_file  = out_file_for(recipe);
        let recipe_md = render_recipe(templates, recipe)?;
        out_file.write_all(recipe_md.as_bytes())?;
    }
    Ok(())
}

fn render_recipe(templates: &Handlebars, data: &Value) -> Result<String, Box<dyn Error>> {
    Ok(format!("{}", templates.render("recipe", data)?))
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
        let templates = load_templates().unwrap();
        let got = render_recipe(&templates, &data).unwrap();
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
}