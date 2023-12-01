use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use proc_macro::{Literal, TokenStream, TokenTree};

fn get_string_literal(input: TokenStream) -> Option<String> {
    let mut input = input.into_iter();
    let literal = match input.next()? {
        TokenTree::Literal(lit) => lit,
        _ => return None,
    };
    if input.next().is_some() {
        None
    } else {
        Some(literal.to_string())
    }
}

fn open_file(path: &Path) -> File {
    match File::open(path) {
        Ok(file) => file,
        Err(err) => panic!("failed to open file '{}': {err}", path.display()),
    }
}

fn read_input_file(input: TokenStream) -> (PathBuf, String) {
    let path = get_string_literal(input).expect("expected string literal");
    let path = &path[1..path.len() - 1];

    match std::fs::read_to_string(path) {
        Ok(contents) => (path.into(), contents),
        Err(err) => panic!("failed to read file '{path}': {err}"),
    }
}

fn process_line(context: &Path, line_no: usize, line: &str, output: &mut String) {
    if let Some(path) = line.strip_prefix("#include ") {
        let path = &path[1..path.len() - 1];
        open_file(&context.join(path))
            .read_to_string(output)
            .expect("failed to read file");
    } else {
        panic!("unknown preprocessor directive: {line:?} (line {line_no})")
    }
}

/// Preprocesses a shader file and returns the resulting WGSL.
#[proc_macro]
pub fn preprocess(input: TokenStream) -> TokenStream {
    let (mut path, contents) = read_input_file(input);
    path.pop();

    let mut output = String::new();

    // Replace the lines that are meant to be processed by the preprocessor.
    for (index, line) in contents.lines().enumerate() {
        let line_number = index + 1;

        if line.starts_with('#') {
            process_line(&path, line_number, line, &mut output);
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }

    TokenTree::Literal(Literal::string(&output)).into()
}
