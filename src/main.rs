use std::io::{fs, USER_DIR};

fn main() {
    let (input_path, output_path) = match lib::shell_args() {
        Ok((path_1, path_2)) => (path_1, path_2),
        _ => {
            println!("Error while parsing opts");
            return
        }
    };

    println!("{} {}", input_path.display(), output_path.display());

    let contents = match fs::readdir(&input_path) {
        Ok(result) => result,
        Err(_) => {
            println!("Enter input directory as a first argument");
            return
        }
    };

    match fs::mkdir_recursive(&output_path, USER_DIR) {
        Err(msg) => {
            println!("{}", msg);
            return
        },
        _ => {}
    };

    for absolute_path in contents.iter() {
        println!("{}", absolute_path.display());

        if absolute_path.display().to_string().ends_with("html") {
            println!("tmpl!");
        };
    }
}

mod lib {
    use std::os;
    use std::io;

    fn parse_path_opt(position: uint, default_value: String) -> io::IoResult<Path> {
        let args = os::args();
        os::make_absolute(&if args.len() > position {
            Path::new(&args[position])
        } else {
            Path::new(default_value) // because of sublime text's cargo build
        })
    }

    pub fn shell_args() -> io::IoResult<(Path, Path)> {
        let input_arg = parse_path_opt(1, "../input/".to_string());
        let output_arg = parse_path_opt(2, "../output/".to_string());

        match (input_arg, output_arg) {
            (Ok(input_path), Ok(output_path)) => Ok((input_path, output_path)),
            (Err(msg), _) | (_, Err(msg)) => Err(msg)
        }
    }
}
