use super::utils::Spanned;
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{
        emit_to_write_style,
        termcolor::{ColorChoice, StandardStream},
        Config,
    },
};

pub type Error = Spanned<String>;

impl Error {
    pub fn to_diagnostic(&self, file_id: usize) -> Diagnostic<usize> {
        Diagnostic::error()
            .with_message(&self.node)
            .with_labels(vec![Label::primary(
                file_id,
                self.span.start..self.span.end,
            )])
    }
}

#[derive(Debug, Default)]
pub struct ErrorReporter {
    file_id: usize,
    errors: Vec<Error>,
}

impl ErrorReporter {
    pub fn new(file_id: usize) -> Self {
        Self {
            file_id,
            errors: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn clear(&mut self) {
        self.errors.clear();
    }

    pub fn report(&mut self, error: Error) {
        self.errors.push(error);
    }

    pub fn emit_all<'a>(&self, files: &'a SimpleFiles<String, &'a str>) {
        let writer = StandardStream::stderr(ColorChoice::Auto);
        let config = Config::default();

        for error in &self.errors {
            let diagnostic = error.to_diagnostic(self.file_id);
            emit_to_write_style(&mut writer.lock(), &config, files, &diagnostic).unwrap();
        }
    }
}
