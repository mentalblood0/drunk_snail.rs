pub mod drunk_snail {
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
    impl Default for Parser {
        fn default() -> Self {
            Parser::from_syntax(&Syntax::default(), "param", "ref")
        }
    }
    impl Parser {
        pub fn from_syntax(
            syntax: &Syntax,
            parameter_operator: &str,
            reference_operator: &str,
        ) -> Self {
            Parser {
                parameter_regex: {
                    let parameter_expression_regex_str = format!(
                        r"{} *(?P<optional>\({}\))?\({}\)(?P<name>\w+) *{}",
                        regex::escape(syntax.open_tag),
                        regex::escape(syntax.optional_operator),
                        regex::escape(parameter_operator),
                        regex::escape(syntax.close_tag)
                    );
                    let result = Regex::new(
                        format!(r"(?P<left>.+?)?{parameter_expression_regex_str}")
                            .as_str(),
                    )
                    .unwrap();
                    println!("{result}");
                    result
                },
                reference_line_regex: Regex::new(
                    format!(
                        r"^(?P<left>.+)?{} *(?P<optional>\({}\))?\({}\)(?P<name>\w+) *{}(?P<right>.+)?$",
                        regex::escape(syntax.open_tag),
                        regex::escape(syntax.optional_operator),
                        regex::escape(reference_operator),
                        regex::escape(syntax.close_tag)
                    )
                    .as_str(),
                )
                .unwrap(),
            }
        }
        pub fn parse<'a>(&'a self, text: &'a str) -> Result<Template<'a>, String> {
            let parsed_lines: Vec<_> = text
                .lines()
                .map(|line| {
                    dbg!(&line);
                    let parameters_captures: Vec<_> =
                        self.parameter_regex.captures_iter(line).collect();
                    dbg!(&parameters_captures);
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
                                        name: capture.name("name").unwrap().as_str(),
                                    })
                                }
                                let right =
                                    &line[parameters_captures.last().unwrap().get_match().end()..];
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
                            name: captures.name("name").unwrap().as_str(),
                            right: if let Some(left_match) = captures.name("right") {
                                Some(left_match.as_str())
                            } else {
                                None
                            },
                        }
                    } else {
                        Line::Raw { value: line }
                    }
                })
                .collect();
            Ok(Template {
                lines: parsed_lines,
            })
        }
    }

    pub enum TemplateParametersValue<'a> {
        Parameters(TemplateParameters<'a>),
        ValuesVec(Vec<&'a str>),
        ParametersVec(Vec<TemplateParameters<'a>>),
        Value(&'a str),
    }
    pub type TemplateParameters<'a> = HashMap<&'a str, TemplateParametersValue<'a>>;
    pub type Templates<'a> = HashMap<&'a str, Template<'a>>;

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
                                                        == (value_index + 1).try_into().unwrap()
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
                                            value_index = -1;
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
                                TemplateParametersValue::ParametersVec(
                                    subtemplate_parameters_vec,
                                ) => {
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
            self,
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
        let parser = Parser::default();
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
    #[test]
    fn test_render_param() {
        let parser = Parser::default();
        assert_eq!(
            parser
                .parse("one <!-- (param)p --> two")
                .unwrap()
                .render(
                    &TemplateParameters::from([("p", TemplateParametersValue::Value("lalala"))]),
                    &Templates::from([]),
                )
                .unwrap(),
            "one lalala two\n"
        );
    }
}
