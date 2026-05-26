use crate::config::{
    BuildMode, Config as beltConfig, Dependecies, FreeStanding, Layout, Link, Project,
};
use crate::leather::lexer::TokenType;
use crate::leather::parser::{Config as astConfig, Expression, Section};

pub struct Semantics {
    ast: astConfig,
}

pub type SemanticResult<T> = Result<T, String>;

fn get_string(expression: &Expression) -> SemanticResult<String> {
    match expression {
        Expression::Identifier(token) => Ok(token.literal.clone()),
        Expression::Keyword(token) => Ok(token.literal.clone()),
        Expression::StringLit(token) => Ok(token.literal.clone()),
        _ => Err("Expected a string value".to_string()),
    }
}

fn get_array_literal_strings(expression: &Expression) -> SemanticResult<Vec<String>> {
    let mut exprs: Vec<String> = Vec::new();
    match expression {
        Expression::Array(literal) => {
            for expr in &literal.expressions {
                exprs.push(get_string(&expr)?);
            }
        }
        _ => return Err("Expected an array literal".to_string()),
    };

    Ok(exprs)
}

impl Semantics {
    pub fn new(ast: astConfig) -> Self {
        Semantics { ast }
    }

    fn analyze_project(&self, section: &Section) -> SemanticResult<Project> {
        let mut name: Option<String> = None;
        let mut version: Option<String> = None;
        let mut entry: Option<String> = None;
        let mut mode = BuildMode::Executable; // default
        let mut target: Option<String> = None;
        let mut freestanding = FreeStanding::False; //default it to false
        let mut script: Option<String> = None;

        for variable in &section.contents {
            let key = match &variable.name {
                Expression::Keyword(token) => token.token_type.clone(),
                _ => return Err("Expected keyword as key".to_string()),
            };
            let value = get_string(&variable.value)?;

            match key {
                TokenType::Name => name = Some(value),
                TokenType::Version => version = Some(value),
                TokenType::Entry => entry = Some(value),
                TokenType::Mode => {
                    mode = match value.as_str() {
                        "executable" => BuildMode::Executable,
                        "static" => BuildMode::Static,
                        "obj" => BuildMode::Object,
                        _ => return Err(format!("Invalid mode: {}", value)),
                    }
                }
                TokenType::Freestanding => {
                    freestanding = match value.as_str() {
                        "true" => FreeStanding::True,
                        "false" => FreeStanding::False,
                        _ => return Err(format!("Invalid freestanding flag: {}", value)),
                    }
                }
                TokenType::Target => target = Some(value),
                TokenType::Script => script = Some(value),
                _ => return Err(format!("Unknown key in [project]")),
            }
        }

        Ok(Project {
            name: name.ok_or("error: missing required field 'name'")?,
            version: version.ok_or("error: missing required field 'version'")?,
            entry,
            mode,
            target,
            freestanding,
            script,
        })
    }

    fn analyze_layout(&self, section: &Section) -> SemanticResult<Layout> {
        let mut src: Option<String> = None;
        let mut stubs: Option<String> = None;
        let mut objs: Option<String> = None;
        let mut build: Option<String> = None;

        for variable in &section.contents {
            let key = match &variable.name {
                Expression::Identifier(_token) => get_string(&variable.name)?,
                _ => return Err("Expected identifier as the key in the layout section".to_string()),
            };
            let value = get_string(&variable.value)?;

            match key.as_str() {
                "src" => src = Some(value),
                "stubs" => stubs = Some(value),
                "objs" => objs = Some(value),
                "build" => build = Some(value),
                _ => return Err("Unknown key in [layout]".to_string()),
            }
        }

        Ok(Layout {
            src: src.unwrap_or_else(|| "src".to_string()),
            stubs: stubs.unwrap_or_else(|| "stubs".to_string()),
            objs: objs.unwrap_or_else(|| "objs".to_string()),
            build: build.unwrap_or_else(|| "build".to_string()),
        })
    }

    fn analyze_dependecies(&self, section: &Section) -> SemanticResult<Dependecies> {
        let mut dependecies: Vec<(String, String)> = Vec::new();

        for variable in &section.contents {
            let lib = get_string(&variable.name)?;
            let path = get_string(&variable.value)?;
            dependecies.push((lib, path));
        }
        Ok(Dependecies { deps: dependecies })
    }

    fn analyze_link(&self, section: &Section) -> SemanticResult<Link> {
        let mut links: Vec<(String, Vec<String>)> = Vec::new();
        for variable in &section.contents {
            let link = get_string(&variable.name)?;
            let list_of_links = get_array_literal_strings(&variable.value)?;
            links.push((link, list_of_links));
        }

        Ok(Link { links })
    }

    pub fn analyze(&self) -> SemanticResult<beltConfig> {
        let mut project: Option<Project> = None;
        let mut layout: Option<Layout> = None;
        let mut dependecies: Option<Dependecies> = None;
        let mut link: Option<Link> = None;

        for section in &self.ast.sections {
            match section.head.head.token_type {
                TokenType::Project => project = Some(self.analyze_project(section)?),
                TokenType::Layout => layout = Some(self.analyze_layout(section)?),
                TokenType::Dependecies => dependecies = Some(self.analyze_dependecies(section)?),
                TokenType::Link => link = Some(self.analyze_link(section)?),
                _ => return Err("Unknown section".to_string()),
            };
        }

        let project = project.ok_or("error: missing [project] section".to_string())?;
        let layout = layout.unwrap_or_else(|| Layout {
            src: "src".to_string(),
            stubs: "stubs".to_string(),
            objs: "objs".to_string(),
            build: "build".to_string(),
        });

        Ok(beltConfig {
            project,
            layout,
            dependecies,
            link,
        })
    }
}
