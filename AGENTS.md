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

## Browser verification

Use a headless Rechrom process for routine automated testing and regression
verification. Connect to its DevTools endpoint to navigate pages, dispatch
input events, collect traces, and inspect results. Do not open or automate a
visible browser window for these checks, because it consumes extra resources
and can interfere with repeatable measurements.

For screenshot and pixel-fidelity comparisons, capture the page through the
browser's page screenshot protocol. Use `Page.captureScreenshot` for Rechrom
and the same CDP method for Chromium. Capture both browsers separately with the
same CSS viewport, device-pixel ratio, URL, scroll position, loading state, and
animation state, then compare the returned page pixels directly. Do not use an
operating-system screenshot as the primary visual-regression input: window
chrome, shadows, occlusion, display scaling, and WindowServer timing make it a
different measurement. A system screenshot may be used only for a final check
of native window and compositor integration.

Rechrom's current DevTools HTTP transport is not full CDP merely because a
method has a CDP-compatible name. When reporting protocol compatibility,
distinguish matching method semantics from matching target discovery,
WebSocket JSON-RPC transport, parameters, and response shape.

Open a visible Release window only for final user-facing experiential
verification, or when the behavior being tested specifically requires native
window interaction.

For a visible URL verification, launch the exact Rechrom executable for the
current platform and pass the requested address through the cross-platform
command-line contract, for example:

```text
<rechrom-executable> --url https://example.com --devtools-port 0
```

Resolve `<rechrom-executable>` to the artifact built in this workspace (for
example the executable inside a macOS bundle, the Linux binary, or the Windows
`.exe`). Stop stale Rechrom processes first. Do not use an operating-system app
name lookup to start the browser: it can select a different installed build
without forwarding the URL and silently fall back to `about:home`. After launch,
use the cross-platform DevTools endpoint to confirm that the Page target URL is
the requested URL before handing the window to the user.

## Core engineering principles

1. Use the repository's browser pixel-comparison workflow whenever visual
   correctness is involved. Capture Rechrom and Chromium through
   `Page.captureScreenshot` under identical viewport, DPR, URL, scroll,
   loading, and animation conditions, then run the pixel-diff tool. A strict
   zero-pixel result counts only when both browsers rendered the intended real
   page successfully; blank pages, error pages, and mismatched page states do
   not count.
2. Preserve the existing architectural and crate boundaries. Keep modules
   independent of their thread placement, communicate through typed
   interfaces, maintain one-way dependencies, and expose the smallest interface
   that expresses the required behavior. Do not solve a local problem with
   cross-layer calls, application-specific branches, or unnecessary public API.
3. Match Chromium's logical behavior and design intent for each fix. Trace the
   corresponding Chromium path and align lifecycle, invalidation, ownership,
   scheduling, and rendering semantics. Do not introduce a plausible custom
   behavior merely because it passes one page or test; implement the reusable
   class of behavior represented by Chromium.
