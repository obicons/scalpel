#![feature(let_chains, variant_count)]
use clap::Parser;
use nalgebra::{DMatrix, DVector};


use crate::constraints::constraint_system_to_linear_system;

mod constraints;
mod types;
mod util;
mod walker;

#[derive(Parser)]
struct Cli {
    // The path to the directory containing compile_commands.json.
    #[arg(short, long)]
    compile_commands_directory: String,
}

fn main() {
    let args = Cli::parse();
    println!("Compile commands directory: {}", args.compile_commands_directory);

    validate_command_line_args(&args);

    let db_result = clang::CompilationDatabase::from_directory(args.compile_commands_directory);
    if let Err(_) = db_result {
        std::process::exit(1);
    }

    let clang_inst_result = clang::Clang::new();
    if let Err(err) = clang_inst_result {
        eprintln!("libclang error: {}", err);
        std::process::exit(1);
    }
    let clang_inst = clang_inst_result.unwrap();
    let index = clang::Index::new(&clang_inst, true, true);

    let sys_include_flags_result = util::get_system_include_flags("clang++");
    if let Err(err) = sys_include_flags_result {
        eprintln!("Could not get system include paths: {}", err);
        std::process::exit(1);
    }
    let sys_include_flags = sys_include_flags_result.unwrap();

    let db = db_result.unwrap();
    for cmd in db.get_all_compile_commands().get_commands() {
        if let Err(err ) = std::env::set_current_dir(cmd.get_directory()) {
            eprintln!("{}: {}", cmd.get_directory().display(), err);
            std::process::exit(1);
        }

        let mut args = cmd.get_arguments();
        args.append(&mut sys_include_flags.clone());
        args = args.into_iter().filter(
            |name| name != &cmd.get_filename().file_name().unwrap().to_string_lossy().to_string()
        ).collect();

        let mut parser = index.parser(cmd.get_filename());
        let parser = parser.arguments(&args);
        let tu_result = parser.parse();
        if let Err(err) = tu_result {
            eprintln!("{}: {}", cmd.get_filename().display(), err);
            std::process::exit(1);
        }

        let walk_result = walker::extract_types(&tu_result.unwrap());
        //println!("{:?}", walk_result.constraints);
        // for row in &walk_result.constraints {
        //     println!("{:?}", row);
        // }
        // constraint_system_to_linear_system(&vec![walk_result.constraints[0].clone()]);
        // println!("====");
        // println!("{:?}", walk_result.constraints[1]);
        // constraint_system_to_linear_system(&vec![walk_result.constraints[1].clone()]);
        let system = constraint_system_to_linear_system(&walk_result.constraints);

        let mut A = Vec::<Vec<f64>>::new();
        for row in &system {
            let v = &row[0..row.len() - 1].to_vec();
            A.push(v.clone());
        }

        let mut b = Vec::<f64>::new();
        for row in &system {
            b.push(*row.last().unwrap());
        }

        let A = DMatrix::from_fn(A.len(), A[0].len(), |i, j| A[i][j]);
        let b = DVector::from_iterator(b.len(), b);
        let results = lstsq::lstsq(&A, &b, 0.000000001).unwrap();

        if results.residuals.abs() > 1. {
            println!("Typechecking error.");
        } else {
            println!("Typechecking succeeded.");
        }
    }
}

fn validate_command_line_args(args: &Cli) {
    if let Err(err) = std::fs::metadata(&args.compile_commands_directory) {
        eprintln!("{}: {}", args.compile_commands_directory, err);
        std::process::exit(1);
    }
}