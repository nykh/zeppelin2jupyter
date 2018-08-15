extern crate clap;

use clap::{Arg, App};

trait FilenameManipulator {
    fn replace_extension(&self, new_ext: &str) -> String;
}

impl FilenameManipulator for String {
    fn replace_extension(&self, new_ext: &str) -> String {
        let mut v: Vec<&str> = self.split(".").collect();
        if let Some(last) = v.last_mut() {
            *last = new_ext;
        }
        v.join(".")
    }
}

fn main() {
    let matches = App::new("zeppelin2jupyter")
        .version("0.1.0")
        .author("nykh <nicholas.ykhuang@gmail.com>")
        .about("Convert zeppelin notebook to jupyter notebook")
        .arg(Arg::with_name("src")
                .required(true)
                .takes_value(true)
                .index(1)
                .help("Zeppelin notebook to convert"))
        .arg(Arg::with_name("dst")
                .required(false)
                .takes_value(true)
                .index(2)
                .help("File path to write to"))
        .get_matches();
    let src = matches.value_of("src")
                .map(String::from).unwrap();
    let dst = matches.value_of("dst")
                .map(String::from)
                .unwrap_or(src.replace_extension("ipynb"));
    println!("src={}", &src);
    println!("dst={}", &dst);
}