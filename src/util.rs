use std::error::Error;
use std::process::Command;
use std::process::Stdio;

// See https://stackoverflow.com/questions/11946294/dump-include-paths-from-g/11946295#11946295.
pub fn get_system_include_paths(compiler: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let cmd = Command::new(compiler)
        .args(["-E", "-x", "c++", "-", "-v"])
        .stdin(Stdio::null())
        .output()?;

    let output_str = String::from_utf8(cmd.stderr)?;
    let mut in_sys_include = false;
    let mut include_path = Vec::<String>::new();
    for line in output_str.lines() {
        if line == "#include <...> search starts here:" {
            in_sys_include = true;
            continue;
        } else if line == "End of search list." {
            break;
        }

        if in_sys_include {
            include_path.push(line.trim().to_owned());
        }
    }

    Ok(include_path)
}

pub fn get_system_include_flags(compiler: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let search_paths = get_system_include_paths(compiler)?;
    Ok(search_paths.iter().map(|path| "-I".to_owned() + path.split_whitespace().collect::<Vec<&str>>()[0]).collect())
}

fn comment_child_to_string(child: clang::documentation::CommentChild) -> String {
    if let clang::documentation::CommentChild::Text(str) = child {
        str
    } else if let clang::documentation::CommentChild::Paragraph(babies) = child {
        babies.into_iter().map(comment_child_to_string).collect::<Vec<String>>().join(" ").trim().to_string()
    } else {
        String::from("")
    }
}

pub fn get_comment_text(comment: &clang::documentation::Comment) -> String {
    let children = comment.get_children();
    if children.is_empty() {
        String::from("")
    } else {
        children.into_iter().map(comment_child_to_string).collect::<Vec<String>>().join(" ").trim().to_string()
    }
}

pub fn get_initialization<'a>(definition: &'a clang::Entity) -> Option<clang::Entity::<'a>> {
    if !definition.is_definition() {
        return None;
    }

    let children = definition.get_children();
    if children.len() != 1 {
        return None;
    }

    return Some(children[0]);
}

pub fn has_initialization(definition: &clang::Entity) -> bool {
    return definition.is_definition() && definition.get_children().len() == 1;
}

pub fn get_entity_spelling(entity: &clang::Entity) -> Option<String> {
    entity.get_range().and_then(
        |range|
            Some(
                range
                    .tokenize()
                    .into_iter()
                    .map(|token| token.get_spelling())
                    .collect::<Vec<String>>()
                    .join(" ")
            )
    )
}

pub fn spell_source_location(entity: &clang::Entity) -> String {
    entity.get_location().
        and_then(
            |location| {
                let (file, line, col) = location.get_presumed_location();
                Some(format!("file {} on line {} column {}", file, line, col))
            }).unwrap_or(String::from("Unknown location"))
}

pub fn get_rhs<'a>(entity: &'a clang::Entity) -> Option<clang::Entity<'a>> {
    entity.get_child(0)
}

pub fn get_binary_operator(entity: &clang::Entity) -> Option<String> {
    let left_offset =
        entity
            .get_child(0)
            .iter()
            .map(|child| child.get_range().and_then(|r| Some(r.tokenize().len())))
            .fold(
                Some(0),
                |acc, elt| {
                    match (acc, elt) {
                        (Some(acc), Some(elt)) => Some(acc + elt),
                        _ => None,
                    }
                },
            );

    if left_offset.is_none() {
        return None;
    }

    let entity_tokens =
        entity
            .get_range()
            .and_then(|r| Some(r.tokenize()));
    if entity_tokens.is_none() {
        return None;
    }

    return Some(entity_tokens.unwrap()[left_offset.unwrap()].get_spelling());
}