# Branding

> **Status:** approved
> **Last updated:** 2026-03-23
> **Owner:** Core Team

---

## Project Name

### Wit

**Meaning:** "Wit" refers to quick, sharp intelligence - the kind of innate smartness that
does not require machinery. A perfect contrast to "AI": wit is natural intelligence, not
artificial.

**Pronunciation:** /wit/ (like "with" but without the "h")

**Name characteristics:**
- Short, 3 characters, easy to remember
- Quick to type in the terminal: `wit`
- Does not conflict with any popular tool
- Conveys the right spirit: smart but not AI

### Basic Information

| Property | Value |
|---|---|
| Name | Wit |
| Repo name | `wit-term` |
| CLI command | `wit` |
| Tagline | "The terminal that gets it." |
| Category | Terminal Emulator, Developer Tool |
| License | MIT |

### GitHub Description

```
A context-aware terminal emulator built with Rust. Wit detects your project
environment - git, Docker, Node, Cargo, Python, and more - and provides
intelligent completions without AI, without cloud, without telemetry.
Just sharp, local instinct.
```

---

## Color Palette

### Primary Colors

| Role | Hex | RGB | Description |
|---|---|---|---|
| Primary | `#58E6D9` | 88, 230, 217 | Cyan/teal - terminal vibe |
| Glow | `#4DF0E0` | 77, 240, 224 | Bright cyan - icon accent, hover |
| Background | `#0D1117` | 13, 17, 23 | Deep dark - main background |
| Surface | `#161B22` | 22, 27, 34 | Elevated background (sidebars, popups) |
| Border | `#30363D` | 48, 54, 61 | Border between regions |

### Text Colors

| Role | Hex | RGB | Description |
|---|---|---|---|
| Text Primary | `#F0F6FC` | 240, 246, 252 | Primary text |
| Text Secondary | `#8B949E` | 139, 148, 158 | Secondary text, labels |
| Text Muted | `#484F58` | 72, 79, 88 | Very faint text, placeholder |

### Accent Colors

| Role | Hex | RGB | Description |
|---|---|---|---|
| Accent / Highlight | `#D4A857` | 212, 168, 87 | Amber - completions, cursor |
| Success | `#3FB950` | 63, 185, 80 | Green - success states |
| Warning | `#D29922` | 210, 153, 34 | Orange-yellow - warnings |
| Error | `#F85149` | 248, 81, 73 | Red - errors |
| Info | `#58A6FF` | 88, 166, 255 | Blue - info |

### Background Gradient (Icon)

```
Start: #1A1A28
End:   #2A2A3A
Angle: 135deg
```

### ANSI Terminal Colors (16-color palette)

```
# Normal colors
color0  = "#282C34"  # Black
color1  = "#F85149"  # Red
color2  = "#3FB950"  # Green
color3  = "#D29922"  # Yellow
color4  = "#58A6FF"  # Blue
color5  = "#BC8CFF"  # Magenta
color6  = "#58E6D9"  # Cyan
color7  = "#F0F6FC"  # White

# Bright colors
color8  = "#484F58"  # Bright Black
color9  = "#FF7B72"  # Bright Red
color10 = "#56D364"  # Bright Green
color11 = "#E3B341"  # Bright Yellow
color12 = "#79C0FF"  # Bright Blue
color13 = "#D2A8FF"  # Bright Magenta
color14 = "#76E4DA"  # Bright Cyan
color15 = "#FFFFFF"  # Bright White
```

---

## Typography

### UI Font

| Context | Font | Fallback |
|---|---|---|
| UI Labels | Inter | system-ui, sans-serif |
| UI Body | Inter | system-ui, sans-serif |

### Terminal Font

| Context | Font | Fallback |
|---|---|---|
| Terminal text | JetBrains Mono | Fira Code, Menlo, Consolas, monospace |
| Terminal bold | JetBrains Mono Bold | - |
| Terminal italic | JetBrains Mono Italic | - |

### Font Sizes

| Context | Size | Line Height |
|---|---|---|
| Terminal default | 14px | 1.4 (20px) |
| Terminal min | 10px | 1.4 |
| Terminal max | 24px | 1.4 |
| UI body | 13px | 1.5 |
| UI small | 11px | 1.4 |
| UI heading | 15px | 1.3 |

---

## Icon

### Concept

A stylized **W** formed from two mirrored **V** shapes, with cyan strokes and soft glow
on a dark background. Evokes the terminal prompt `>_`.

### Specifications

| Property | Value |
|---|---|
| File | `wit-icon.svg` |
| Standard size | 1024 x 1024 px |
| Style | Flat minimal, subtle LED glow |
| Shape | Superellipse rounded square |
| Stroke color | `#4DF0E0` (Glow variant) |
| Background | `#1A1A2E` |
| Stroke width | ~60px at 1024px |
| Corner radius | ~220px at 1024px (~21.5%) |

### Icon Sizes Needed

| Platform | Sizes |
|---|---|
| macOS | 16, 32, 64, 128, 256, 512, 1024 |
| Windows | 16, 24, 32, 48, 64, 256 (.ico) |
| Linux | 16, 22, 24, 32, 48, 64, 128, 256, 512 |
| Favicon | 16, 32, 180 (apple-touch), 192, 512 |

### Image generation prompt (Midjourney / DALL-E / Flux)

```
A modern app icon featuring a stylized letter "W" made of two mirrored
chevron "V" shapes that meet at the bottom center. The left V angles
down-right, the right V angles down-left, forming a symmetrical W.
Each stroke is a thick rounded-end line segment in bright cyan aqua
(#4DF0E0) with a soft subtle glow. The strokes do not connect - there
is a small gap at the bottom center where the two V shapes nearly touch.
Dark charcoal background (#1A1A2E) with a smooth superellipse rounded
square shape. Flat minimal design, slight ambient glow on the cyan strokes.
No text, no other elements. Clean tech aesthetic. macOS app icon style,
1024x1024.
```

```
Negative prompt:
text, words, additional shapes, 3D extrusion, glossy reflection,
realistic materials, busy details, gradient background, shadows on background
```

---

## Tone of Voice

### In documentation

- Clear, direct, not verbose
- Technical but accessible
- Confident but not arrogant

### In marketing / README

- Concise, punchy
- Emphasize the difference: no AI, no cloud, no telemetry
- Demo-driven: "Show, don't tell"

### Do not use

- "Revolutionary", "game-changing", "next-gen"
- Unnecessary buzzwords
- Direct negative comparisons with competitors

### Do use

- "Context-aware", "local-first", "rule-based"
- "Sharp", "fast", "transparent"
- "Open-source", "community-driven"
