use drunk_snail::*;

extern crate drunk_snail;

fn main() {
    divan::main();
}

#[divan::bench]
fn table(bencher: divan::Bencher) {
    let parser = drunk_snail::Parser::default();

    let table_template = parser.parse("<table>\n    <!-- (ref)Row -->\n</table>");
    let row_template = parser.parse("<tr>\n    <td><!-- (param)cell --></td>\n</tr>");
    let templates = Templates::from([("Row", &row_template)]);

    const SIZE: usize = 100;
    let mut rows: Vec<TemplateParameters> = Vec::from([]);
    for _ in 0..SIZE {
        rows.push(TemplateParameters::from([(
            "cell",
            TemplateParametersValue::ValuesVec(
                (0..SIZE)
                    .flat_map(|y| (0..SIZE).map(move |x| (x + y * SIZE).to_string()))
                    .collect(),
            ),
        )]));
    }
    let parameters =
        TemplateParameters::from([("Row", TemplateParametersValue::ParametersVec(rows))]);

    bencher.bench(|| table_template.render(&parameters, &templates).unwrap());
}
