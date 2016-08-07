mod folder;
mod lifetime_walker;
mod loader;
mod refactor_calls;
mod util;

use std::{env, thread};

use rustc::lint;
use rustc::session::{self, config};
use rustc_driver::{CompilerCalls, Compilation, diagnostics_registry, driver,
                   handle_options, monitor};
use rustc_lint;
use syntax::diagnostic;
use syntax::ast::NodeId;
use syntax::codemap::{self, FileLoader};

use self::loader::ReplaceLoader;
use self::refactor_calls::RefactorCalls;
use self::util::{make_input, make_output};
use refactor::{RefactorType, Response};

// Used to check same-block conflicts in modules (since name resolution doesn't seem to).
pub fn check_reduced_graph(root: String,
                       files: Vec<(String, String)>,
                       new_name: String,
                       node: NodeId)
                       -> Result<(), Response> {
    match run_resolution(root, Some(files), None, RefactorType::Reduced, new_name, node, false) {
        Ok(()) => Ok(()),
        Err(_) => Err(Response::Conflict)
    }
}

// What does this do?
pub fn run_resolution(root: String,
                      file_override: Option<Vec<(String, String)>>,
                      working_file: Option<String>,
                      kind: RefactorType,
                      new_name: String,
                      node: NodeId,
                      full: bool)
                      -> Result<(), (usize, usize, String, i32)> {
    let key = "RUST_FOLDER";
    let mut path = String::new();
    let args = match env::var(key) {
        Ok(val) => {
            path.push_str("-L");
            path.push_str(&val[..]);
            vec!["refactor".to_owned(),
                path,
                "-Z".to_owned(), "keep_mtwt_tables".to_owned(),
                //"--crate-type", "lib",
                root]
        }
        Err(_) => vec!["refactor".to_owned(), root],
    };

    let mut loader = ReplaceLoader::new();
    match file_override.as_ref() {
        Some(input) => {
            for file in input.iter() {
                loader.add_file(file.0.clone(), file.1.clone());
            }
        },
        None => ()
    }

    thread::catch_panic(move || {
        let mut call_ctxt = RefactorCalls::new(kind, new_name, node, file_override,
                                               working_file, full);
        // Calling monitor fixes a bug where this process is put into an
        // invalid (logging) state.
        match kind {
            RefactorType::InlineLocal |
            RefactorType::ReifyLifetime |
            RefactorType::ElideLifetime => run_compiler(&args, &mut call_ctxt, Box::new(loader)),
            _ => monitor(move || run_compiler(&args, &mut call_ctxt, Box::new(loader)))
        }
    }).map_err(|any|
        *any.downcast().ok().unwrap_or_default()
    )
}

// Basically a clone of the function librustc_driver
fn run_compiler<'a, T: CompilerCalls<'a>>(args: &[String], callbacks: &mut T, loader: Box<FileLoader>) {
    macro_rules! do_or_return {($expr: expr) => {
        match $expr {
            Compilation::Stop => return,
            Compilation::Continue => {}
        }
    }}

    let matches = match handle_options(args.to_vec()) {
        Some(matches) => matches,
        None => return
    };

    let sopts = config::build_session_options(&matches);

    let descriptions = diagnostics_registry();

    do_or_return!(callbacks.early_callback(&matches, &descriptions, sopts.color));

    let (odir, ofile) = make_output(&matches);
    let (input, input_file_path) = match make_input(&matches.free) {
        Some((input, input_file_path)) => callbacks.some_input(input, input_file_path),
        None => match callbacks.no_input(&matches, &sopts, &odir, &ofile, &descriptions) {
            Some((input, input_file_path)) => (input, input_file_path),
            None => return
        }
    };

    let can_print_warnings = sopts.lint_opts
        .iter()
        .filter(|&&(ref key, _)| *key == "warnings")
        .map(|&(_, ref level)| *level != lint::Allow)
        .last()
        .unwrap_or(true);


    let codemap = codemap::CodeMap::with_file_loader(loader);
    let diagnostic_handler =
        diagnostic::Handler::new(sopts.color, Some(descriptions), can_print_warnings);
    let span_diagnostic_handler =
        diagnostic::SpanHandler::new(diagnostic_handler, codemap);

    let mut sess = session::build_session_(sopts, input_file_path, span_diagnostic_handler);
    rustc_lint::register_builtins(&mut sess.lint_store.borrow_mut(), Some(&sess));
    if sess.unstable_options() {
        sess.opts.show_span = matches.opt_str("show-span");
    }
    let cfg = config::build_configuration(&sess);

    do_or_return!(callbacks.late_callback(&matches, &sess, &input, &odir, &ofile));

    let plugins = sess.opts.debugging_opts.extra_plugins.clone();
    let control = callbacks.build_controller(&sess);
    driver::compile_input(sess, cfg, &input, &odir, &ofile, Some(plugins), control);
}
