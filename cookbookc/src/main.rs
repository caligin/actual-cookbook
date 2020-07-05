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
    let mut reg = Handlebars::new();
    let recipes = read_dir(Path::new("../src"))
        .unwrap()
        .map(|r| r.unwrap())
        .filter(|e| e.path().extension() == Some(OsStr::new("yml")))
        .map(|e| serde_yaml::from_reader(BufReader::new(File::open(e.path()).unwrap())).unwrap())
        .collect::<Vec<Value>>();

    // TODO validate that they have all required fields and structure

    // TODO find a clever way to use tags
    reg.register_template_string("recipe", include_str!("recipe.md.tpl"))?;
    for recipe in recipes.iter() {
        let filename = recipe["recipe"]["title"].as_str().unwrap().to_lowercase().replace(" ", "-") + ".md";
        let recipe_path = Path::new("../").join(recipe["recipe"]["section"].as_str().unwrap()).join(filename);
        let recipe_md = format!("{}", reg.render("recipe", recipe)?);
        File::create(recipe_path).unwrap().write_all(recipe_md.as_bytes())?;
    }
    Ok(())
}