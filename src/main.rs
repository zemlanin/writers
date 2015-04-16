#![feature(core)]
#![feature(std_misc)]
#![feature(path_ext)]
#![feature(slice_patterns)]
#![feature(path_relative_from)]

extern crate core;
extern crate mustache;
// extern crate markdown;
extern crate rustc_serialize;

use std::fs;
use core::ops::Deref;
use std::ffi::OsStr;
use std::io::prelude::*;

use std::borrow::ToOwned;

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
    let template = try_print!(lib::get_base_template(&input_path));

    let mut file_paths = Vec::new();
    let mut dir_paths = Vec::new();

    for absolute_entry in try_print!(fs::read_dir(&input_path)) {
        let absolute_entry = try_print!(absolute_entry);
        let absolute_path = absolute_entry.path().deref().to_owned();

        if absolute_path.is_dir() {
            try_print!(lib::output_mkdir(&absolute_path, &input_path, &output_path));
            dir_paths.push(absolute_path);
        } else {
            file_paths.push(absolute_path);
        };
    };

    loop {
        let directory = match dir_paths.pop() {
            Some(val) => val,
            None => break,
        };

        for absolute_entry in try_print!(fs::read_dir(&directory)) {
            let absolute_entry = try_print!(absolute_entry);
            let absolute_path = absolute_entry.path().deref().to_owned();

            if absolute_path.is_dir() {
                try_print!(lib::output_mkdir(&absolute_path, &input_path, &output_path));
                dir_paths.push(absolute_path);
            } else {
                file_paths.push(absolute_path);
            };
        };
    }

    for absolute_path in file_paths
                            .into_iter()
                            .filter(|p| p.extension() == Some(OsStr::new("md"))) {
        let mut markdown_file = try_print!(fs::File::open(&absolute_path));
        let mut markdown_string = String::new();
        try_print!(markdown_file.read_to_string(&mut markdown_string));

        let page = PageData {
            posts: vec![PostData {
                // content: markdown::to_html(&markdown_string[..]),
                content: markdown_string,
            }],
        };

        let relative_path_md = absolute_path.relative_from(&input_path).unwrap();
        let mut relative_path_html = output_path.clone();
        relative_path_html.push(
            &relative_path_md.with_extension("html").into_os_string()
        );

        let mut bytes = vec![];
        try_print!(template.render(&mut bytes, &page));

        let mut output_file = try_print!(fs::File::create(&relative_path_html));
        try_print!(output_file.write(&bytes));
    }
}

mod lib {
    extern crate mustache;

    use std::io;
    use std::env;
    use std::fs;
    use std::fs::File;
    use std::ffi::AsOsStr;
    use std::io::prelude::*;
    use std::path::{Path, PathBuf};

    pub fn shell_args() -> Option<(PathBuf, PathBuf)> {
        let args: Vec<String> = env::args()
                                    .map(|x| x.to_string())
                                    .collect();
        let input_path: PathBuf;
        let output_path: PathBuf;

        match &args[1..] {
            [ref input_arg, ref output_arg] => {
                input_path = PathBuf::from(input_arg);
                output_path = PathBuf::from(output_arg);
                Some((input_path, output_path))
            },
            _ => {
                println!("Usage: {} input_path output_path", args[0]);
                None
            },
        }
    }

    fn get_output_target(input_target: &Path, input_path: &PathBuf, output_path: &PathBuf) -> PathBuf {
        let relative_path = input_target.relative_from(input_path).unwrap();
        let mut output_target = output_path.clone();
        output_target.push(relative_path);
        output_target
    }

    pub fn output_mkdir(input_target: &Path, input_path: &PathBuf, output_path: &PathBuf) -> io::Result<()> {
        let output_target = get_output_target(input_target, input_path, output_path);
        fs::create_dir_all(&output_target)
    }

    pub fn get_base_template(input_path: &PathBuf) -> Result<mustache::Template, io::Error> {
        let mut template_path = input_path.clone();
        template_path.push("base.mustache");

        let mut f = try!(File::open(template_path.as_os_str()));
        let mut s = String::new();
        try!(f.read_to_string(&mut s));

        Ok(mustache::compile_str(&s))
    }
}
