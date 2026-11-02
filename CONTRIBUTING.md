# Coding

## Getting started

1. Fork the repo and create your branch from `master`.
2. Run `git submodule update --init` to ensure that test cases are pulled it. This needs to be done rarely since the test cases are not
   changing .
3. If adding code, add tests that fail if the newly added code wouldn't be there.
4. If changing API, update the documentation and examples accordingly.

### Note on PRs

I'll try to respond in 24-72h but be patient, it's not my full-time job. All PRs will be squashed into and merged into the `master` branch.

### Considerations

- Keep dependencies to a minimum.
- No C dependencies
- Must work on at least Windows, macOS, and Linux (including with their line endings).

### Testing

For testing it's suggested to use `cargo nextest run`
because during tests a case can sometimes `TIMEOUT`. These cases are nicely addressed by `nextest`.

#### Installing nextest

To install [cargo-nextest](https://github.com/nextest-rs/nextest/tree/main/cargo-nextest) visit the previous link. You can install from
source for your OS or you can install it from source run using:

```sh
cargo install --locked cargo-nextest
```

## LLM and AI policy

I don't welcome nor reject any AI generated code or PRs.

It's a tool, but at the end of the day it's your responsibility. If I have questions I'll expect **YOU** (the submitter) to answer it, not
your AI. If you submit LLM response without me asking for it, it will be grounds for closing the PR.

