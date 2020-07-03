extern crate handlebars;
// #[macro_use]
extern crate serde_yaml;
//extern crate serde_json;

use handlebars::Handlebars;
use serde_yaml::Value;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut reg = Handlebars::new();
    // render without register
    let data: Value = serde_yaml::from_str("name: foo")?;
    println!(
        "{}",
        reg.render_template("Hello {{name}}", &data)?
    );

    // register template using given name
    reg.register_template_string("tpl_1", "Good afternoon, {{name}}")?;
    println!("{}", reg.render("tpl_1", &data)?);
    Ok(())
}