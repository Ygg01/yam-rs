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