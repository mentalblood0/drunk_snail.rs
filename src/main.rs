use regex::Regex;

#[derive(Debug, PartialEq, Eq)]
struct Syntax {
    open_tag: String,
    close_tag: String,
    optional_operator: String,
}
impl Default for Syntax {
    fn default() -> Self {
        Syntax {
            open_tag: String::from("<!--"),
            close_tag: String::from("-->"),
            optional_operator: String::from("optional"),
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
struct Template<'a> {
    lines: Vec<Line<'a>>,
}

#[derive(Debug)]
struct Parser {
    parameter_regex: Regex,
    reference_line_regex: Regex,
}
impl Default for Parser {
    fn default() -> Self {
        Parser::from_syntax(&Syntax::default(), "param", "ref")
    }
}
impl Parser {
    fn from_syntax(syntax: &Syntax, parameter_operator: &str, reference_operator: &str) -> Self {
        Parser {
            parameter_regex: {
                let parameter_expression_regex_str = format!(
                    r"{} *(?P<optional>\({}\))?\({}\)(?P<name>\w+) *{}",
                    regex::escape(syntax.open_tag.as_str()),
                    regex::escape(syntax.optional_operator.as_str()),
                    regex::escape(parameter_operator),
                    regex::escape(syntax.close_tag.as_str())
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
                    regex::escape(syntax.open_tag.as_str()),
                    regex::escape(syntax.optional_operator.as_str()),
                    regex::escape(reference_operator),
                    regex::escape(syntax.close_tag.as_str())
                )
                .as_str(),
            )
            .unwrap(),
        }
    }
    fn parse<'a>(&'a self, text: &'a str) -> Result<Template<'a>, String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let parser = Parser::default();
        println!("{parser:?}");

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
}
