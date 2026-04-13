use std::{fs, process::exit};

use codespan_reporting::files::SimpleFiles;
use monte::frontend::{
    error::*,
    parser::{Lexer, Parser},
    sema::Analyzer,
    utils::*,
};

fn main() -> Result<(), Error> {
    let file_name = "src.monte";
    let file_content = fs::read_to_string(file_name).expect("cannot read file");

    let source = Source::new(file_name, &file_content);

    let mut files = SimpleFiles::new();
    let file_id = files.add(source.file_name.clone(), source.input);

    let mut reporter = ErrorReporter::new(file_id);

    let tokens = Lexer::new(&source, &mut reporter).tokenize();
    let mut parser = Parser::new(tokens);

    let program = parser.parse_program();

    for error in parser.get_errors() {
        reporter.report(error.clone());
    }

    if reporter.reported() > 0 {
        reporter.emit_all(&files);
        eprintln!("too many errors reported");
        exit(1);
    }

    if let Err(_err) = Analyzer::new(&mut reporter).analyze(&program) {
        reporter.emit_all(&files);
    }

    Ok(())
}
