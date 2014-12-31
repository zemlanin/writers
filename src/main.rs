use std::os;
use std::io::{fs, USER_DIR};

fn main() {
    let [input_path, output_path] = match lib::shell_args().as_slice() {
        [Ok(ref path_1), Ok(ref path_2)] => [path_1, path_2],
        _ => {
            println!("Error while parsing args");
            return
        }
    };

    println!("{} {}", input_path.display(), output_path.display());

    let contents = match fs::readdir(input_path) {
        Ok(result) => result,
        Err(_) => {
            println!("Enter input directory as a first argument");
            return
        }
    };

    fs::mkdir_recursive(output_path, USER_DIR);

    for inner_path in contents.iter() {
        let absolute_path = os::make_absolute(inner_path).unwrap();
        println!("{}", absolute_path.display());
    }
}

mod lib {
    use std::os;
    use std::io::IoResult;

    pub fn shell_args() -> Vec<IoResult<Path>> {
        let args = os::args();
        let input_path = if args.len() > 1 {
            os::make_absolute(&Path::new(&args[1]))
        } else {
            os::make_absolute(&Path::new("../input/")) // because of sublime text's cargo build
        };

        let output_path = if args.len() > 2 {
            os::make_absolute(&Path::new(&args[2]))
        } else {
            os::make_absolute(&Path::new("../output/")) // because of sublime text's cargo build
        };

        return vec!(input_path, output_path)
    }
}
