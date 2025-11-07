use drunk_snail::*;

extern crate drunk_snail;

fn main() {
    divan::main();
}

#[divan::bench(args=[10, 100, 1000])]
fn table(bencher: divan::Bencher, size: usize) {
    let parser = drunk_snail::Parser::from_syntax(&Syntax::default(), "param", "ref").unwrap();

    let table_template = parser
        .parse("<table>\n    <!-- (ref)Row -->\n</table>")
        .unwrap();
    let row_template = parser
        .parse("<tr>\n    <td><!-- (param)cell --></td>\n</tr>")
        .unwrap();
    let templates = Templates::from([("Row", row_template)]);

    let parameters = TemplateParameters::from([(
        "Row",
        TemplateParametersValue::ParametersVec(
            (0..size)
                .map(move |y| {
                    TemplateParameters::from([(
                        "cell",
                        TemplateParametersValue::ValuesVec(
                            (0..size).map(move |x| (x + y * size).to_string()).collect(),
                        ),
                    )])
                })
                .collect(),
        ),
    )]);

    bencher.bench(|| table_template.render(&parameters, &templates).unwrap());
}
