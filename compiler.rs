use regex::Regex;

fn main() {
    let version : u32 = std::env::args().nth(1).expect("No version number specified").parse().expect("Version number must be an integer");
    let input_file = std::env::args().nth(2).expect("No input file given");
    let data: String =  match fs::read_to_string(input_file) {
        Ok(v) => v,
        Err(_) => return Err("Problem opening or reading input file".to_string()),
    }
    let output = Vec::new();
    let asm_regex = Regex::new(r"([0-9A-Z]*[A-Z]) ([0-1])").unwrap(); //15TN makes this regex needlessly complicated

    if (version == 1100) || (version == 1000) {
        for line in re.captures_iter(&data) {
            let binary = match line[1] {
                "A2AAC" => 0b01111000
                "

            }
    }
}
