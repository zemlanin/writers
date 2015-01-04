#![feature(macro_rules)]
#![feature(old_orphan_check)]

extern crate mustache;
extern crate markdown;
extern crate "rustc-serialize" as rustc_serialize;

use std::io;
use std::io::fs;
use std::io::fs::PathExtensions;

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
                println!("{}", err);
                return
            }
        }
    })
);

macro_rules! try_option(
    ($expr:expr) => ({
        match $expr {
            Ok(val) => Some(val),
            Err(err) => {
                println!("{}", err);
                None
            }
        }
    })
);

fn main() {
    let (input_path, output_path) = try_print!(lib::shell_args());
    let contents = try_print!(fs::readdir(&input_path));
    let mut file_paths = Vec::new();
    let mut dir_paths = Vec::new();
    let mut template_option: Option<mustache::Template> = None;

    // TODO: pass base template as an argument
    for absolute_path in contents.iter().filter(|p| p.extension_str() == Some("mustache")) {
        if template_option.is_none() {
            template_option = try_option!(mustache::compile_path(absolute_path.clone()));
        } else {
            break;
        };
    }

    let template = match template_option {
        None => {
            println!("No base mustache template in input folder");
            return
        },
        Some(val) => val
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
        let mut markdown_file = try_print!(io::File::open(absolute_path));
        let markdown_string = try_print!(markdown_file.read_to_string());

        let page = PageData {
            posts: vec![PostData {
                content: markdown::to_html(markdown_string.as_slice()),
            }],
        };

        let relative_path_md = absolute_path.path_relative_from(&input_path).unwrap();
        let mut relative_path_html = output_path.clone();
        relative_path_html.push(relative_path_md.with_extension("html").as_str().unwrap());

        let mut output_file = io::File::create(&relative_path_html);
        template.render(&mut output_file, &page);
    }
}

mod lib {
    use std::os;
    use std::io;

    fn parse_path_opt(position: uint, default_value: &str) -> io::IoResult<Path> {
        let args = os::args();
        os::make_absolute(&if args.len() > position {
            Path::new(&args[position])
        } else {
            Path::new(default_value)
        })
    }

    pub fn shell_args() -> io::IoResult<(Path, Path)> {
        let input_arg = parse_path_opt(1, "../input/");
        let output_arg = parse_path_opt(2, "../output/");

        match (input_arg, output_arg) {
            (Ok(input_path), Ok(output_path)) => Ok((input_path, output_path)),
            (Err(msg), _) | (_, Err(msg)) => Err(msg)
        }
    }

    fn get_output_target(input_target: &Path, input_path: &Path, output_path: &Path) -> Path {
        let relative_path = input_target.path_relative_from(input_path).unwrap();
        let mut output_target = output_path.clone();
        output_target.push(relative_path.as_str().unwrap());
        output_target
    }

    pub fn output_mkdir(input_target: &Path, input_path: &Path, output_path: &Path) -> io::IoResult<()> {
        let output_target = get_output_target(input_target, input_path, output_path);
        io::fs::mkdir_recursive(&output_target, io::USER_DIR)
    }
}
