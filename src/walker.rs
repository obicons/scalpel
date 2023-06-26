use crate::util::*;
use crate::{constraints, types};
use std::collections::HashSet;
use std::rc::Rc;

pub struct WalkResult {
    context: Vec<String>,
    pub constraints: Vec<Rc<constraints::Constraint>>,
    object_name: Option<String>,
}

pub fn extract_types(tu: &clang::TranslationUnit) -> WalkResult {
    let root_entity = tu.get_entity();
    let mut w = WalkResult::new();
    root_entity.visit_children(|n, p| w.analyze_entity(n, p));
    return w;
}

impl WalkResult {
    fn qualify_name(&self, name: &str) -> String {
        if self.context.len() > 0 {
            self.context.join("::") + "::" + name
        } else {
            String::from(name)
        }
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
            if let Some(comment) = node.get_parsed_comment() &&
               let text = get_comment_text(&comment) &&
               let Some((_, type_info)) = types::parse_type_comment(&text) {
                let object = Rc::new(constraints::Object::new(&qname));
                let constraint = constraints::type_to_constraint(&type_info, object);
                //println!("Added constraint {}", constraint);
                self.constraints.push(constraint);
            }
        }

        // Handle functions, namespaces, etc.
        if node.is_definition() && context_introducers.contains(&node.get_kind()) && let Some(name) = node.get_mangled_name() {
            self.context.push(name);
            node.visit_children(|n, p| self.analyze_entity(n, p));
            self.context.pop();
            return clang::EntityVisitResult::Continue;
        } else if node.is_unexposed() {
            // Just recurse until we get to an expose expression.
            node.visit_children(|n, p| self.analyze_entity(n, p));
            return clang::EntityVisitResult::Continue;
        } else {
            // println!("Visiting node {} of kind {:?}: is_def = {},  is_decl = {}",
            //          get_entity_spelling(&node).unwrap_or(String::from("Unknown spelling")),
            //          node.get_kind(), node.is_definition(), node.is_declaration());

            // Create constraints based on the RHS.
            if node.is_definition() && has_initialization(&node) {
                self.object_name = None;
                node.visit_children(|n, p| self.analyze_entity(n, p));

                // We want to enforce that the LHS minus the RHS = 0.
                if let None = self.object_name {
                    eprintln!("Warning: has a RHS with an unknown object name in {}.", spell_source_location(&node));
                    return clang::EntityVisitResult::Continue;
                }

                let lhs_object =
                    node.get_name().
                                    and_then(|name| Some(self.qualify_name(&name))).
                                    unwrap_or(format!("Unknown object in {}", spell_source_location(&node)));

                let lobj = Rc::new(constraints::Object::new(&lhs_object));
                let robj = Rc::new(constraints::Object::new(&self.object_name.as_ref().unwrap()));
                let constraint = constraints::assert_equal(lobj, robj);
                //println!("Constraint: {}", constraint);
                self.constraints.push(constraint);

                return clang::EntityVisitResult::Continue;
            } else if node.get_kind() == clang::EntityKind::DeclRefExpr {
                self.object_name = Some(
                    self.qualify_name(&node.get_name()
                                        .unwrap_or(String::from("Unknown object"))));
                return clang::EntityVisitResult::Break;
            } else if node.get_kind() == clang::EntityKind::FloatingLiteral {
                if let Some(clang::EvaluationResult::Float(f)) = node.evaluate() {
                    let object_name = format!("literal {} at {}", f, spell_source_location(&node));
                    self.object_name = Some(object_name);
                } else {
                    eprintln!("Warning: Could not evaluate node at {}", spell_source_location(&node));
                    self.object_name = None;
                }
                return clang::EntityVisitResult::Break;
            }

            return clang::EntityVisitResult::Recurse
        }
    }

    fn new() -> WalkResult {
        WalkResult {
            context: vec![],
            constraints: vec![],
            object_name: None,
        }
    }
}