#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod read_PLA {
    use std::fs;
    use std::error::Error;
   // use std::io::prelude::*;
   // use std::io::BufReader;
   // use std::io::Read;
    use regex::Regex;


    pub fn read_from_file(filename : &'static str) -> Result<()> {
        let file = File::open(filename)?;
        let mut reader = BufReader::new(file);
        let re = Regex::new(r"([\-0-9]+) ([\-0-9]+)").unwrap();

        for line in reader.lines() {
            println!("{}", line?);
        }

        Ok(())
    }
}

fn main() {
    read_PLA::read_from_file("simonexpo.txt");

}
