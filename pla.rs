#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod read_PLA {
    use std::fs;
    use std::error::Error;
   // use std::io::prelude::*;
   // use std::io::BufReader;
   // use std::io::Read;
    use regex::Regex;


    pub fn read_from_file(filename : &'static str) -> Result<(), E> {

        let data: String = fs::read_to_string(filename)?;
        let re = Regex::new(r"([\-0-9]+) ([\-0-9]+)").unwrap();

        for line in re.captures_iter(data) {
            println!("{}", &line[1])
        }
        Ok(())
    }
}

fn main() {
    read_PLA::read_from_file("simonexpo.txt");

}
