use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub struct Syntax<'a> {
    open_tag: &'a str,
    close_tag: &'a str,
    optional_operator: &'a str,
}
impl Default for Syntax<'_> {
    fn default() -> Self {
        Syntax {
            open_tag: "<!--",
            close_tag: "-->",
            optional_operator: "optional",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ParametersLineToken<'a> {
    Raw { value: &'a str },
    Parameter { is_optional: bool, name: &'a str },
}
#[derive(Debug, PartialEq, Eq)]
enum Line<'a> {
    Raw {
        value: &'a str,
    },
    Parameters {
        tokens: Vec<ParametersLineToken<'a>>,
    },
    Reference {
        left: Option<&'a str>,
        is_optional: bool,
        name: &'a str,
        right: Option<&'a str>,
    },
}
#[derive(Debug, PartialEq, Eq)]
pub struct Template<'a> {
    lines: Vec<Line<'a>>,
}

#[derive(Debug)]
pub struct Parser {
    parameter_regex: Regex,
    reference_line_regex: Regex,
}
impl Parser {
    pub fn from_syntax(
        syntax: &Syntax,
        parameter_operator: &str,
        reference_operator: &str,
    ) -> Result<Self, String> {
        Ok(Parser {
                parameter_regex: Regex::new(
                    format!(r"(?P<left>.+?)?{} *(?P<optional>\({}\))?\({}\)(?P<name>\w+) *{}",
                        regex::escape(syntax.open_tag),
                        regex::escape(syntax.optional_operator),
                        regex::escape(parameter_operator),
                        regex::escape(syntax.close_tag)
                    ).as_str(),
                ).map_err(|error| format!("Can not parse parameter regex: {error}"))?,
                reference_line_regex: Regex::new(
                    format!(
                        r"^(?P<left>.+)?{} *(?P<optional>\({}\))?\({}\)(?P<name>\w+) *{}(?P<right>.+)?$",
                        regex::escape(syntax.open_tag),
                        regex::escape(syntax.optional_operator),
                        regex::escape(reference_operator),
                        regex::escape(syntax.close_tag)
                    ).as_str(),
                ).map_err(|error| format!("Can not parse reference line regex: {error}"))?,
            })
    }
    pub fn parse<'a>(&'a self, text: &'a str) -> Result<Template<'a>, String> {
        let mut parsed_lines: Vec<Line> = Vec::new();
        for line in text.lines() {
            parsed_lines.push({
                let parameters_captures: Vec<_> =
                    self.parameter_regex.captures_iter(line).collect();
                if parameters_captures.len() > 0 {
                    Line::Parameters {
                        tokens: {
                            let mut result: Vec<ParametersLineToken> = Vec::new();
                            for capture in &parameters_captures {
                                if let Some(left) = capture.name("left") {
                                    result.push(ParametersLineToken::Raw {
                                        value: left.as_str(),
                                    });
                                }
                                result.push(ParametersLineToken::Parameter {
                                    is_optional: capture.name("optional").is_some(),
                                    name: capture
                                        .name("name")
                                        .ok_or(format!(
                                            "Can not get parameter name from line {line}"
                                        ))?
                                        .as_str(),
                                })
                            }
                            let right = &line[parameters_captures
                                .last()
                                .ok_or(format!(
                                    "Can not get last parameter captures from line {line}"
                                ))?
                                .get_match()
                                .end()..];
                            result.push(ParametersLineToken::Raw { value: right });
                            result
                        },
                    }
                } else if let Some(captures) = self.reference_line_regex.captures(line) {
                    Line::Reference {
                        left: if let Some(left_match) = captures.name("left") {
                            Some(left_match.as_str())
                        } else {
                            None
                        },
                        is_optional: captures.name("optional").is_some(),
                        name: captures
                            .name("name")
                            .ok_or(format!("Can not get parameter name from line {line}"))?
                            .as_str(),
                        right: if let Some(left_match) = captures.name("right") {
                            Some(left_match.as_str())
                        } else {
                            None
                        },
                    }
                } else {
                    Line::Raw { value: line }
                }
            });
        }
        Ok(Template {
            lines: parsed_lines,
        })
    }
}

pub enum TemplateParametersValue<'a> {
    Parameters(TemplateParameters<'a>),
    ValuesVec(Vec<String>),
    ParametersVec(Vec<TemplateParameters<'a>>),
    Value(String),
}
pub type TemplateParameters<'a> = HashMap<&'a str, TemplateParametersValue<'a>>;
pub type Templates<'a> = HashMap<&'a str, Template<'a>>;

#[macro_export]
macro_rules! tp_value {
    ($val:expr) => {
        TemplateParametersValue::Value($val)
    };
}

#[macro_export]
macro_rules! tp_values {
    ($($val:expr),*) => {
        TemplateParametersValue::ValuesVec(vec![$($val.to_string()),*])
    };
}

#[macro_export]
macro_rules! tp_params {
    ($($key:expr => $value:expr),*) => {{
        let mut params = HashMap::new();
        $(params.insert($key, $value);)*
        TemplateParametersValue::Parameters(params)
    }};
    () => {
        TemplateParametersValue::Parameters(HashMap::new())
    };
}

#[macro_export]
macro_rules! tp_params_vec {
    ($($params:expr),*) => {
        TemplateParametersValue::ParametersVec(vec![$($params),*])
    };
}

#[macro_export]
macro_rules! params {
    ($($key:expr => $value:expr),*) => {{
        let mut map = HashMap::new();
        $(map.insert($key, $value);)*
        map
    }};
    () => {
        HashMap::new()
    };
}

impl Template<'_> {
    pub fn render_internal(
        &self,
        parameters: &TemplateParameters,
        templates: &Templates,
        external_left: &Option<String>,
        external_right: &Option<String>,
        result: &mut String,
    ) -> Result<(), String> {
        for line in self.lines.iter() {
            match line {
                Line::Raw { value } => {
                    if let Some(external_left) = external_left {
                        result.push_str(external_left.as_str());
                    }
                    result.push_str(value);
                    if let Some(external_right) = external_right {
                        result.push_str(external_right.as_str());
                    }
                    result.push('\n');
                }
                Line::Parameters { tokens } => {
                    let all_tokens_are_optional = tokens.iter().all(|token| match token {
                        ParametersLineToken::Raw { value: _ } => true,
                        ParametersLineToken::Parameter {
                            is_optional,
                            name: _,
                        } => *is_optional,
                    });
                    let mut value_index = 0_i64;
                    loop {
                        let mut new_value_index = value_index + 1;
                        if let Some(external_left) = external_left {
                            result.push_str(external_left.as_str());
                        }
                        for token in tokens {
                            match token {
                                ParametersLineToken::Raw { value } => result.push_str(value),
                                ParametersLineToken::Parameter { is_optional, name } => {
                                    if let Some(value_variant) = parameters.get(*name) {
                                        match value_variant {
                                            TemplateParametersValue::Value(value) => {
                                                if value_index == 0 {
                                                    result.push_str(value);
                                                }
                                                if !is_optional || all_tokens_are_optional {
                                                    new_value_index = -1;
                                                }
                                            }
                                            TemplateParametersValue::ValuesVec(values) => {
                                                if values.len() == 0 {
                                                    return Err(format!(
                                                        "Expected non-empty Vec of values for parameter \"{name}\""
                                                    ));
                                                } else if values.len()
                                                    == (value_index + 1).try_into().map_err(|error| format!("Can not convert value index plus one: {error}"))?
                                                {
                                                    new_value_index = -1;
                                                }
                                                let value = &values[value_index as usize];
                                                result.push_str(value);
                                            }
                                            _ => {
                                                return Err(format!(
                                                    "Expected value or non-empty Vec of values for parameter \"{name}\""
                                                ));
                                            }
                                        }
                                    } else {
                                        if !is_optional {
                                            return Err(format!(
                                                "Expected key for parameter \"{name}\""
                                            ));
                                        }
                                        new_value_index = -1;
                                    }
                                }
                            }
                        }
                        if let Some(external_right) = external_right {
                            result.push_str(external_right.as_str());
                        }
                        result.push('\n');
                        value_index = new_value_index;
                        if value_index == -1 {
                            break;
                        }
                    }
                }
                Line::Reference {
                    left,
                    is_optional,
                    name,
                    right,
                } => {
                    if let Some(value_variant) = parameters.get(*name) {
                        match value_variant {
                            TemplateParametersValue::Parameters(subtemplate_parameters) => {
                                if let Some(subtemplate) = templates.get(*name) {
                                    subtemplate.render_internal(
                                        subtemplate_parameters,
                                        templates,
                                        &Some(
                                            external_left.clone().unwrap_or(String::new())
                                                + left.unwrap_or(""),
                                        ),
                                        &Some(
                                            external_right.clone().unwrap_or(String::new())
                                                + right.unwrap_or(""),
                                        ),
                                        result,
                                    )?;
                                } else if !*is_optional {
                                    return Err(format!(
                                        "No template parameters provided for template reference \"{name}\""
                                    ));
                                }
                            }
                            TemplateParametersValue::ParametersVec(subtemplate_parameters_vec) => {
                                for subtemplate_parameters in subtemplate_parameters_vec {
                                    if let Some(subtemplate) = templates.get(*name) {
                                        subtemplate.render_internal(
                                            subtemplate_parameters,
                                            templates,
                                            &Some(
                                                external_left.clone().unwrap_or(String::new())
                                                    + left.unwrap_or(""),
                                            ),
                                            &Some(
                                                external_right.clone().unwrap_or(String::new())
                                                    + right.unwrap_or(""),
                                            ),
                                            result,
                                        )?;
                                    } else if !*is_optional {
                                        return Err(format!(
                                            "No template parameters provided for template reference \"{name}\""
                                        ));
                                    }
                                }
                            }
                            _ => {
                                return Err(format!(
                                    "Expected template parameters of template parameters Vec for template reference \"{name}\""
                                ));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
    pub fn render(
        &self,
        parameters: &TemplateParameters,
        templates: &Templates,
    ) -> Result<String, String> {
        let mut result = String::new();
        self.render_internal(parameters, templates, &None, &None, &mut result)?;
        Ok(result)
    }
}

#[test]
fn test_parse() {
    let parser = Parser::from_syntax(&Syntax::default(), "param", "ref").unwrap();
    let template_text = "<tr>\n    <td><!-- (param)cell1 --></td><td><!-- (optional)(param)cell2 --></td>\n</tr>\n<!-- (ref)Ref1 -->";
    let template = parser.parse(template_text).unwrap();

    assert_eq!(template.lines.len(), 4);

    assert_eq!(template.lines[0], Line::Raw { value: "<tr>" });
    assert_eq!(
        template.lines[1],
        Line::Parameters {
            tokens: Vec::from([
                ParametersLineToken::Raw { value: "    <td>" },
                ParametersLineToken::Parameter {
                    is_optional: false,
                    name: "cell1"
                },
                ParametersLineToken::Raw { value: "</td><td>" },
                ParametersLineToken::Parameter {
                    is_optional: true,
                    name: "cell2"
                },
                ParametersLineToken::Raw { value: "</td>" }
            ])
        }
    );
    assert_eq!(template.lines[2], Line::Raw { value: "</tr>" });
    assert_eq!(
        template.lines[3],
        Line::Reference {
            left: None,
            is_optional: false,
            name: "Ref1",
            right: None
        }
    );
}
macro_rules! test {
    ($name:ident, $template_text:expr, $parameters:expr, $templates:expr, $correct_result:expr) => {
        #[test]
        fn $name() {
            let parser = Parser::from_syntax(&Syntax::default(), "param", "ref").unwrap();
            let templates: HashMap<&str, &str> = HashMap::from($templates);
            let mut parsed_templates = Templates::from([]);
            for (template_name, template_text) in templates {
                let parsed = parser.parse(template_text).unwrap();
                parsed_templates.insert(template_name, parsed);
            }
            assert_eq!(
                parser
                    .parse($template_text)
                    .unwrap()
                    .render($parameters, &parsed_templates,)
                    .unwrap(),
                $correct_result
            );
        }
    };
}
test!(
    render_param,
    "one <!-- (param)p --> two",
    &params! {"p" => tp_value!("lalala".to_string())},
    [],
    "one lalala two\n"
);
test!(
    render_multivalued_param,
    "one <!-- (param)p --> two",
    &params! {"p" => tp_values! ["v1", "v2"]},
    [],
    "one v1 two\none v2 two\n"
);
test!(
    render_multiple_params,
    "one <!-- (param)p1 --> <!-- (param)p2 --> two",
    &params! {"p1" => tp_value!("v1".to_string()), "p2" => tp_value!("v2".to_string())},
    [],
    "one v1 v2 two\n"
);
test!(
    render_optional_param,
    "one <!-- (optional)(param)p --> two",
    &TemplateParameters::from([]),
    [],
    "one  two\n"
);
test!(
    render_optional_param_while_there_is_also_param_with_more_than_one_value,
    "left <!-- (param)p1 --> middle <!-- (optional)(param)p2 --> right\nplain text",
    &params! {"p1" => tp_values!("lalala", "lululu"), "p2" => tp_value!("lololo".to_string())},
    [],
    "left lalala middle lololo right\nleft lululu middle  right\nplain text\n"
);
test!(
    render_ref,
    "one <!-- (ref)r --> two",
    &params! {"r" => tp_params! {"p" => tp_value!("v".to_string())}},
    [("r", "three")],
    "one three two\n"
);
test!(
    render_2x2_html_table,
    "<table>\n    <!-- (ref)Row -->\n</table>",
    &params! {"Row" => tp_params_vec!(params! {"cell" => tp_values!("1.1", "2.1")}, params! {"cell" => tp_values!("1.2", "2.2")})},
    [("Row", "<tr>\n    <td><!-- (param)cell --></td>\n</tr>")],
    "<table>\n    <tr>\n        <td>1.1</td>\n        <td>2.1</td>\n    </tr>\n    <tr>\n        <td>1.2</td>\n        <td>2.2</td>\n    </tr>\n</table>\n"
);
test!(
    render_ref_with_param,
    "one <!-- (ref)r --> two",
    &params! {"r" => tp_params! {"p" => tp_value!("three".to_string())}},
    [("r", "<!-- (param)p -->")],
    "one three two\n"
);
test!(
    render_multivalued_ref_with_param,
    "one <!-- (ref)r --> two",
    &params! {"r" => tp_params_vec!(params! {"p" => tp_value!("three".to_string())}, params! {"p" => tp_value!("four".to_string())})},
    [("r", "<!-- (param)p -->")],
    "one three two\none four two\n"
);
