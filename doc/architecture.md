# Architecture 

## Two stage pass

The SIMD parser has two passes. First pass finds interesting characters for analysis. It will find structurals like: `[`, `]`, `{`, `}`,
`:`, `? `, `: `, `- `, `---` (if at start of line) or `...` (if at start of line) and quasi structurals which are non-whitespace chars
between two structurals. We care about indent only on structurals.

Let's look at the analysis of a fragment below:

| Input               | ` ` | `[` | `a` | `]` | `:` | ` ` | `\n` | ` ` | ` ` | `b` |
|---------------------|-----|-----|-----|-----|-----|-----|------|-----|-----|-----|
| Primary structurals |     | 1   |     | 3   | 4   |     |      |     |     |     |
| Quasi structurals   |     |     | 2   |     |     |     |      |     |     | 9   |
| Row                 |     | 0   | 0   | 0   | 0   |     |      |     |     | 1   |
| Indent              |     | 1   | 1   | 1   | 1   |     |      |     |     | 2   |

- First structural is `[` at position `1`. It is at the start of a list. It has an indent `1`.
- Second structural is `a` at position `2`. It is an unquoted scalar.
- Third structural is `]` at position `3`. It is the end of a list.
- The interesting one is `:` at position `4`. It has an indent `1`. It is a start of a block mapping, which means that parser will scan
  the structurals to find an
  element that has greater indent than the `1`. In this case it's the next structural `b` at position `9`.

## Indent scan

The scan for indent structurals is necessary mostly in block scalars to be able to find starts of a block scalars while avoiding false
positives.
Take, for example, the following YAML:

```yaml
a: >
  - eggs
  - oil
```

Due to way structurals work, there will be false positives. Let's look at the structural table below:

| Input               | `a` | `:` | `>` | `-` | `-` | ` ` |
|---------------------|-----|-----|-----|-----|-----|-----|
| Primary structurals |     | 1   | 7   | 9   | 15  | 19  |
| Quasi structurals   | 0   |     | 2   |     |     |     |
| Indent              | 0   | 0   | 0   | 1   | 1   | 1   |

First issue we encounter is that `a` could be a scalar, but also it could be a key in a map. So we defer resolving it until we find the next
structural.
Second issue is that after `:` we have to find the key either in same line or on the next line but more indented.
Third issue is that after `>` is encountered any structurals more indented than `>` that aren't an empty line are also a part of it.

To solve these issues in order, we have to do the following:

1. When a scalar is found, we need to add it to the buffer, memorize the mark for its later processing and then wait for the next structural
   to decide.
2. When a mapping start is found (`:`) we have to know if we are in block, and if yes, check that there is a newline between it and next
   structural.
3. When a block scalar is found (`>` or `|`) we have to scan until we find next structural with less or equal indent (it has to be part of
   current map or a nested one). 

## Parsing string

How to parse string in the fastest way? Let's start with the simplest string to parse - a single quote.

```yaml
  'single  
     quotes '
  # Parsed to 'single␣quotes␣
```

To parse this we need to scan for odd number `'`.