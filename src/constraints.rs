use std::{rc::Rc, collections::HashMap};
use crate::types::{self, NUM_BASE_UNITS, SIBaseUnits};

pub const COLUMNS_PER_OBJECT: usize = 1 + NUM_BASE_UNITS;

#[derive(Debug)]
pub enum Constraint {
    // Both of the constraints must be true.
    And(Rc<Constraint>, Rc<Constraint>),

    // The equation must be true.
    Equation(Rc<Equation>),
}

impl std::fmt::Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constraint::And(l, r) => write!(f, "({} /\\ {})", l, r),
            Constraint::Equation(eq) => write!(f, "{}", eq),
        }
    }
}

// Types define a natural constraint: Each members must be equal to what we expect.
pub fn type_to_constraint(t: &types::Type, obj: Rc<Object>) -> Rc<Constraint> {
    let mut eqs: Vec<Rc<Equation>> = vec![
        Rc::new(Equation {
            term: Rc::new(Term::Object(obj.clone(), Selector::ScalarPrefix)),
            value: t.scalar_prefix,
        })
    ];
    for dimension in 0..types::NUM_BASE_UNITS {
        eqs.push(Rc::new(Equation{
            term: Rc::new(Term::Object(
                            obj.clone(),
                            Selector::BaseUnit(types::SIBaseUnits::from(dimension))
                        )),
            value: t.si_units[dimension] as f64,
        }));
    }

    // The first and second equations form the first and.
    let and1 = Rc::new(Constraint::And(
        Rc::new(Constraint::Equation(eqs[0].clone())),
        Rc::new(Constraint::Equation(eqs[1].clone())),
    ));
    eqs.split_off(2).into_iter().fold(
        and1,
        |constraint, eq|
            Rc::new(Constraint::And(constraint, Rc::new(Constraint::Equation(eq))))
    )
}

pub fn assert_equal(obj1: Rc<Object>, obj2: Rc<Object>) -> Rc<Constraint> {
    let mut eqs: Vec<Rc<Equation>> = vec![
        Rc::new(Equation {
                    term: Rc::new(Term::Sub(
                        Rc::new(Term::Object(obj1.clone(), Selector::ScalarPrefix)),
                        Rc::new(Term::Object(obj2.clone(), Selector::ScalarPrefix)),
                    )),
                    value: 0.0,
                })
    ];
    for dimension in 0..types::NUM_BASE_UNITS {
        eqs.push(Rc::new(Equation{
            term: Rc::new(Term::Sub(
                Rc::new(Term::Object(obj1.clone(), Selector::BaseUnit(types::SIBaseUnits::from(dimension)))),
                Rc::new(Term::Object(obj2.clone(), Selector::BaseUnit(types::SIBaseUnits::from(dimension)))),
            )),
            value: 0.0,
        }));
    }

    let and1 = Rc::new(Constraint::And(
        Rc::new(Constraint::Equation(eqs[0].clone())),
        Rc::new(Constraint::Equation(eqs[1].clone())),
    ));
    eqs.split_off(2).into_iter().fold(
        and1,
        |constraint, eq|
            Rc::new(Constraint::And(constraint, Rc::new(Constraint::Equation(eq))))
    )
}

// Asserts that rhs can be repaired into lhs.
pub fn assert_repairable(lhs: Rc<Object>, rhs: Rc<Object>, repair_term: Rc<Object>) -> Rc<Constraint> {
    let mut eqs: Vec<Rc<Equation>> = vec![
        Rc::new(Equation {
                    term: Rc::new(Term::Sub(
                        Rc::new(Term::Object(lhs.clone(), Selector::ScalarPrefix)),
                        Rc::new(Term::Add(
                            Rc::new(Term::Object(rhs.clone(), Selector::ScalarPrefix)),
                            Rc::new(Term::Object(repair_term.clone(), Selector::ScalarPrefix)),
                        )),
                    )),
                    value: 0.0,
                })
    ];
    for dimension in 0..types::NUM_BASE_UNITS {
        eqs.push(Rc::new(Equation{
            term: Rc::new(Term::Sub(
                Rc::new(Term::Object(lhs.clone(), Selector::BaseUnit(types::SIBaseUnits::from(dimension)))),
                Rc::new(Term::Object(rhs.clone(), Selector::BaseUnit(types::SIBaseUnits::from(dimension)))),
            )),
            value: 0.0,
        }));
    }

    let and1 = Rc::new(Constraint::And(
        Rc::new(Constraint::Equation(eqs[0].clone())),
        Rc::new(Constraint::Equation(eqs[1].clone())),
    ));
    eqs.split_off(2).into_iter().fold(
        and1,
        |constraint, eq|
            Rc::new(Constraint::And(constraint, Rc::new(Constraint::Equation(eq))))
    )
}

// Creates a new type by multiplying lhs and rhs.
pub fn create_multiplicative_type(result_type: Rc<Object>, lhs: Rc<Object>, rhs: Rc<Object>) -> Rc<Constraint> {
    let mut eqs: Vec<Rc<Equation>> = vec![
        Rc::new(Equation {
                    term: Rc::new(Term::Sub(
                        Rc::new(Term::Object(result_type.clone(), Selector::ScalarPrefix)),
                        Rc::new(Term::Add(
                            Rc::new(Term::Object(lhs.clone(), Selector::ScalarPrefix)),
                            Rc::new(Term::Object(rhs.clone(), Selector::ScalarPrefix)),
                        )),
                    )),
                    value: 0.0,
                })
    ];
    for dimension in 0..types::NUM_BASE_UNITS {
        eqs.push(Rc::new(Equation{
            term: Rc::new(Term::Sub(
                Rc::new(Term::Object(result_type.clone(), Selector::BaseUnit(types::SIBaseUnits::from(dimension)))),
                Rc::new(Term::Add(
                    Rc::new(Term::Object(lhs.clone(), Selector::BaseUnit(types::SIBaseUnits::from(dimension)))),
                    Rc::new(Term::Object(rhs.clone(), Selector::BaseUnit(types::SIBaseUnits::from(dimension)))),
                )),
            )),
            value: 0.0,
        }));
    }

    let and1 = Rc::new(Constraint::And(
        Rc::new(Constraint::Equation(eqs[0].clone())),
        Rc::new(Constraint::Equation(eqs[1].clone())),
    ));
    eqs.split_off(2).into_iter().fold(
        and1,
        |constraint, eq|
            Rc::new(Constraint::And(constraint, Rc::new(Constraint::Equation(eq))))
    )
}

// Creates a new type by dividing lhs and rhs.
pub fn create_division_type(result_type: Rc<Object>, lhs: Rc<Object>, rhs: Rc<Object>) -> Rc<Constraint> {
    let mut eqs: Vec<Rc<Equation>> = vec![
        Rc::new(Equation {
                    term: Rc::new(Term::Sub(
                        Rc::new(Term::Object(result_type.clone(), Selector::ScalarPrefix)),
                        Rc::new(Term::Add(
                            Rc::new(Term::Object(lhs.clone(), Selector::ScalarPrefix)),
                            Rc::new(Term::Object(rhs.clone(), Selector::ScalarPrefix)),
                        )),
                    )),
                    value: 0.0,
                })
    ];
    for dimension in 0..types::NUM_BASE_UNITS {
        eqs.push(Rc::new(Equation{
            term: Rc::new(Term::Add(
                Rc::new(Term::Object(result_type.clone(), Selector::BaseUnit(types::SIBaseUnits::from(dimension)))),
                Rc::new(Term::Sub(
                    Rc::new(Term::Object(rhs.clone(), Selector::BaseUnit(types::SIBaseUnits::from(dimension)))),
                    Rc::new(Term::Object(lhs.clone(), Selector::BaseUnit(types::SIBaseUnits::from(dimension)))),
                )),
            )),
            value: 0.0,
        }));
    }

    let and1 = Rc::new(Constraint::And(
        Rc::new(Constraint::Equation(eqs[0].clone())),
        Rc::new(Constraint::Equation(eqs[1].clone())),
    ));
    eqs.split_off(2).into_iter().fold(
        and1,
        |constraint, eq|
            Rc::new(Constraint::And(constraint, Rc::new(Constraint::Equation(eq))))
    )
}

#[derive(Clone, Debug)]
pub struct Equation {
    term: Rc<Term>,
    value: f64,
}

impl std::fmt::Display for Equation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.term, self.value)
    }
}

#[derive(Debug)]
enum Selector {
    BaseUnit(types::SIBaseUnits),
    ScalarPrefix,
}

impl std::fmt::Display for Selector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
enum Term {
    Add(Rc<Term>, Rc<Term>),
    Sub(Rc<Term>, Rc<Term>),
    Object(Rc<Object>, Selector),
}

impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Add(l, r) => write!(f, "Add({}, {})", l, r),
            Term::Sub(l, r) => write!(f, "Sub({}, {})", l, r),
            Term::Object(obj, s) => write!(f, "{}.{}", obj, s),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Object {
    // A human-readable description of the object.
    pub label: String,
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

impl Object {
    pub fn new(label: &str) -> Object {
        Object{label: String::from(label)}
    }
}

fn add_term_to_map(term: &Rc<Term>, object_to_column: &mut HashMap<Object, i32>) {
    match &**term {
        Term::Add(t1, t2) => {
            add_term_to_map(t1, object_to_column);
            add_term_to_map(t2, object_to_column);
        },
        Term::Sub(t1, t2) => {
            add_term_to_map(t1, object_to_column);
            add_term_to_map(t2, object_to_column);
        },
        Term::Object(o, _) => {
            if let None = object_to_column.get(o) {
                let max_id = object_to_column.values().reduce(
                    |max, col| {
                        if col > max {
                            col
                        } else {
                            max
                        }
                    }
                );

                object_to_column.insert(Object::new(&o.label), *max_id.unwrap_or(&-1) + 1);
            }
        },
    }
}

fn add_constraint_to_map(constraint: &Constraint, map: &mut HashMap<Object, i32>) {
    match constraint {
        Constraint::And(c1, c2) => {
            add_constraint_to_map(c1, map);
            add_constraint_to_map(c2, map);
        },
        Constraint::Equation(eq) => {
            add_term_to_map(&eq.term, map);
        },
    }
}

fn constraint_objects_to_column_numbers(constraints: &Vec<Rc<Constraint>>) -> HashMap<Object, i32> {
    let mut result = HashMap::<Object, i32>::new();
    for constraint in constraints {
        add_constraint_to_map(constraint, &mut result);
    }
    return result;
}

fn add_term_to_row(term: &Rc<Term>,
                   object_to_column_offset: &HashMap<Object, i32>,
                   row: &mut Vec<f64>) {
    match &**term {
        Term::Add(t1, t2) => {
            add_term_to_row(t1, object_to_column_offset, row);
            add_term_to_row(t2, object_to_column_offset, row);
        },
        Term::Sub(t1, t2) => {
            add_term_to_row(t1, object_to_column_offset, row);

            let mut tmp = vec![0.0; row.len()];
            add_term_to_row(t2, object_to_column_offset, &mut tmp);

            for i in 0..tmp.len() {
                row[i] -= tmp[i];
            }
        },
        Term::Object(object, selector) => {
            let idx = (*object_to_column_offset.get(object).unwrap() as usize) * COLUMNS_PER_OBJECT;
            match selector {
                Selector::BaseUnit(bu) => {
                    let offset: usize = bu.into_usize() + 1;
                    row[idx + offset] += 1.0;
                },
                Selector::ScalarPrefix => {
                    row[idx] += 1.0;
                },
            }
        },
    }
}

fn add_constraint_to_system(constraint: &Constraint,
                            object_to_column_offset: &HashMap<Object, i32>,
                            system: &mut Vec<Vec<f64>>) {
    match constraint {
        Constraint::And(c1, c2) => {
            add_constraint_to_system(c1, object_to_column_offset, system);
            add_constraint_to_system(c2, object_to_column_offset, system);
        },
        Constraint::Equation(eq) => {
            let mut eq_row = Vec::<f64>::new();
            let total_columns = COLUMNS_PER_OBJECT * object_to_column_offset.keys().len() + 1;
            for _ in 0..total_columns {
                eq_row.push(0.0);
            }

            let last_idx = eq_row.len() - 1;
            eq_row[last_idx] = eq.value;
            add_term_to_row(&eq.term, object_to_column_offset, &mut eq_row);
            system.push(eq_row);
        },
    }
}

pub fn constraint_system_to_linear_system(constraints: &Vec<Rc<Constraint>>, output_csv: bool) -> (Vec<Vec<f64>>, HashMap<Object, i32>) {
    // Step 1: Map each object appearing in a constraint to a unique column number.
    let object_name_to_column = constraint_objects_to_column_numbers(constraints);

    // Step 2: Build the result vector.

    // Each row has columns_per_object columns.
    let mut system = Vec::<Vec<f64>>::new();
    for constraint in constraints {
        add_constraint_to_system(constraint,
                                 &object_name_to_column,
                                 &mut system);
    }

    if output_csv {
        let mut header: HashMap<i32, String> = HashMap::new();
        for (object, column) in &object_name_to_column {
            header.insert(*column, object.label.clone());
        }

        let mut sep = "";
        for i in 0..header.len() {
            print!("{}{}.sm", sep, header.get(&(i as i32)).unwrap());
            sep = ",";

            for j in 0..NUM_BASE_UNITS {
                print!("{}{}.{:?}", sep, header.get(&(i as i32)).unwrap(), SIBaseUnits::from(j));
            }
        }
        println!("{}equals", sep);

        for row in &system {
            let mut sep = "";
            for val in row {
                print!("{}{}", sep, val);
                sep = ",";
            }
            println!();
        }
    }

    return (system, object_name_to_column);
}