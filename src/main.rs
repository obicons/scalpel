#![feature(let_chains, variant_count)]
use std::collections::HashMap;

use clap::Parser;
use constraints::COLUMNS_PER_OBJECT;
use lstsq::Lstsq;
use nalgebra::{DMatrix, DVector, ComplexField};


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

    // Whether to output the CSV of the system of linear equations.
    #[arg(short, long)]
    show_equations: bool,
}

fn main() {
    let cli_args = Cli::parse();

    validate_command_line_args(&cli_args);

    let db_result = clang::CompilationDatabase::from_directory(cli_args.compile_commands_directory);
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
        let (system, object_name_to_colums)
             = constraint_system_to_linear_system(
                &walk_result.constraints,
                cli_args.show_equations,
        );

        let mut a = Vec::<Vec<f64>>::new();
        for row in &system {
            let v = &row[0..row.len() - 1].to_vec();
            a.push(v.clone());
        }

        let mut b = Vec::<f64>::new();
        for row in &system {
            b.push(*row.last().unwrap());
        }

        let mut tmp_terms: Vec<constraints::Object> =
            walk_result.tmp_terms_to_repair_contexts.keys()
                .map(|o| o.clone())
                .collect();
        let result = do_sparsest_repair(
            &a,
            &b,
            &object_name_to_colums,
            &walk_result.tmp_terms_to_repair_contexts,
            &mut tmp_terms,
        );

        if !result {
            eprintln!("Program not repairable.")
        }

        // let a = DMatrix::from_fn(a.len(), a[0].len(), |i, j| a[i][j]);
        // let b = DVector::from_iterator(b.len(), b);
        // let results = lstsq::lstsq(&a, &b, 0.000000001).unwrap();

        // if results.residuals.abs() > 0.01 {
        //     println!("Not repairable.");
        // } else {
        //     println!("Repairable.");
        //     generate_repair(results, &object_name_to_colums, &walk_result.tmp_terms_to_repair_contexts);
        // }
    }
}

fn validate_command_line_args(args: &Cli) {
    if let Err(err) = std::fs::metadata(&args.compile_commands_directory) {
        eprintln!("{}: {}", args.compile_commands_directory, err);
        std::process::exit(1);
    }
}

fn do_sparsest_repair(system: &Vec<Vec<f64>>, result: &Vec<f64>, object_to_column: &HashMap<constraints::Object, i32>,
                      terms_to_contexts: &HashMap<constraints::Object, walker::RepairContext>,
                      temp_terms: &mut Vec<constraints::Object>) -> bool {
    if temp_terms.is_empty() {
        let a = DMatrix::from_fn(system.len(), system[0].len(), |i, j| system[i][j]);
        let b = DVector::from_iterator(result.len(), result.clone());
        let results = lstsq::lstsq(&a, &b, 0.001).unwrap();
        if results.residuals.abs() <= 0.01 {
            generate_repair(results, object_to_column, terms_to_contexts);
            return true;
        } else {
            println!("Repair failed with error: {}", results.residuals.abs());
            return false;
        }
    }

    let candidate_zero_term = temp_terms.pop().unwrap();
    let mut new_row = vec![0.0; system[0].len()];
    new_row[*object_to_column.get(&candidate_zero_term).unwrap() as usize * COLUMNS_PER_OBJECT] = 1.0;

    let mut new_system = system.clone();
    new_system.push(new_row);

    let mut new_result = result.clone();
    new_result.push(0.0);

    if do_sparsest_repair(&new_system, &new_result, object_to_column, terms_to_contexts, temp_terms) {
        return true;
    } else {
        return do_sparsest_repair(system, result, object_to_column, terms_to_contexts, temp_terms);
    }
}

fn generate_repair(x: Lstsq<f64, nalgebra::Dyn>, object_to_column: &HashMap<constraints::Object, i32>, terms_to_contexts: &HashMap<constraints::Object, walker::RepairContext>) {
    let solution = x.solution;
    // for i in 0..solution.shape().0 {
    //     println!("{}, {}", i, solution[i]);
    // }

    for (obj, context) in terms_to_contexts {
        match object_to_column.get(obj) {
            Some(column) => {
                let real_column = constraints::COLUMNS_PER_OBJECT * (*column as usize);
                //println!("{} in {} at {} -> {}", obj.label, context.original_expression, context.source_location, solution[real_column]);
                if  solution[real_column].abs() > 0.00000001 {
                    println!("{}: (pow(10.0, {:.3}) * ({})) ({})", context.source_location, solution[real_column] * -1.0, context.original_expression, obj.label);
                }
            },
            None => eprintln!("WARNING: Unable to find column for constant {}", obj.label)
        }
    }
}