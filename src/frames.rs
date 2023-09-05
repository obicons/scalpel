use std::{collections::HashMap, ops::Deref, rc::Rc};

use z3::ast::Ast;

#[derive(Debug)]
pub enum InertialFrames {
    Local,
    Global,
    Unconstrained,
}

#[derive(Debug)]
pub enum TemporalFrames {
    Boot,
    Epoch,
    Unconstrained,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Conversions {
    NoOp,
    LocalToGlobal,
    GlobalToLocal,
    BootToEpoch,
    EpochToBoot,
}

pub const INERTIAL_FRAMES_COUNT: i64 = 3;

pub const ALL_INERTIAL_FRAMES: [InertialFrames; 2] =
    [InertialFrames::Local, InertialFrames::Global];

pub const ALL_TEMPORAL_FRAMES: [TemporalFrames; 2] = [TemporalFrames::Boot, TemporalFrames::Epoch];

impl std::convert::From<i64> for InertialFrames {
    fn from(value: i64) -> Self {
        match value {
            0 => InertialFrames::Local,
            1 => InertialFrames::Global,
            2 => InertialFrames::Unconstrained,
            _ => panic!("Unknown inertial frame {}", value),
        }
    }
}

impl std::convert::Into<i64> for &InertialFrames {
    fn into(self) -> i64 {
        match self {
            InertialFrames::Local => 0,
            InertialFrames::Global => 1,
            InertialFrames::Unconstrained => 2,
        }
    }
}

impl std::convert::From<&str> for InertialFrames {
    fn from(str: &str) -> InertialFrames {
        match str {
            "local" => InertialFrames::Local,
            "global" => InertialFrames::Global,
            "_" => InertialFrames::Unconstrained,
            _ => panic!("Unrecognized inertial frame: {}", str),
        }
    }
}

impl std::convert::From<i64> for TemporalFrames {
    fn from(value: i64) -> Self {
        match value {
            0 => TemporalFrames::Boot,
            1 => TemporalFrames::Epoch,
            2 => TemporalFrames::Unconstrained,
            _ => panic!("Unknown temporal frame {}", value),
        }
    }
}

impl std::convert::Into<i64> for &TemporalFrames {
    fn into(self) -> i64 {
        match self {
            TemporalFrames::Boot => 0,
            TemporalFrames::Epoch => 1,
            TemporalFrames::Unconstrained => 2,
        }
    }
}

impl std::convert::From<&str> for TemporalFrames {
    fn from(str: &str) -> TemporalFrames {
        match str {
            "boot" => TemporalFrames::Boot,
            "epoch" => TemporalFrames::Epoch,
            "_" => TemporalFrames::Unconstrained,
            _ => panic!("Unrecognized temporal frame: {}", str),
        }
    }
}

impl std::convert::Into<i64> for Conversions {
    fn into(self) -> i64 {
        match self {
            Conversions::NoOp => 0,
            Conversions::LocalToGlobal => 1,
            Conversions::GlobalToLocal => 2,
            Conversions::BootToEpoch => 3,
            Conversions::EpochToBoot => 4,
        }
    }
}

impl std::convert::From<i64> for Conversions {
    fn from(value: i64) -> Self {
        match value {
            0 => Conversions::NoOp,
            1 => Conversions::LocalToGlobal,
            2 => Conversions::GlobalToLocal,
            3 => Conversions::BootToEpoch,
            4 => Conversions::EpochToBoot,
            _ => panic!("Unrecognized conversion: {}", value),
        }
    }
}

pub fn parse_human_frame<'a>(str: &'a str) -> Option<(&'a str, InertialFrames, TemporalFrames)> {
    let frame_regex = regex::Regex::new(
        "frame\\(([a-zA-Z_]+[a-zA-Z0-9_]*)\\) = \\((local|global|_), (boot|epoch|_)\\)",
    )
    .unwrap();
    for caps in frame_regex.captures_iter(str) {
        let (_, [var_name, iframe, tframe]) = caps.extract();
        return Some((var_name, iframe.into(), tframe.into()));
    }

    return None;
}

fn frame_number(frame: (&InertialFrames, &TemporalFrames)) -> i64 {
    let iframe: i64 = frame.0.into();
    let tframe: i64 = frame.1.into();
    return iframe * INERTIAL_FRAMES_COUNT + tframe;
}

pub fn frame_from_number(frame_no: i64) -> (InertialFrames, TemporalFrames) {
    return (
        (frame_no / INERTIAL_FRAMES_COUNT).into(),
        (frame_no % INERTIAL_FRAMES_COUNT).into(),
    );
}

pub fn frame_assert<'a>(
    var_name: &str,
    frame: (&InertialFrames, &TemporalFrames),
    solver: &'a z3::Optimize<'a>,
) -> Rc<z3::ast::Int<'a>> {
    let context = solver.get_context();
    let z3_var = Rc::new(z3::ast::Int::new_const(
        context,
        get_frame_var_name(var_name),
    ));
    solver.assert(&z3_var._eq(&z3::ast::Int::from_i64(context, frame_number(frame) as i64)));
    return z3_var;
}

pub fn get_frame_var_name(object_name: &str) -> String {
    format!("{}_frame", object_name)
}

pub fn on_frame_assignment<'a, F>(
    lhs_name: &str,
    rhs_name: &str,
    solver: &'a z3::Optimize,
    object_name_to_frame_var: &mut HashMap<String, Rc<z3::ast::Int<'a>>>,
    frame_conversion_name_to_conversion: &mut HashMap<String, Rc<z3::ast::Int<'a>>>,
    frame_repair_consts: &mut Vec<Rc<z3::ast::Int<'a>>>,
    mut generate_variable_name: F,
) -> String
where
    F: FnMut() -> String,
{
    let lhs_z3_var = match object_name_to_frame_var.get(lhs_name) {
        Some(val) => val.clone(),
        None => {
            object_name_to_frame_var.insert(
                String::from(lhs_name),
                Rc::new(z3::ast::Int::new_const(
                    solver.get_context(),
                    get_frame_var_name(lhs_name),
                )),
            );
            object_name_to_frame_var.get(lhs_name).unwrap().clone()
        }
    };

    let rhs_z3_var = if let Some(val) = object_name_to_frame_var.get(rhs_name) {
        val.clone()
    } else {
        object_name_to_frame_var.insert(
            String::from(rhs_name),
            Rc::new(z3::ast::Int::new_const(
                solver.get_context(),
                get_frame_var_name(rhs_name),
            )),
        );
        object_name_to_frame_var.get(rhs_name).unwrap().clone()
    };

    let conversion_name = generate_variable_name();
    let conversion_const = Rc::new(z3::ast::Int::new_const(
        solver.get_context(),
        conversion_name.clone(),
    ));
    frame_conversion_name_to_conversion.insert(conversion_name.clone(), conversion_const.clone());

    let repair_const = Rc::new(z3::ast::Int::new_const(
        solver.get_context(),
        format!("{}_repair_const", conversion_name),
    ));
    frame_repair_consts.push(repair_const.clone());

    let disjunction = z3::ast::Bool::or(
        solver.get_context(),
        &[
            &z3::ast::Bool::and(
                solver.get_context(),
                &[
                    &lhs_z3_var._eq(&rhs_z3_var),
                    &conversion_const._eq(&z3::ast::Int::from_i64(
                        solver.get_context(),
                        Conversions::NoOp.into(),
                    )),
                    &repair_const._eq(&z3::ast::Int::from_i64(solver.get_context(), 0)),
                ],
            ),
            &z3::ast::Bool::and(
                solver.get_context(),
                &[
                    &conversion_const._eq(&z3::ast::Int::from_i64(
                        solver.get_context(),
                        Conversions::LocalToGlobal.into(),
                    )),
                    &lhs_z3_var._eq(&z3::ast::Int::from_i64(
                        solver.get_context(),
                        frame_number((&InertialFrames::Global, &TemporalFrames::Unconstrained)),
                    )),
                    &rhs_z3_var._eq(&z3::ast::Int::from_i64(
                        solver.get_context(),
                        frame_number((&InertialFrames::Local, &TemporalFrames::Unconstrained)),
                    )),
                    &repair_const._eq(&z3::ast::Int::from_i64(solver.get_context(), 1)),
                ],
            ),
            // TODO
        ],
    );
    solver.assert(&disjunction);

    return conversion_name;
}

pub fn add_minimization_constraint<'a>(
    solver: &'a z3::Optimize<'a>,
    repair_constants: &Vec<Rc<z3::ast::Int<'a>>>,
) {
    if repair_constants.is_empty() {
        return;
    }

    let mut sum = z3::ast::Int::from_i64(solver.get_context(), 0);
    for x in repair_constants {
        sum = sum + x.as_ref().deref();
    }

    solver.minimize(&sum);
}
