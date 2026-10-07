# Repository instructions

## Commit messages

All commits must follow [Conventional Commits](https://www.conventionalcommits.org/) so they remain compatible with conventional-changelog.

Use this form:

```text
<type>[optional scope][!]: <description>
```

Allowed types are `feat`, `fix`, `perf`, `refactor`, `test`, `docs`, `build`, `ci`, `chore`, and `revert`.

- Write the description in lowercase imperative form without a trailing period.
- Keep the subject concise; aim for 72 characters or fewer.
- Use a scope when it makes the affected subsystem clearer, such as `paint`, `layout`, `renderer`, or `app`.
- Mark breaking changes with `!` and explain them in a `BREAKING CHANGE:` footer.

Examples:

```text
perf(renderer): retain raster tiles across scroll frames
fix(layout): invalidate fragments after style mutation
feat!: rename the public crate to rechrom
```
