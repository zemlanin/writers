extern crate mustache;
extern crate markdown;
extern crate "rustc-serialize" as rustc_serialize;

use std::io;
use std::io::{fs, USER_DIR};

#[deriving(RustcEncodable)]
struct PostData {
    content: String,
}

fn main() {
    let (input_path, output_path) = match lib::shell_args() {
        Ok((path_1, path_2)) => (path_1, path_2),
        Err(msg) => {
            println!("Error while parsing opts: {}", msg);
            return
        }
    };

    let contents = match fs::readdir(&input_path) {
        Ok(result) => result,
        Err(_) => {
            println!("Enter input directory as a first argument");
            return
        }
    };

    match fs::mkdir_recursive(&output_path, io::USER_DIR) {
        Err(msg) => {
            println!("{}", msg);
            return
        },
        _ => {}
    };

    let mut template: Option<mustache::Template> = None;
    let file_ext_closure = |ext, p: &Path| p.as_str().unwrap().ends_with(ext);

    for absolute_path in contents.iter().filter(|&p| file_ext_closure("mustache", p)) {
        if template.is_none() {
            template = match mustache::compile_path(absolute_path.clone()) {
                Ok(result) => Some(result),
                Err(msg) => {
                    println!("{}", msg); None
                }
            };
        };
    }

    if template.is_none() {
        println!("No base mustache in input folder");
        return
    }

    for absolute_path in contents.iter().filter(|&p| file_ext_closure("md", p)) {
        println!("{}", absolute_path.display());
        let mut markdown_file = match io::File::open(absolute_path) {
            Ok(result) => result,
            Err(msg) => {
                println!("{}", msg);
                return
            }
        };

        let markdown_string = match markdown_file.read_to_string() {
            Ok(result) => markdown::to_html(result.as_slice()),
            Err(msg) => {
                println!("{}", msg);
                return
            }
        };

        let post = PostData {
            content: markdown_string
        };

        println!("{}", post.content);

        // rust-mustache still depends on deprecated Encodable
        // let rendered_mustache = template.unwrap().render(&mut io::stdout(), &vec!(post));
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
            Path::new(default_value)
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
