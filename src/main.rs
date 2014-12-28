use std::os;
use std::io::fs;

fn main() {
    let args = os::args();
    let input_path = if args.len() == 2 {
        Path::new(&args[1])
    } else {
        Path::new("../input") // because of sublime text's cargo build
    };

    let contents = match fs::readdir(&input_path) {
        Ok(result) => result,
        Err(_) => {
            println!("Enter input directory as a single argument");
            return
        }
    };

    for inner_path in contents.iter() {
        println!("{}", os::make_absolute(inner_path).unwrap().display())
    }
}
