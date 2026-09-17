use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, BindingPatternKind, Declaration, Expression, FormalParameters, Function,
    ObjectPattern, Statement, TSSignature, TSType, TSTypeName, VariableDeclaration,
};
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{GetSpan, SourceType, Span};
use oxc_transformer::{JsxOptions, TransformOptions, Transformer};

use crate::catalog::PropMetadata;

pub fn transpile_component_source(path: &Path, source: &str) -> Result<String, String> {
    let source_type = SourceType::from_path(path)
        .map_err(|_| format!("unsupported component source: {}", path.display()))?;
    if !source_type.is_typescript() {
        return Ok(source.to_string());
    }

    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if !parsed.errors.is_empty() {
        return Err(parse_error(path, &parsed.errors));
    }
    let mut program = parsed.program;
    let semantic = SemanticBuilder::new()
        .with_excess_capacity(2.0)
        .build(&program);
    if !semantic.errors.is_empty() {
        let summary = semantic
            .errors
            .iter()
            .take(3)
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ");
        return Err(format!("could not analyze {}: {summary}", path.display()));
    }
    let options = TransformOptions {
        jsx: JsxOptions::disable(),
        ..TransformOptions::default()
    };
    let transformed = Transformer::new(&allocator, path, &options)
        .build_with_scoping(semantic.semantic.into_scoping(), &mut program);
    if !transformed.errors.is_empty() {
        let summary = transformed
            .errors
            .iter()
            .take(3)
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ");
        return Err(format!("could not transform {}: {summary}", path.display()));
    }
    Ok(Codegen::new().build(&program).code)
}

pub fn extract_component_props(
    path: &Path,
    source: &str,
    component_name: &str,
) -> Result<Vec<PropMetadata>, String> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path)
        .map_err(|_| format!("unsupported component source: {}", path.display()))?;
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if !parsed.errors.is_empty() {
        return Err(parse_error(path, &parsed.errors));
    }

    let Some(pattern) = find_component_pattern(&parsed.program.body, component_name) else {
        return Ok(Vec::new());
    };
    let Some(object) = object_pattern(pattern) else {
        return Ok(Vec::new());
    };

    let runtime_props = runtime_props(object, &parsed.program.comments, source);
    let Some(type_name) = referenced_type_name(pattern) else {
        return Ok(runtime_props);
    };
    let Some(members) = find_type_members(&parsed.program.body, type_name) else {
        return Ok(runtime_props);
    };
    let declared_props = declared_props(members, &parsed.program.comments, source);
    Ok(merge_declared_and_runtime(declared_props, runtime_props))
}

fn parse_error<T: ToString>(path: &Path, errors: &[T]) -> String {
    let summary = errors
        .iter()
        .take(3)
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ");
    format!("could not parse {}: {summary}", path.display())
}

fn runtime_props(
    object: &ObjectPattern<'_>,
    comments: &[oxc_ast::ast::Comment],
    source: &str,
) -> Vec<PropMetadata> {
    object
        .properties
        .iter()
        .filter_map(|property| {
            let name = property.key.static_name()?.into_owned();
            let default_expression = assignment_expression(&property.value);
            let type_name = type_annotation(&property.value, source)
                .or_else(|| {
                    default_expression
                        .map(|expression| inferred_expression_type(expression).to_string())
                })
                .unwrap_or_else(|| "Unknown".to_string());
            let default =
                default_expression.map(|expression| source_text(expression.span(), source));
            let description =
                leading_description(comments, property.key.span(), property.span, source);
            Some(PropMetadata {
                name,
                description,
                type_name,
                default,
            })
        })
        .collect()
}

fn referenced_type_name<'a>(pattern: &'a BindingPattern<'a>) -> Option<&'a str> {
    let annotation = pattern
        .type_annotation
        .as_ref()
        .or_else(|| match &pattern.kind {
            BindingPatternKind::AssignmentPattern(assignment) => {
                assignment.left.type_annotation.as_ref()
            }
            _ => None,
        })?;
    simple_type_reference(&annotation.type_annotation)
}

fn simple_type_reference<'a>(type_annotation: &'a TSType<'a>) -> Option<&'a str> {
    match type_annotation {
        TSType::TSTypeReference(reference) => match &reference.type_name {
            TSTypeName::IdentifierReference(identifier) if reference.type_arguments.is_none() => {
                Some(identifier.name.as_str())
            }
            _ => None,
        },
        TSType::TSParenthesizedType(parenthesized) => {
            simple_type_reference(&parenthesized.type_annotation)
        }
        _ => None,
    }
}

fn find_type_members<'a>(
    statements: &'a [Statement<'a>],
    type_name: &str,
) -> Option<&'a [TSSignature<'a>]> {
    for statement in statements {
        let members = match statement {
            Statement::TSTypeAliasDeclaration(alias) => alias_members(alias, type_name),
            Statement::TSInterfaceDeclaration(interface) => interface_members(interface, type_name),
            Statement::ExportNamedDeclaration(export) => export
                .declaration
                .as_ref()
                .and_then(|declaration| declaration_members(declaration, type_name)),
            _ => None,
        };
        if members.is_some() {
            return members;
        }
    }
    None
}

fn declaration_members<'a>(
    declaration: &'a Declaration<'a>,
    type_name: &str,
) -> Option<&'a [TSSignature<'a>]> {
    match declaration {
        Declaration::TSTypeAliasDeclaration(alias) => alias_members(alias, type_name),
        Declaration::TSInterfaceDeclaration(interface) => interface_members(interface, type_name),
        _ => None,
    }
}

fn alias_members<'a>(
    alias: &'a oxc_ast::ast::TSTypeAliasDeclaration<'a>,
    type_name: &str,
) -> Option<&'a [TSSignature<'a>]> {
    if alias.id.name != type_name {
        return None;
    }
    match &alias.type_annotation {
        TSType::TSTypeLiteral(literal) => Some(&literal.members),
        _ => None,
    }
}

fn interface_members<'a>(
    interface: &'a oxc_ast::ast::TSInterfaceDeclaration<'a>,
    type_name: &str,
) -> Option<&'a [TSSignature<'a>]> {
    if interface.id.name == type_name {
        Some(&interface.body.body)
    } else {
        None
    }
}

fn declared_props(
    members: &[TSSignature<'_>],
    comments: &[oxc_ast::ast::Comment],
    source: &str,
) -> Vec<PropMetadata> {
    members
        .iter()
        .filter_map(|member| {
            let TSSignature::TSPropertySignature(property) = member else {
                return None;
            };
            if property.computed {
                return None;
            }
            let name = property.key.static_name()?.into_owned();
            let type_name = property
                .type_annotation
                .as_ref()
                .map(|annotation| annotation_source(annotation.span, source))
                .unwrap_or_else(|| "Unknown".to_string());
            let description =
                leading_description(comments, property.key.span(), property.span, source);
            Some(PropMetadata {
                name,
                description,
                type_name,
                default: None,
            })
        })
        .collect()
}

fn merge_declared_and_runtime(
    mut declared: Vec<PropMetadata>,
    runtime: Vec<PropMetadata>,
) -> Vec<PropMetadata> {
    for prop in &mut declared {
        let Some(runtime_prop) = runtime.iter().find(|candidate| candidate.name == prop.name)
        else {
            continue;
        };
        if !runtime_prop.description.is_empty() {
            prop.description.clone_from(&runtime_prop.description);
        }
        if prop.type_name == "Unknown" {
            prop.type_name.clone_from(&runtime_prop.type_name);
        }
        prop.default.clone_from(&runtime_prop.default);
    }
    for prop in runtime {
        if !declared.iter().any(|candidate| candidate.name == prop.name) {
            declared.push(prop);
        }
    }
    declared
}

fn find_component_pattern<'a>(
    statements: &'a [Statement<'a>],
    component_name: &str,
) -> Option<&'a BindingPattern<'a>> {
    for statement in statements {
        let found = match statement {
            Statement::FunctionDeclaration(function) => function_pattern(function, component_name),
            Statement::VariableDeclaration(declaration) => {
                variable_pattern(declaration, component_name)
            }
            Statement::ExportNamedDeclaration(export) => export
                .declaration
                .as_ref()
                .and_then(|declaration| declaration_pattern(declaration, component_name)),
            Statement::ExportDefaultDeclaration(export) => match &export.declaration {
                oxc_ast::ast::ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
                    function_pattern(function, component_name)
                }
                _ => None,
            },
            _ => None,
        };
        if found.is_some() {
            return found;
        }
    }
    None
}

fn declaration_pattern<'a>(
    declaration: &'a Declaration<'a>,
    component_name: &str,
) -> Option<&'a BindingPattern<'a>> {
    match declaration {
        Declaration::FunctionDeclaration(function) => function_pattern(function, component_name),
        Declaration::VariableDeclaration(declaration) => {
            variable_pattern(declaration, component_name)
        }
        _ => None,
    }
}

fn function_pattern<'a>(
    function: &'a Function<'a>,
    component_name: &str,
) -> Option<&'a BindingPattern<'a>> {
    if function.id.as_ref()?.name != component_name {
        return None;
    }
    first_pattern(&function.params)
}

fn variable_pattern<'a>(
    declaration: &'a VariableDeclaration<'a>,
    component_name: &str,
) -> Option<&'a BindingPattern<'a>> {
    for declarator in &declaration.declarations {
        if declarator.id.get_identifier_name().as_deref() != Some(component_name) {
            continue;
        }
        let pattern = match declarator.init.as_ref()? {
            Expression::ArrowFunctionExpression(function) => first_pattern(&function.params),
            Expression::FunctionExpression(function) => first_pattern(&function.params),
            _ => None,
        };
        if pattern.is_some() {
            return pattern;
        }
    }
    None
}

fn first_pattern<'a>(parameters: &'a FormalParameters<'a>) -> Option<&'a BindingPattern<'a>> {
    parameters.items.first().map(|parameter| &parameter.pattern)
}

fn object_pattern<'a>(pattern: &'a BindingPattern<'a>) -> Option<&'a ObjectPattern<'a>> {
    match &pattern.kind {
        BindingPatternKind::ObjectPattern(object) => Some(object),
        BindingPatternKind::AssignmentPattern(assignment) => object_pattern(&assignment.left),
        _ => None,
    }
}

fn assignment_expression<'a>(pattern: &'a BindingPattern<'a>) -> Option<&'a Expression<'a>> {
    match &pattern.kind {
        BindingPatternKind::AssignmentPattern(assignment) => Some(&assignment.right),
        _ => None,
    }
}

fn type_annotation(pattern: &BindingPattern<'_>, source: &str) -> Option<String> {
    let annotation = pattern
        .type_annotation
        .as_ref()
        .or_else(|| match &pattern.kind {
            BindingPatternKind::AssignmentPattern(assignment) => {
                assignment.left.type_annotation.as_ref()
            }
            _ => None,
        })?;
    Some(annotation_source(annotation.span, source))
}

fn annotation_source(span: Span, source: &str) -> String {
    source_text(span, source)
        .trim_start_matches(':')
        .trim()
        .to_string()
}

fn inferred_expression_type(expression: &Expression<'_>) -> &'static str {
    match expression {
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => "String",
        Expression::BooleanLiteral(_) => "Boolean",
        Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::UnaryExpression(_) => "Number",
        Expression::ArrayExpression(_) => "Array",
        Expression::ObjectExpression(_) => "Object",
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => "Function",
        Expression::NullLiteral(_) => "Null",
        _ => "Unknown",
    }
}

fn leading_description(
    comments: &[oxc_ast::ast::Comment],
    key_span: Span,
    property_span: Span,
    source: &str,
) -> String {
    let mut matching = comments
        .iter()
        .filter(|comment| {
            comment.is_leading()
                && (comment.attached_to == key_span.start
                    || comment.attached_to == property_span.start)
        })
        .collect::<Vec<_>>();
    matching.sort_by_key(|comment| comment.span.start);
    matching
        .into_iter()
        .map(|comment| clean_comment(source_text(comment.content_span(), source)))
        .filter(|comment| !comment.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn clean_comment(comment: String) -> String {
    comment
        .lines()
        .map(|line| line.trim().trim_start_matches('*').trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn source_text(span: Span, source: &str) -> String {
    source
        .get(span.start as usize..span.end as usize)
        .unwrap_or_default()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_destructured_props_from_function_and_arrow_components() {
        let source = r#"
export function Button({
  /** Button text. */
  label,
  // Visual treatment.
  variant = "primary",
  /** Prevents interaction. */
  disabled = false,
  count = 2,
}) { return <button>{label}</button>; }

export const Link = ({ href = "/", children }) => <a href={href}>{children}</a>;
"#;
        let button = extract_component_props(Path::new("button.jsx"), source, "Button").unwrap();
        assert_eq!(button.len(), 4);
        assert_eq!(button[0].name, "label");
        assert_eq!(button[0].description, "Button text.");
        assert_eq!(button[0].type_name, "Unknown");
        assert_eq!(button[0].default, None);
        assert_eq!(button[1].description, "Visual treatment.");
        assert_eq!(button[1].type_name, "String");
        assert_eq!(button[1].default.as_deref(), Some("\"primary\""));
        assert_eq!(button[2].type_name, "Boolean");
        assert_eq!(button[3].type_name, "Number");

        let link = extract_component_props(Path::new("button.jsx"), source, "Link").unwrap();
        assert_eq!(link.len(), 2);
        assert_eq!(link[0].name, "href");
        assert_eq!(link[0].default.as_deref(), Some("\"/\""));
    }

    #[test]
    fn resolves_type_alias_and_interface_prop_metadata() {
        let source = r#"
export type CardProps = {
  /** Main content shown in the card. */
  label: string;
  /** Optional supporting count. */
  count?: number;
  tone: "neutral" | "accent";
};

export const Card = ({ label, count = 2, tone = "neutral" }: CardProps) => (
  <article data-tone={tone}>{label} {count}</article>
);

interface ButtonProps {
  // Accessible button label.
  label: string;
  /** Prevents interaction. */
  disabled?: boolean;
  icon?: React.ReactNode;
}

export function Button({
  label,
  /** Runtime comments override declaration comments. */
  disabled = false,
}: ButtonProps) {
  return <button disabled={disabled}>{label}</button>;
}
"#;
        let card = extract_component_props(Path::new("cards.tsx"), source, "Card").unwrap();
        assert_eq!(card.len(), 3);
        assert_eq!(card[0].name, "label");
        assert_eq!(card[0].description, "Main content shown in the card.");
        assert_eq!(card[0].type_name, "string");
        assert_eq!(card[0].default, None);
        assert_eq!(card[1].name, "count");
        assert_eq!(card[1].type_name, "number");
        assert_eq!(card[1].default.as_deref(), Some("2"));
        assert_eq!(card[2].type_name, "\"neutral\" | \"accent\"");
        assert_eq!(card[2].default.as_deref(), Some("\"neutral\""));

        let button = extract_component_props(Path::new("cards.tsx"), source, "Button").unwrap();
        assert_eq!(button.len(), 3, "declared-only props stay visible");
        assert_eq!(button[0].description, "Accessible button label.");
        assert_eq!(
            button[1].description,
            "Runtime comments override declaration comments."
        );
        assert_eq!(button[1].type_name, "boolean");
        assert_eq!(button[1].default.as_deref(), Some("false"));
        assert_eq!(button[2].name, "icon");
        assert_eq!(button[2].type_name, "React.ReactNode");
        assert_eq!(button[2].default, None);

        let runtime = transpile_component_source(Path::new("cards.tsx"), source).unwrap();
        assert!(!runtime.contains("type CardProps"), "{runtime}");
        assert!(!runtime.contains("interface ButtonProps"), "{runtime}");
        assert!(!runtime.contains(": CardProps"), "{runtime}");
        assert!(runtime.contains("export const Card"), "{runtime}");
        assert!(runtime.contains("<article"), "{runtime}");
    }
}
