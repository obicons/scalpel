use crate::constraints::assert_literal;
use crate::util::*;
use crate::{constraints, frames, types};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub struct RepairContext {
    pub source_location: String,
    pub original_expression: String,
}

pub struct WalkContext<'a> {
    context: Vec<String>,
    pub constraints: Vec<Rc<constraints::Constraint>>,
    object_name: Option<String>,
    fresh_count: i32,
    pub tmp_terms_to_repair_contexts: HashMap<constraints::Object, RepairContext>,

    // Frame stuff.
    z3_solver: &'a z3::Optimize<'a>,
    pub object_name_to_frame_var: HashMap<String, Rc<z3::ast::Int<'a>>>,
    pub frame_conversion_name_to_conversion: HashMap<String, Rc<z3::ast::Int<'a>>>,
    pub frame_conversion_name_to_repair_context: HashMap<String, RepairContext>,
    pub frame_repair_consts: Vec<Rc<z3::ast::Int<'a>>>,
}

pub fn extract_types<'a>(
    tu: &clang::TranslationUnit,
    solver: &'a z3::Optimize<'a>,
) -> WalkContext<'a> {
    let root_entity = tu.get_entity();
    let mut w = WalkContext::new(&solver);
    root_entity.visit_children(|n, p| w.analyze_entity(n, p));
    return w;
}

impl<'a> WalkContext<'a> {
    fn qualify_name(&self, name: &str) -> String {
        if self.context.len() > 0 {
            self.context.join("::") + "::" + name
        } else {
            String::from(name)
        }
    }

    fn fresh_variable(&mut self) -> String {
        let varname = "T".to_owned() + &self.fresh_count.to_string();
        self.fresh_count += 1;
        return varname;
    }

    fn analyze_entity(
        &mut self,
        node: clang::Entity,
        _parent: clang::Entity,
    ) -> clang::EntityVisitResult {
        let context_introducers = HashSet::from([
            clang::EntityKind::ClassDecl,
            clang::EntityKind::ClassTemplate,
            clang::EntityKind::FunctionDecl,
            clang::EntityKind::Method,
            clang::EntityKind::FunctionTemplate,
            clang::EntityKind::Namespace,
        ]);

        if let Some(qname) = node.get_name().and_then(|n| Some(self.qualify_name(&n))) {
            if let Some(comment) = node.get_parsed_comment() {
                let text = get_comment_text(&comment);
                if let Some((_, iframe, tframe)) = frames::parse_human_frame(&text) {
                    let z3_var = frames::frame_assert(&qname, (&iframe, &tframe), self.z3_solver);
                    self.object_name_to_frame_var
                        .insert(String::from(&qname), z3_var);
                    println!("I see {} {:?} {:?}", qname, iframe, tframe);
                }
                if let Some((_, type_info)) = types::parse_type_comment(&text) {
                    let object = Rc::new(constraints::Object::new(&qname));
                    let constraint = constraints::type_to_constraint(&type_info, object);
                    //println!("For object {} added constraint {}", qname, constraint);
                    self.constraints.push(constraint);
                }
            }
        }

        // Handle functions, namespaces, etc.
        if node.is_definition()
            && context_introducers.contains(&node.get_kind())
            && node.get_mangled_name().is_some()
        {
            let name = node.get_mangled_name().unwrap();
            self.context.push(name);
            node.visit_children(|n, p| self.analyze_entity(n, p));
            self.context.pop();
            return clang::EntityVisitResult::Continue;
        } else if node.is_unexposed() {
            // Just recurse until we get to an expose expression.
            node.visit_children(|n, p| self.analyze_entity(n, p));
            return clang::EntityVisitResult::Continue;
        } else {
            // println!(
            //     "Visiting node {} of kind {:?}: is_def = {},  is_decl = {}",
            //     get_entity_spelling(&node).unwrap_or(String::from("Unknown spelling")),
            //     node.get_kind(),
            //     node.is_definition(),
            //     node.is_declaration()
            // );

            // Create constraints based on the RHS.
            if node.is_definition() && has_initialization(&node) {
                self.object_name = None;
                node.visit_children(|n, p| self.analyze_entity(n, p));

                // We want to enforce that the LHS minus the RHS = 0.
                if let None = self.object_name {
                    eprintln!(
                        "Warning: has a RHS with an unknown object name in {}.",
                        spell_source_location(&node)
                    );
                    return clang::EntityVisitResult::Continue;
                }

                let lhs_object = node
                    .get_name()
                    .and_then(|name| Some(self.qualify_name(&name)))
                    .unwrap_or(format!(
                        "Unknown object in {}",
                        spell_source_location(&node)
                    ));

                println!(
                    "Visiting lhs {} and rhs {} in assignment.",
                    lhs_object,
                    self.object_name.as_ref().unwrap()
                );

                let mut count = self.fresh_count;
                let naming_fn = || {
                    let name = format!("T{}", count);
                    count += 1;
                    return name;
                };

                let frame_repair_name = frames::on_frame_assignment(
                    &lhs_object,
                    self.object_name.as_ref().unwrap(),
                    self.z3_solver,
                    &mut self.object_name_to_frame_var,
                    &mut self.frame_conversion_name_to_conversion,
                    &mut self.frame_repair_consts,
                    naming_fn,
                );
                self.fresh_count = count;
                self.frame_conversion_name_to_repair_context.insert(
                    frame_repair_name,
                    RepairContext {
                        source_location: spell_source_location(&node),
                        original_expression: get_rhs(&node)
                            .and_then(|entity| get_entity_spelling(&entity))
                            .unwrap_or(String::from("Unknown spelling")),
                    },
                );

                let lobj = Rc::new(constraints::Object::new(&lhs_object));
                let repair_term = self.fresh_variable();
                let repair_constant = Rc::new(constraints::Object::new(&repair_term));
                let robj = Rc::new(constraints::Object::new(
                    &self.object_name.as_ref().unwrap(),
                ));

                let original_expression = get_rhs(&node)
                    .and_then(|entity| get_entity_spelling(&entity))
                    .unwrap_or(String::from("Unknown spelling"));
                let source_location = spell_source_location(&node);
                self.tmp_terms_to_repair_contexts.insert(
                    constraints::Object::new(&repair_term),
                    RepairContext {
                        source_location,
                        original_expression,
                    },
                );

                let constraint = constraints::assert_repairable(lobj, robj, repair_constant);
                self.constraints.push(constraint);

                return clang::EntityVisitResult::Continue;
            } else if node.get_kind() == clang::EntityKind::DeclRefExpr {
                self.object_name = Some(
                    self.qualify_name(&node.get_name().unwrap_or(String::from("Unknown object"))),
                );
                return clang::EntityVisitResult::Continue;
            } else if node.get_kind() == clang::EntityKind::FloatingLiteral {
                if let Some(clang::EvaluationResult::Float(f)) = node.evaluate() {
                    let object_name = format!("literal {} at {}", f, spell_source_location(&node));
                    self.constraints.push(assert_literal(f, &object_name));
                    self.object_name = Some(object_name);
                } else {
                    eprintln!(
                        "Warning: Could not evaluate node at {}",
                        spell_source_location(&node)
                    );
                    self.object_name = None;
                }
                return clang::EntityVisitResult::Continue;
            } else if node.get_kind() == clang::EntityKind::BinaryOperator {
                println!(
                    "binop: lhs = {}, rhs = {}",
                    get_entity_spelling(&node.get_child(0).unwrap())
                        .unwrap_or(String::from("unknown")),
                    get_entity_spelling(&node.get_child(1).unwrap())
                        .unwrap_or(String::from("unknown"))
                );
                let mut first_parent: Option<clang::Entity> = None;
                let mut lhs_object: Option<String> = None;
                node.visit_children(|n, p| {
                    println!("Calling: {:?}", get_entity_spelling(&n));
                    if let None = first_parent {
                        first_parent = Some(p);
                    } else if Some(p) == first_parent && lhs_object.is_none() {
                        // We have the LHS object.
                        // Keep going to find the RHS.
                        lhs_object = self.object_name.clone();
                    }

                    return self.analyze_entity(n, p);
                });

                if lhs_object.is_none() {
                    return clang::EntityVisitResult::Continue;
                }
                let lhs_object = lhs_object.unwrap();

                let operator = get_binary_operator(&node);
                if operator.is_none() {
                    return clang::EntityVisitResult::Continue;
                }

                let operator = operator.unwrap();
                if operator == "="
                    || operator == "+"
                    || operator == "-"
                    || operator == "<"
                    || operator == "<="
                    || operator == ">"
                    || operator == ">="
                {
                    let lobj = Rc::new(constraints::Object::new(&lhs_object));
                    let repair_term = self.fresh_variable();
                    let repair_constant = Rc::new(constraints::Object::new(&repair_term));
                    let robj = Rc::new(constraints::Object::new(
                        &self.object_name.as_ref().unwrap(),
                    ));

                    let original_expression = node
                        .get_child(1)
                        .and_then(|entity| get_entity_spelling(&entity))
                        .unwrap_or(String::from("Unknown spelling"));
                    let source_location = spell_source_location(&node.get_child(1).unwrap());
                    self.tmp_terms_to_repair_contexts.insert(
                        constraints::Object::new(&repair_term),
                        RepairContext {
                            source_location,
                            original_expression,
                        },
                    );

                    let constraint = constraints::assert_repairable(lobj, robj, repair_constant);
                    self.constraints.push(constraint);

                    self.object_name = Some(lhs_object);
                    return clang::EntityVisitResult::Continue;
                } else if operator == "*" {
                    let lobj = Rc::new(constraints::Object::new(&lhs_object));
                    let type_term = self.fresh_variable();
                    let type_constant = Rc::new(constraints::Object::new(&type_term));
                    let robj = Rc::new(constraints::Object::new(
                        &self.object_name.as_ref().unwrap(),
                    ));
                    let constraint =
                        constraints::create_multiplicative_type(type_constant, lobj, robj);
                    self.constraints.push(constraint);
                    self.object_name = Some(type_term);
                    return clang::EntityVisitResult::Continue;
                } else if operator == "/" {
                    let lobj = Rc::new(constraints::Object::new(&lhs_object));
                    let type_term = self.fresh_variable();
                    let type_constant = Rc::new(constraints::Object::new(&type_term));
                    let robj = Rc::new(constraints::Object::new(
                        &self.object_name.as_ref().unwrap(),
                    ));
                    let constraint = constraints::create_division_type(type_constant, lobj, robj);
                    self.constraints.push(constraint);
                    self.object_name = Some(type_term);
                    return clang::EntityVisitResult::Continue;
                }
            }

            return clang::EntityVisitResult::Recurse;
        }
    }

    fn new(solver: &'a z3::Optimize<'a>) -> WalkContext<'a> {
        WalkContext {
            context: vec![],
            constraints: vec![],
            object_name: None,
            fresh_count: 0,
            tmp_terms_to_repair_contexts: HashMap::new(),
            z3_solver: solver,
            object_name_to_frame_var: HashMap::new(),
            frame_conversion_name_to_conversion: HashMap::new(),
            frame_conversion_name_to_repair_context: HashMap::new(),
            frame_repair_consts: Vec::new(),
        }
    }
}
