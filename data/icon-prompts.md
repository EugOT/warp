# Imagen prompts for Walcode and ZeroClaw icon assets

These prompts target Imagen 3 / 4. The output is consumed at
`app/assets/bundled/svg/walcode.svg` and `app/assets/bundled/svg/zeroclaw.svg`,
so request final delivery as a flat SVG (or transparent PNG that we can vector
trace) sized 24x24 with a single fill color matching the brand.

## Brand constraints

| Agent     | Brand color     | File path                                       | viewBox  |
| --------- | --------------- | ----------------------------------------------- | -------- |
| Walcode   | `#DC2626` crimson | `app/assets/bundled/svg/walcode.svg`            | `0 0 24 24` |
| ZeroClaw  | `#06B6D4` cyan    | `app/assets/bundled/svg/zeroclaw.svg`           | `0 0 24 24` |

Existing siblings to match in weight: `claude.svg`, `codex.svg`, `gemini_cli.svg`,
`opencode.svg`, `pi.svg`, `auggie.svg` (all are flat, single-color, glyph-style
marks ~16-22px of inked area inside a 24x24 box).

---

## Walcode (`#DC2626`, crimson)

```text
A minimalist square monogram logo for "Walcode", a wall-of-code coding agent.
Single flat color #DC2626 (deep crimson) on transparent background.
Composition: a stylized wall built from rectangular bricks on the left half,
fused with an angle-bracket "code chevron" </> on the right half. Bricks are
solid, no outlines. Chevron is bold, two simple < and > strokes. Geometry only,
no text, no gradient, no shading, no drop-shadow. Monoline, vector-clean,
sharp 90-degree corners, even stroke widths.
Render fits cleanly inside a 24x24 viewBox with ~2px optical padding.
Style reference: pictographic glyph icon, comparable to GitHub or Linear marks.
Output: optimized inline SVG with one <path> using fill="#DC2626", or a
transparent PNG at 256x256 suitable for vectorization.
```

Notes for the operator:
- Aspect ratio: 1:1 square.
- Negative space matters more than detail; the icon is shown at 16px in chips.
- Reject any output that includes the word "Walcode" or any character glyphs.

---

## ZeroClaw (`#06B6D4`, electric cyan)

```text
A minimalist square emblem logo for "ZeroClaw", an Agent Client Protocol coding
agent. Single flat color #06B6D4 (electric cyan) on transparent background.
Composition: a stylized "0" zero ring on the left fused with three sharp claw
slashes on the right that exit the ring at increasing angles, suggesting both
"zero" (the digit) and "claw" (a striking talon). Strokes are bold, terminals
are squared. No outlines, no gradients, no shading, no drop-shadow. Geometric,
not organic; angles are precise multiples of 15 degrees.
Render fits cleanly inside a 24x24 viewBox with ~2px optical padding.
Style reference: pictographic glyph icon comparable to Vercel or Linear marks.
Output: optimized inline SVG with one or two <path> elements using
fill="#06B6D4", or a transparent PNG at 256x256 suitable for vectorization.
```

Notes for the operator:
- Aspect ratio: 1:1 square.
- The mark must read at 12-14px (notification badge size).
- Reject any output that includes the digit "0" rendered as a typographic glyph
  or any letterforms; the "0" must be a pure geometric ring.

---

## Drop-in replacement workflow

1. Save the Imagen output as `walcode.svg` / `zeroclaw.svg`.
2. Run through SVGO (or a similar minifier) so the path is a single,
   un-prefixed `<path>` with `fill="#DC2626"` or `fill="#06B6D4"`.
3. Set `width="24"`, `height="24"`, `viewBox="0 0 24 24"`, and remove all
   inline styles, `<g>` wrappers, ids, and titles.
4. Overwrite the placeholders at `app/assets/bundled/svg/walcode.svg` and
   `app/assets/bundled/svg/zeroclaw.svg`.
5. Re-run the Warp build (`cargo build -p warp`); no code change is needed
   because `WalcodeLogo` / `ZeroclawLogo` already point at these filenames.
