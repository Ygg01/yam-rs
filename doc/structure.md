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

| Code points | Character | Classification |
|-------------|-----------|----------------|
| `0x09`      | `\t`      | WS             |
| `0x0A`      | `\n`      | WS             |
| `0x0D`      | `\r`      | WS             |
| `0x20`      | ` `       | WS             |
| `0x21`      | `!`       | TAG            |
| `0x22`      | `>`       | STRING         |
| `0x22`      | `"`       | STRING         |
| `0x23`      | `#`       | COMMENT        |
| `0x25`      | `%`       | TAG            |
| `0x26`      | `&`       | ALIAS          |
| `0x27`      | `'`       | STRING         |
| `0x2A`      | `*`       | ALIAS          |
| `0x2C`      | `,`       | SEQ            |
| `0x2D`      | `-`       | SEQ/START      |
| `0x2E`      | `.`       | END            |
| `0x3A`      | `:`       | MAP            |
| `0x3F`      | `?`       | MAP            |
| `0x5B`      | `[`       | SEQ            |
| `0x5D`      | `]`       | SEQ            |
| `0x7B`      | `{`       | MAP            |
| `0x7C`      | `\|`      | STRING         |
| `0x7D`      | `}`       | MAP            |

Because of string/comment shadowing, we only look for non-string and non-comment elements.

First stage we get rid of comments, then get rid of single and double-quoted strings.

## Flow classificator

We need two classifications unlike JSONs. First we do the flow classificator.

| Code points                    | Characters         | Desired values |
|--------------------------------|--------------------|----------------|
| `0x2C`                         | `,`                | 1              |
| `0x3A`                         | `:`                | 2              |
| `0x5B`, `0x5D`, `0x7B`, `0x7D` | `[`, `]`, `{`, `}` | 4              |
| `0x09`, `0x0A`, `0x0D`         | `\t`, `\n`, `\r`   | 8              |
| `0x20`                         | ` `                | 16             |

Which produces the below low/high nibble table.

|            | 0  | 1 | ... | 8 | 9 | A  | B | C | D  | E | F | high nibble |
|------------|----|---|-----|---|---|----|---|---|----|---|---|-------------|
| 0          |    |   |     |   | 8 | 8  |   |   | 8  |   |   | 8           |
| 1          |    |   |     |   |   |    |   |   |    |   |   | 0           |
| 2          | 16 |   |     |   |   |    |   | 1 |    |   |   | 17          |
| 3          |    |   |     |   |   | 2  |   |   |    |   |   | 2           |
| 4          |    |   |     |   |   |    |   |   |    |   |   | 0           |
| 5          |    |   |     |   |   |    | 4 |   | 4  |   |   | 4           |
| 6          |    |   |     |   |   |    |   |   |    |   |   | 0           |
| 7          |    |   |     |   |   |    | 4 |   | 4  |   |   | 4           |
| ...        |    |   |     |   |   |    |   |   |    |   |   | 0           |
| low nibble | 16 |   | ... |   | 8 | 10 | 4 | 1 | 12 |   |   | x           |

From which we can derive the following values

```rust
const LOW_NIBBLE: [u8; 16] = [16, 0, 0, 0, 0, 0, 0, 0, 0, 8, 10, 4, 1, 12, 0, 0];
const HIGH_NIBBLE: [u8; 16] = [8, 0, 17, 2, 0, 4, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0];
```

## Block classificator

YAML block classificators is a bit different. We can reuse `:`, `,` and ` ` from Flow classificator.
That leaves `>` (0x3E), `|` (0x7C), `?` (0x3F), `!` (0x21), `%` (0x25), `&` (0x26), `*` (0x2A) ,`-` (0x2D), `.` (0x2E)

| Code points                                    | Characters                    | Desired values |
|------------------------------------------------|-------------------------------|----------------|
| `0x3A`, `0x3E`, `0x3F`                         | `:`, `>`,  `?`                | 1              |
| `0x21`, `0x25`, `0x26`, `0x2A`, `0x2D`, `0x2E` | `!`,  `%`, `&`, `*`, `-`, `.` | 2              |
| `0x7C`                                         | `\| `                         | 4              |
| `0x20`                                         | ` `                           | 16             |

First we find all comments, single and double quotes that might shadow the structurals, then any block strings
candidates
that mig

Sadly, because of YAML unquoted nature, we need to store not just index of element but also its indent, so we can skip
over
any indents lesser than a value.