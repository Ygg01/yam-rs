# Architecture 

## Two stage pass

The SIMD parser has two passes. First pass finds interesting characters for analysis. It will find structurals like: `[`, `]`, `{`, `}`,
`:`, `? `, `: `, `- `, and quasi structurals which are non-whitespace chars between two structurals. We care about indent only on
structurals.

Let's look at the analysis of a fragment below:

| Input               | ` ` | `[` | `a` | `]` | `:` | ` ` | `\n` | ` ` | ` ` | `b` |
|---------------------|-----|-----|-----|-----|-----|-----|------|-----|-----|-----|
| Primary structurals |     | 1   |     | 3   | 4   |     |      |     |     |     |
| Quasi structurals   |     |     | 2   |     |     |     |      |     |     | 9   |
| Row                 |     | 0   | 0   | 0   | 0   |     |      |     |     | 1   |

| Line data | 0 | 1 |
|-----------|---|---|
| IND       | 1 | 2 |
| NL        | 6 | 9 |

| Input | ` ` | ` ` | `\n ` | `\n` | ` ` | `\n ` | `b` |
|-------|-----|-----|-------|------|-----|-------|-----|
| pos   | 0   | 1   | 2     | 3    | 4   | 5     | 6   |
| row   | 0   | 0   | 0     | 1    | 2   | 2     | 3   |
| NL    |     |     | 2     | 3    |     | 5     |     |
