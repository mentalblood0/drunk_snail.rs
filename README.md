# 🌪️ drunk_snail

Pure [rust](https://rust-lang.org/) implementation of template language originally presented [here](https://github.com/mentalblood0/drunk_snail)

## Why this language?

- Easy syntax
- Separates logic and data

## Why better then C / Python / Nim / Crystal implementations?

- Compiled and statically typed yet memory safe
- Small codebase
- Allow for parser configuration
- Significantly (~x2.5) faster then Crystal implementation

## Example

Row:

```html
<tr>
  <td><!-- (param)cell --></td>
</tr>
```

Table:

```html
<table>
  <!-- (ref)Row -->
</table>
```

Arguments:

```json
{
  "Row": [
    {
      "cell": ["1", "2"]
    },
    {
      "cell": ["3", "4"]
    }
  ]
}
```

Result:

```html
<table>
  <tr>
    <td>1</td>
    <td>2</td>
  </tr>
  <tr>
    <td>3</td>
    <td>4</td>
  </tr>
</table>
```

See [tests](./src/lib.rs) and [benchmark](./benches/main.rs) for usage examples
