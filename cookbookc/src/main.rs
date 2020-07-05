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
    let recipes = load_data();
    // TODO validate that they have all required fields and structure
    // TODO find a clever way to use tags ?
    render_recipes(&reg, &recipes)
}

fn load_templates() -> Result<Handlebars<'static>, Box<dyn Error>> {
    let mut reg = Handlebars::new();
    reg.register_template_string("recipe", include_str!("recipe.md.tpl"))?;
    Ok(reg)
}

fn load_data() -> Vec<Value> {
    read_dir(Path::new("../src"))
        .unwrap()
        .map(|r| r.unwrap())
        .filter(|e| e.path().extension() == Some(OsStr::new("yml")))
        .map(|e| serde_yaml::from_reader(BufReader::new(File::open(e.path()).unwrap())).unwrap())
        .collect::<Vec<Value>>()
}

fn render_recipes(templates: &Handlebars, data: &Vec<Value>) -> Result<(), Box<dyn Error>> {
    for recipe in data.iter() {
        let filename = recipe["recipe"]["title"].as_str().unwrap().to_lowercase().replace(" ", "-") + ".md";
        let recipe_path = Path::new("../").join(recipe["recipe"]["section"].as_str().unwrap()).join(filename);
        let recipe_md = render_recipe(templates, recipe)?;
        File::create(recipe_path).unwrap().write_all(recipe_md.as_bytes())?;
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
}