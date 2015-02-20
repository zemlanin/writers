#![feature(old_io)]
#![feature(env)]
#![feature(old_path)]

extern crate mustache;
extern crate markdown;
extern crate "rustc-serialize" as rustc_serialize;

use std::old_io::fs;
use std::old_io::fs::PathExtensions;

#[derive(RustcEncodable)]
struct PostData {
    content: String,
}

#[derive(RustcEncodable)]
struct PageData {
    posts: Vec<PostData>,
}

macro_rules! try_print(
    ($expr:expr) => ({
        match $expr {
            Ok(val) => val,
            Err(err) => {
                println!("{:?}", err);
                return
            }
        }
    })
);

fn main() {
    let (input_path, output_path) = match lib::shell_args() {
        Some(val) => val,
        _ => return
    };
    let contents = try_print!(fs::readdir(&input_path));
    let mut file_paths = Vec::new();
    let mut dir_paths = Vec::new();

    let template = match lib::get_base_template(contents.clone()) {
        Some(val) => val,
        _ => return
    };

    for absolute_path in contents.into_iter() {
        if absolute_path.is_dir() {
            try_print!(lib::output_mkdir(&absolute_path, &input_path, &output_path));
            dir_paths.push(absolute_path.clone());
        } else {
            file_paths.push(absolute_path.clone());
        };
    };

    loop {
        let directory = match dir_paths.pop() {
            Some(val) => val,
            None => break,
        };

        for absolute_path in try_print!(fs::readdir(&directory)).into_iter() {
            if absolute_path.is_dir() {
                try_print!(lib::output_mkdir(&absolute_path, &input_path, &output_path));
                dir_paths.push(absolute_path.clone());
            } else {
                file_paths.push(absolute_path.clone());
            };
        };
    }

    for absolute_path in file_paths.iter().filter(|p| p.extension_str() == Some("md")) {
        let mut markdown_file = try_print!(fs::File::open(absolute_path));
        let markdown_string = try_print!(markdown_file.read_to_string());

        let page = PageData {
            posts: vec![PostData {
                content: markdown::to_html(&markdown_string[..]),
            }],
        };

        let relative_path_md = absolute_path.path_relative_from(&input_path).unwrap();
        let mut relative_path_html = output_path.clone();
        relative_path_html.push(relative_path_md.with_extension("html").as_str().unwrap());

        let mut output_file = fs::File::create(&relative_path_html);
        template.render(&mut output_file, &page).unwrap();
    }
}

mod lib {
    extern crate mustache;

    use std::env;
    use std::old_io;
    use std::old_io::fs;
    use std::old_io::USER_DIR;

    pub fn shell_args() -> Option<(Path, Path)> {
        let args: Vec<String> = env::args()
                                    .map(|x| x.to_string())
                                    .collect();
        let input_path: Path;
        let output_path: Path;

        match &args[1..] {
            [ref input_arg, ref output_arg] => {
                let current_dir = env::current_dir().unwrap();
                input_path = current_dir.join(input_arg);
                output_path = current_dir.join(output_arg);
                Some((input_path, output_path))
            },
            _ => {
                println!("Usage: {} input_path output_path", args[0]);
                None
            },
        }

    }

    fn get_output_target(input_target: &Path, input_path: &Path, output_path: &Path) -> Path {
        let relative_path = input_target.path_relative_from(input_path).unwrap();
        let mut output_target = output_path.clone();
        output_target.push(relative_path.as_str().unwrap());
        output_target
    }

    pub fn output_mkdir(input_target: &Path, input_path: &Path, output_path: &Path) -> old_io::IoResult<()> {
        let output_target = get_output_target(input_target, input_path, output_path);
        fs::mkdir_recursive(&output_target, USER_DIR)
    }

    pub fn get_base_template(input_contents: Vec<Path>) -> Option<mustache::Template> {
        let templates_in_input: Vec<Path> = input_contents
                                    .into_iter()
                                    .filter(|p| p.extension_str() == Some("mustache"))
                                    .collect();

        match &templates_in_input[..] {
            [ref template_path] => match mustache::compile_path(template_path.clone()) {
                Ok(result) => Some(result),
                Err(err) => {
                    println!("{:?}", err);
                    None
                }
            },
            _ => {
                print!("single .mustache file in input path is required");
                None
            }
        }
    }
}
