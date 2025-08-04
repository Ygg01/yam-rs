# Structurals and Indents
The parser does a double pass. First pass does a scan for structurals and indents . Structurals are a `Vec<u64>` that can address up to `u64 / 2` characters. We reserve the highest bit for `SEQ`/`MAP`/`ALIAS`/`TAG` tag.

Indentation is a `u32`. Assumption is that most YAML documents will not need more than 4GiB of indents.

First we need to track indents and structurals. Why? Consider following example

```yaml
 - name: x
   type: y
```

For this simple example we need to emit following set of structurals and indents (assuming LF line separator):

| structurals 64th bit | structurals 63..0 bit | indent | node       |
|----------------------|-----------------------|--------|------------|
| 1                    | 1                     | 1      | - (SEQ)    |
| 0                    | 3                     | 3      | `name`     |
| 1                    | 7                     | 3      | : (MAP)    |
| 0                    | 9                     | 3      | `x`        |
| 0                    | 10                    | 3      | END of `x` |
| 0                    | 14                    | 3      | `type`     |
| 1                    | 18                    | 3      | : (MAP)    |
| 0                    | 20                    | 3      | `y`        |
| 0                    | 21                    | 3      | EOF        |

Indents are important because if string `type` was not indented properly, it would have been a valid MAP node.

Interesting enough `-` doesn't immediately impact indentation. For example.

```yaml
- - abc
  -  xyz 
```


| structurals 64th bit | structurals 63..0 bit | indent | node         |
|----------------------|-----------------------|--------|--------------|
| 1                    | 0                     | 0      | - (SEQ)      |
| 1                    | 2                     | 2      | - (SEQ)      |
| 0                    | 4                     | 4      | `abc`        |
| 0                    | 7                     | 4      | END of `abc` |
| 1                    | 10                    | 2      | - (SEQ)      |
| 0                    | 13                    | 5      | `xyz`        |
| 0                    | 16                    | 5      | END OF `xyz` |
| 0                    | 17                    | 5      | EOF          |

This is to allow nesting block `SEQ` per specifications

We have to keep 64-th bit to differentiate a scalar starting with special reserved letters to differentiate the MAP/SEQ structurals from beginning of unquoted scalars. E.g.

```yaml
- abc
  - xyz 
```

| structurals 64th bit | structurals 63..0 bit | indent | node         |
|----------------------|-----------------------|--------|--------------|
| 1                    | 0                     | 0      | - (SEQ)      |
| 0                    | 2                     | 2      | `abc`        |
| 0                    | 5                     | 2      | END of `abc` |
| 0                    | 9                     | 2      | `- xyz`      |
| 0                    | 13                    | 5      | EOF          |

Structural of `- xyz` points to `bytes[9]` i.e. `-` so looking at it isn't enough to determine if it's a sequence or a scalar start.

# Classifying bytes

We have to classify following character values: `[` (0x5B), `]` (0x5D), `{` (0x7B), `}`(0x7D), `,` (0x2C) and `:` (0x3A)
for flow scalar; `'` (0x27), `"` (0x22), `>` (0x3E), `|` (0x7C) for strings; ` ` (0x20), `\t` (0x09), `\n` (0x0A) `\r` (
0x0D) for whitespace; `&` (0x26), `*` (0x2A) , `%` (0x25), `?` (0x3F), `!` (0x21), `-` (0x2D), `#` (0x23), `.` (0x2E)

| Code points | Character | Classification    |
|-------------|-----------|-------------------|
| `0x09`      | `\t`      | WS                |
| `0x0A`      | `\n`      | WS                |
| `0x0D`      | `\r`      | WS                |
| `0x20`      | ` `       | WS                |
| `0x21`      | `!`       | TAG               |
| `0x22`      | `>`       | STRING            |
| `0x22`      | `"`       | STRING            |
| `0x23`      | `#`       | COMMENT           |
| `0x25`      | `%`       | TAG               |
| `0x26`      | `&`       | ALIAS             |
| `0x27`      | `'`       | STRING            |
| `0x2A`      | `*`       | ALIAS             |
| `0x2C`      | `,`       | SEQ               |
| `0x2D`      | `-`       | SEQ/START (BLOCK) |
| `0x2E`      | `.`       | END (BLOCK)       |
| `0x3A`      | `:`       | MAP (BLOCK)       |
| `0x3F`      | `?`       | MAP (BLOCK)       |
| `0x5B`      | `[`       | SEQ               |
| `0x5D`      | `]`       | SEQ               |
| `0x7B`      | `{`       | MAP               |
| `0x7C`      | `\|`      | STRING            |
| `0x7D`      | `}`       | MAP               |

Because of string/comment shadowing, we only look for non-string and non-comment elements.

First stage we get rid of comments, then get rid of single and double-quoted strings.

## Classificator

We need to classify stuff into:

- Flow: `[` (0x5B), `]` (0x5D),`{` (0x7B), `}` (0x7D), `,` (0x2C), `:` (0x3A)
- Block: `?` (0x3F), `:` (0x3A), `-` (0x2D)
- Whitespace: ` ` (0x20), `\t` (0x09), `\n` (0x0A), `\r` (0x0D)

| Code points                    | Characters         | Desired values |
|--------------------------------|--------------------|----------------|
| `0x3F`                         | `?`                | 1              |
| `0x2D`                         | `-`                | 2              |
| `0x2C`                         | `,`                | 4              |
| `0x3A`                         | `:`                | 8              |
| `0x5B`, `0x5D`, `0x7B`, `0x7D` | `[`, `]`, `{`, `}` | 16             |
| `0x09`, `0x0A`, `0x0D`         | `\t`, `\n`, `\r`   | 32             |
| `0x20`                         | ` `                | 64             |
| Anything else                  |                    | 0              |

Which produces the below low/high nibble table.

|            | 0  | 1 | ... | 8 | 9  | A  | B  | C | D  | E | F | high nibble |
|------------|----|---|-----|---|----|----|----|---|----|---|---|-------------|
| 0          |    |   |     |   | 32 | 32 |    |   | 32 |   |   | 32          |
| 1          |    |   |     |   |    |    |    |   |    |   |   | 0           |
| 2          | 64 |   |     |   |    |    |    | 4 | 2  |   |   | 70          |
| 3          |    |   |     |   |    | 8  |    |   |    |   | 1 | 9           |
| 4          |    |   |     |   |    |    |    |   |    |   |   | 0           |
| 5          |    |   |     |   |    |    | 16 |   | 16 |   |   | 16          |
| 6          |    |   |     |   |    |    |    |   |    |   |   | 0           |
| 7          |    |   |     |   |    |    | 16 |   | 16 |   |   | 16          |
| ...        |    |   |     |   |    |    |    |   |    |   |   | 0           |
| low nibble | 64 |   | ... |   | 32 | 40 | 16 | 4 | 50 |   | 1 | x           |

From which we can derive the following values

```rust
const LOW_NIBBLE: [u8; 16] = [64, 0, 0, 0, 0, 0, 0, 0, 0, 32, 40, 16, 4, 50, 0, 1];
const HIGH_NIBBLE: [u8; 16] = [32, 0, 70, 9, 0, 16, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0];
```

This way we can extract simultaneously:

- Block structurals with `0b1011` (`0xB`)
- Flow structurals with `0b1_1000` (`0x18`)
- Spaces with `0b0100_0000` (`0x40`)
- Whitespaces with `0b0110_0000` (`0x60`)

# Strings overlapping

The problem with strings is that they can overlap - you can have double-quoted, single-quoted, comments and unquoted strings side by side.

| Input              | `'` | ` ` | `"` | ` ` | `#` | `"` | ` ` | `'` |     |
|--------------------|-----|-----|-----|-----|-----|-----|-----|-----|-----|
| Double-quotes (DQ) |     |     | 1   | 1   | 1   | 1   |     |     | 60  |
| Single-quotes (SQ) | 1   | 1   | 1   | 1   | 1   | 1   | 1   | 1   | 255 |
| Comments (CM)      |     |     |     |     | 1   | 1   | 1   | 1   | 240 |

To get the first string, we need some sort of find first byte.

# Approach number one - do it in Stage 2 Scanner

Most correct approach. In Stage 2, one can skip any number of pseudo structurals. When encountering SQ at position 0, we fast-forward to
position 7. Pro would be useful for other string types.

# Approach number two - do it in Stage 1 Scanner.

`OR` all other quotes types (e.g. for single quotes `240 | 60`), and then negate it with `NOT` (e.g. `!( 240 | 60) & 1`) and check start
values.

Calulate for DQ: `255 | 240 = 255` -> `!255 & 4 = 0`

Calulate for SQ: `240 | 60  = 252` -> `!252 & 1` = `1`

Calulate for CM: `255 | 240 = 255` -> `!255 & 16` = `0`

# Approach number three - ???

`XOR` all quotes types (e.g. `255 ^ 240 ^ 60 = 51`) and then and them with their original value