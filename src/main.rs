//extern crate rustpython_parser;
#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate rustpython_parser;
extern crate rustpython_vm;

use clap::{App, Arg};
use rustpython_parser::parser;
use rustpython_vm::obj::objstr;
use rustpython_vm::print_exception;
use rustpython_vm::pyobject::{PyObjectRef, PyResult};
use rustpython_vm::VirtualMachine;
use rustpython_vm::{compile, import};
use std::io;
use std::io::prelude::*;
use std::path::Path;

fn main() {
    env_logger::init();
    let matches = App::new("RustPython")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Rust implementation of the Python language")
        .arg(Arg::with_name("script").required(false).index(1))
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Give the verbosity"),
        )
        .arg(
            Arg::with_name("c")
                .short("c")
                .takes_value(true)
                .help("run the given string as a program"),
        )
        .arg(
            Arg::with_name("m")
                .short("m")
                .takes_value(true)
                .help("run library module as script"),
        )
        .get_matches();

    // Construct vm:
    let mut vm = VirtualMachine::new();

    // Figure out if a -c option was given:
    let result = if let Some(command) = matches.value_of("c") {
        run_command(&mut vm, command.to_string())
    } else if let Some(module) = matches.value_of("m") {
        run_module(&mut vm, module)
    } else {
        // Figure out if a script was passed:
        match matches.value_of("script") {
            None => run_shell(&mut vm),
            Some(filename) => run_script(&mut vm, &filename.to_string()),
        }
    };

    // See if any exception leaked out:
    handle_exception(&mut vm, result);
}

fn _run_string(vm: &mut VirtualMachine, source: &str, source_path: Option<String>) -> PyResult {
    let code_obj = compile::compile(vm, &source.to_string(), compile::Mode::Exec, source_path)?;
    debug!("Code object: {:?}", code_obj.borrow());
    let builtins = vm.get_builtin_scope();
    let vars = vm.context().new_scope(Some(builtins)); // Keep track of local variables
    vm.run_code_obj(code_obj, vars)
}

fn handle_exception(vm: &mut VirtualMachine, result: PyResult) {
    match result {
        Ok(_value) => {}
        Err(err) => {
            print_exception(vm, &err);
        }
    }
}

fn run_command(vm: &mut VirtualMachine, mut source: String) -> PyResult {
    debug!("Running command {}", source);

    // This works around https://github.com/RustPython/RustPython/issues/17
    source.push_str("\n");
    _run_string(vm, &source, None)
}

fn run_module(vm: &mut VirtualMachine, module: &str) -> PyResult {
    debug!("Running module {}", module);
    import::import_module(vm, module)
}

fn run_script(vm: &mut VirtualMachine, script_file: &str) -> PyResult {
    debug!("Running file {}", script_file);
    // Parse an ast from it:
    let filepath = Path::new(script_file);
    match parser::read_file(filepath) {
        Ok(source) => _run_string(vm, &source, Some(filepath.to_str().unwrap().to_string())),
        Err(msg) => {
            error!("Parsing went horribly wrong: {}", msg);
            std::process::exit(1);
        }
    }
}

fn shell_exec(vm: &mut VirtualMachine, source: &str, scope: PyObjectRef) -> bool {
    match compile::compile(vm, &source.to_string(), compile::Mode::Single, None) {
        Ok(code) => {
            match vm.run_code_obj(code, scope) {
                Ok(_value) => {
                    // Printed already.
                }
                Err(err) => {
                    print_exception(vm, &err);
                }
            }
        }
        Err(err) => {
            // Enum rather than special string here.
            let msg = match vm.get_attribute(err.clone(), "msg") {
                Ok(value) => objstr::get_value(&value),
                Err(_) => panic!("Expected msg attribute on exception object!"),
            };
            if msg == "Unexpected end of input." {
                return false;
            } else {
                print_exception(vm, &err);
            }
        }
    };
    true
}

fn run_shell(vm: &mut VirtualMachine) -> PyResult {
    println!(
        "Welcome to the magnificent Rust Python {} interpreter",
        crate_version!()
    );
    let builtins = vm.get_builtin_scope();
    let vars = vm.context().new_scope(Some(builtins)); // Keep track of local variables

    // Read a single line:
    let mut input = String::new();
    loop {
        // TODO: modules dont support getattr / setattr yet
        //let prompt = match vm.get_attribute(vm.sys_module.clone(), "ps1") {
        //        Ok(value) => objstr::get_value(&value),
        //        Err(_) => ">>>>> ".to_string(),
        //};
        print!(">>>>> ");

        io::stdout().flush().expect("Could not flush stdout");
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                break;
            }
            Ok(_) => {
                debug!("You entered {:?}", input);
                if shell_exec(vm, &input, vars.clone()) {
                    // Line was complete.
                    input = String::new();
                } else {
                    loop {
                        // until an empty line is pressed AND the code is complete
                        //let prompt = match vm.get_attribute(vm.sys_module.clone(), "ps2") {
                        //        Ok(value) => objstr::get_value(&value),
                        //        Err(_) => "..... ".to_string(),
                        //};
                        print!("..... ");
                        io::stdout().flush().expect("Could not flush stdout");
                        let mut line = String::new();
                        match io::stdin().read_line(&mut line) {
                            Ok(_) => {
                                line = line
                                    .trim_right_matches(|c| c == '\r' || c == '\n')
                                    .to_string();
                                if line.len() == 0 {
                                    if shell_exec(vm, &input, vars.clone()) {
                                        input = String::new();
                                        break;
                                    }
                                } else {
                                    input.push_str(&line);
                                    input.push_str("\n");
                                }
                            }
                            Err(msg) => panic!("Error: {:?}", msg),
                        }
                    }
                }
            }
            Err(msg) => panic!("Error: {:?}", msg),
        };
    }

    Ok(vm.get_none())
}
