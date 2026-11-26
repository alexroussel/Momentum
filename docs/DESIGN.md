# Momentum — Design System & Screens

> Status: **draft**. Tokens here are the source for `apps/desktop/src/styles/tokens.css` and `tailwind.config.ts`.
> Mood: *watch dial + logbook + dashboard*. Dark by default, dense but airy, precise.
> Identity is our own: no Session logo, icon, illustration or copy is reused.

---

## 1. Design tokens

All colors are exposed as CSS custom properties; Tailwind classes reference the variables, so themes and the
category tint switch without re-rendering.

### 1.1 Surfaces & text

| Token | Dark (default) | Light | Use |
|---|---|---|---|
| `--bg-app` | `#1E1E20` | `#F4F3EF` (warm paper) | window background |
| `--bg-panel` | `#2A2A2D` | `#FBFAF7` | panels, sidebars |
| `--bg-card` | `#323236` | `#FFFFFF` | cards, inputs |
| `--bg-hover` | `#3A3A3F` | `#ECEAE4` | hover rows / buttons |
| `--bg-inset` | `#161618` | `#E9E7E1` | dial inner disc, wells |
| `--border` | `#3F3F45` (1 px, 60 % opacity) | `#DEDBD3` | hairlines |
| `--text-1` | `#F2F2F3` | `#17171A` | primary |
| `--text-2` | `#A1A1AA` | `#55555E` | secondary |
| `--text-3` | `#6B6B73` | `#8C8C95` | tertiary, ticks, labels |
| `--focus-ring` | `--cat` @ 70 % | `--cat` @ 80 % | keyboard focus (2 px, 2 px offset) |
| `--now-line` | `#FF4D4F` | `#E5484D` | current-time line |

Light theme is designed, not inverted: warm paper background, ink-like text, the dial inner disc becomes a light
recessed well with darker ticks, glows are replaced by soft colored shadows at 25 % strength.

### 1.2 Category palette (12 presets + free picker)

| Name | Hex | Name | Hex |
|---|---|---|---|
| Coral (default "Focus") | `#E5484D` | Pink | `#EC4899` |
| Violet | `#7C3AED` | Orange | `#F97316` |
| Teal | `#14B8A6` | Indigo | `#6366F1` |
| Amber | `#F59E0B` | Cyan | `#06B6D4` |
| Blue | `#3B82F6` | Lime | `#84CC16` |
| Green | `#22C55E` | Graphite | `#8B8D98` |

**Tinting.** The active category sets `--cat`. Derived tokens (computed in OKLCH so every preset behaves alike):

| Token | Derivation | Use |
|---|---|---|
| `--cat` | category hex | hand, primary button, focus ring, timeline blocks |
| `--cat-hi` | `oklch(from --cat calc(l + 0.08) c h)` | radial gradient end of dial sector (coral → `#F07178`) |
| `--cat-soft` | `--cat` @ 16 % over `--bg-app` | chips, break blocks, hover fills |
| `--cat-glow` | `--cat` @ 45 %, blur 32 px | primary button halo |
| `--brk` | `oklch(from --cat 0.72 calc(c * 0.30) 235)` | break sector: cool, desaturated |
| `--overtime-bg` | `color-mix(in oklch, --cat 55 %, --bg-app)` | full-window overtime background |
| `--brk-overtime-bg` | `color-mix(in oklch, --brk 25 %, #0E0E10)` | break overtime: darker |

Text on `--cat` uses `--text-on-cat`, picked automatically (white or `#141416`) to reach AA contrast.

### 1.3 Typography

- **UI**: Inter (OFL), `font-feature-settings: "cv11", "ss01"`.
- **Numbers, stats, labels**: JetBrains Mono (OFL), `font-variant-numeric: tabular-nums` everywhere.
- **Label style** (`.label-mono`): mono, 11 px, `letter-spacing: 0.08em`, uppercase, `--text-3`
  → e.g. `3 SESSIONS · 0 DISTRACTED · 3H 20M FOCUS`.

| Role | Size / line / weight |
|---|---|
| Display (timer title) | 28 / 34 / 600, tracking −0.01em |
| Countdown | mono 56 / 60 / 500 |
| Title | 18 / 24 / 600 |
| Body | 14 / 20 / 400 |
| Small | 12 / 16 / 450 |
| Label mono | 11 / 14 / 500, caps, +0.08em |

### 1.4 Spacing, radii, elevation, motion

- Spacing scale (px): 2, 4, 6, 8, 12, 16, 20, 24, 32, 40, 56.
- Radii: window 12, card 12, popover 10, input 10, chip 999, primary button 999 (pill).
- Elevation: none by default; popovers `0 8px 24px rgb(0 0 0 / .35)`; the only colored light is the primary
  button glow and the dial halo.
- Motion: `--ease-out: cubic-bezier(.2,.8,.2,1)`, `--ease-in-out: cubic-bezier(.65,0,.35,1)`.
  Phase transitions = fade + scale 0.98→1, 320 ms. Hover 120 ms. Overtime background crossfade 2 s.
  `prefers-reduced-motion`: no scale, no breathing animation (static circle + countdown), drain updates at 1 Hz.
- Density setting: *Comfortable* (default) / *Compact* (spacing × 0.8, timeline hour height 48→36 px).

---

## 2. Signature component: the Dial

```
viewBox 0 0 400 400, center (200,200)

        · · · · ╷ · · · ·            60 ticks on r = 186..194
     ·          │          ·         minute tick: length 7,  width 1.25, --text-3 @ 60 %
   ·      ╭─────┼─────╮      ·       5-min tick:  length 15, width 2.5,  --text-2
  ╷      ╱▓▓▓▓▓▓│      ╲      ╷      inner disc: r = 160, --bg-inset, inner shadow 0 0 0 1px --border
  ·     │▓▓▓▓▓▓▓●──────│      ·      sector: r = 156, from 12 o'clock, sweep = minutes × 6°,
  ·     │▓▓▓▓▓▓▓▓▓▓▓▓▓▓│      ·              radial gradient --cat → --cat-hi (center → edge)
   ·     ╲▓▓▓▓▓▓▓▓▓▓▓▓╱      ·       hand: center → r = 172, width 2.5, --cat, round caps
     ·    ╰───────────╯    ·         hub: r = 9, --text-1, ring 3 px --cat
        · · · · ╵ · · · ·            halo: soft --cat-glow behind sector at 20 %
```

- **Mapping**: 60 min = one full turn. Clockwise from 12 → focus (1–180 min; beyond 60, the sector stays full and
  a thin outer "second lap" arc at r = 178 shows the extra). Counter-clockwise → break (1–60 min), sector tinted
  `--brk`, label and button switch to "Start break".
- **Drag**: pointer angle is *unwrapped* (accumulated delta), so crossing 12 o'clock never jumps 59 → 0; crossing
  0 flips direction (focus ⇄ break). Snap to 1 min; stronger 5-min magnet (±0.6 min capture zone). Optional tick
  sound per detent (rate-limited).
- **Other inputs**: wheel (±1, Shift ±5), arrows (±1, Shift ±5), PageUp/Down (±5), Home/End (min/max), typing digits
  while the dial is focused opens an inline field: `45`, `1h30`, `90m`, `-10` (break).
- **Running**: sector drains continuously (rAF, derived from absolute timestamps), the hand points to remaining
  time; dragging the hand during a session extends/shortens it (scrubbing), with a ghost of the original end.
- **Overtime**: sector fades out (400 ms), the window background crossfades to `--overtime-bg` over 2 s, counter
  shows `+03:12` counting up, hub pulses once per minute (disabled with reduced motion).
- **A11y**: `role="slider"`, `aria-valuemin/max/now`, `aria-valuetext="45 minutes focus, ends at 16:27"`.

Below the dial: countdown `45:00` (mono 56) and the time pill `15:42 → 16:27` (live end time).

---

## 3. Component inventory

| Component | Notes |
|---|---|
| `TitleBar` | drag region; macOS native traffic lights left (overlay); Win/Linux min/max/close right; center: nothing (quiet). |
| `GoalDots` | `● ● ● ○ ○ ○ ○ ○`, filled dots in `--cat`, pop animation (scale 0.6→1.15→1) on validate. |
| `CategoryPill` | color dot + name + chevron; opens `CategoryMenu` (search, create inline, reorder, Ctrl/⌘+K). |
| `IntentField` | auto-growing textarea, placeholder "What are you focusing on?"; `SuggestionList` (intent + source icon + category dot). Tab = accept, Enter = start, ↑/↓ navigate. |
| `LastSessionLine` | "Last session: *Redesign onboarding* ↻". |
| `Dial`, `Countdown`, `TimePill` | see §2. |
| `PrimaryButton` | pill, uppercase mono label, `--cat` fill, glow; states: start / start break / pause / resume. |
| `StopMenu` | Finish session · Abandon (undo toast 5 s) · Start break. |
| `BottomToolbar` | pill group: analytics · mini player · settings. |
| `UndoToast` | bottom-center, countdown bar. |
| `MoodCard` | large card, emoji (Fluent Emoji, MIT, bundled SVG), label, arrow glyph, shortcut hint `1/2/3`. |
| `StatLine` | mono caps line with `·` separators and ↑ → ↓ mood glyphs. |
| `WeekStrip`, `SplitBar`, `Timeline`, `TimelineBlock`, `NowLine`, `FabAdd` | calendar panel. |
| `Sparkline`, `Barcode24h`, `KpiCard`, `StackedBars`, `Donut`, `Heatmap`, `StreakBadge` | analytics (visx). |
| `SettingsNav`, `SettingsRow`, `KeyRecorder`, `TestButton` | settings. |

---

## 4. Screens (ASCII wireframes, dark theme)

### 4.1 + 4.2 Main window — dual panel (home)

```
╭──────────────────────────────────────────────────────────────────────────────────────────────────╮
│ ● ● ●                                                                                     ─ □ ✕ │
├───────────────────────────────────────────┬──────────────────────────────────────────────────────┤
│                ● ● ● ○ ○ ○ ○ ○            │  ‹  MON 16  TUE 17  WED 18 [THU 19] FRI 20  SAT 21  SUN 22  › │
│                                           │                                                      │
│        What are you focusing on?          │  Thursday, November 19                               │
│                                           │  5 ↑ · 0 → · 0 ↓ · 4H 38M FOCUS                       │
│          ( ● Deep work      ⌄ )           │  ▕████████████▒▒▒▒▒▒▒▒▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░▏      │
│   ┌───────────────────────────────────┐   │                                                      │
│   │ Redesign the onboarding flow▏     │   │  08:00 ┊                                             │
│   └───────────────────────────────────┘   │  09:00 ┊ ┌──────────────────────────┐                │
│   Last session: Write release notes ↻     │        ┊ │ Redesign onboarding  ✎   │  ┌───────────┐ │
│                                           │  10:00 ┊ │                    52m   │  │ Standup   │ │
│              · · · ╷ · · ·                │        ┊ └──────────────────────────┘  └───────────┘ │
│           ·     ╭──┼──╮     ·             │        ┊ ┌ ─ break ─ 8m ┐                            │
│          ╷    ╱▓▓▓▓│   ╲    ╷             │  11:00 ┊ ┌──────────────────────────┐                │
│          ·   │▓▓▓▓▓●───│    ·             │        ┊ │╱╱╱ Review PR #42 ╱╱╱╱╱╱╱ │ (live, hatched)│
│           ·   ╲▓▓▓▓▓▓▓╱    ·              │ 11:24 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│              · · · ╵ · · ·                │  12:00 ┊                                             │
│                                           │  13:00 ┊                                             │
│                 45:00                     │                                                      │
│            ( 15:42 → 16:27 )              │                                                 ( + )│
│                                           │                                                      │
│        (((  START SESSION  )))            │                                                      │
│                                           │                                                      │
│           ( ▤   ▶   ⚙ )                   │                                                      │
╰───────────────────────────────────────────┴──────────────────────────────────────────────────────╯
```
- Left panel min width 380 px; right panel collapsible (chevron on the divider, `Ctrl/⌘+\`) → compact timer-only
  window. Window size, position, collapsed state persisted.
- Timeline blocks: session = `--cat` fill 85 %, intent top-left, duration mono bottom-right, ✎ if notes;
  break = `--cat-soft`; external events = neutral gray outline, their calendar color as a 3 px left bar (muted);
  live session = animated diagonal hatching (CSS `background-position` animation, paused with reduced motion).
- Interactions: drag block (15-min snap, Alt = 1 min), resize top/bottom edges, click = detail side sheet,
  click external event = "Start a session for this slot", "+" FAB = manual session.

**During a session** (left panel):
```
                ● ● ● ○ ○ ○ ○ ○
        Redesign the onboarding flow          ← title becomes the intention
              ( ● Deep work ⌄ )
              [ draining dial ]
                    27:14
              ( 15:42 → 16:27 )
          (( ❚❚ PAUSE ))   ( ■ )              ← ■ opens StopMenu
                                              ┌─────────────────────┐
                                              │ Finish session      │
                                              │ Abandon             │
                                              │ Start break         │
                                              └─────────────────────┘
```
**Overtime**: whole window background = `--overtime-bg`; dial sector gone; counter `+03:12`; buttons `☕` (break) and
`■` (stop working).

### Intention suggestions (reference 2)

```
   ┌───────────────────────────────────────────────┐
   │ Rede▏                                         │
   ├───────────────────────────────────────────────┤
   │ ↻ Redesign the onboarding flow          ● Deep│
   │ ↻ Redesign settings navigation          ● Deep│
   │ ☐ Refactor dial snapping   LINEAR       ● Code│
   │ ☐ Reply to design review   TODOIST      ● Comm│
   └───────────────────────────────────────────────┘
     Tab accept · Enter start · ↑↓ navigate
```

### 4.3 Reflection (reference 3)

```
                       Let's pause and reflect.
              What did you learn during this session?

     ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
     │      😔      │    │      😐      │    │      🤩      │
     │  Distracted  │    │   Neutral    │    │   Focused    │
     │      ↓    1  │    │      →    2  │    │      ↑    3  │
     └──────────────┘    └──────────────┘    └──────────────┘

     FOCUS / BREAK     CATEGORY            INTENTION
     111m / 4m         ( ● Deep work ⌄ )   Redesign the onboarding flow

     ┌────────────────────────────────────────────────────────────┐
     │ • Simplified step 2, merged name + goal                    │
     │ • Need copy review for FR                              ⧉   │
     └────────────────────────────────────────────────────────────┘
     ☐ Start break automatically

              ((( VALIDATE SESSION )))        auto-validate in 00:53
```

### 4.4 Break

```
                ● ● ● ● ○ ○ ○ ○
                   Break time
          Stand up, stretch, drink some water.
              [ dial with --brk sector ]
                      04:12
              ( 16:27 → 16:32 )
          (( ❚❚ PAUSE ))   ( ⏭ skip )
```
Long break every N sessions (default 4) uses the same screen with "Long break" and the longer default.
Break overtime: background `--brk-overtime-bg`, counter `+01:40`, notification "Your break went too long".
Optional "always on top" break window.

### 4.5 Journal & analytics (references 4 and 5)

```
╭────┬────────────────────────┬──────────────────────────────┬──────────────────────────────────────╮
│ ◷  │ 2026                   │  Calendar | [Timeline]        │  November 2026                       │
│DAY │ ┌────────────────────┐ │                              │  ┌────────────┐ ┌────────────┐       │
│    │ │ November   ╱╲_╱╲╱  │ │  THU 19                      │  │ TOTAL FOCUS│ │ AVG / DAY  │       │
│ ▦  │ │ 84 ↑·0 →·0 ↓·24H42M│ │  ▏▏▌▌█▌ ▏  ▌█▌▏    ▏▏        │  │ 24h 42m    │ │ 1h 14m     │       │
│WEEK│ └────────────────────┘ │  00    06    12    18    21   │  │ ▲ 259 %    │ │ ▲ 31 %     │       │
│    │ ┌────────────────────┐ │  5 ↑ · 4H 38M · "Simplified…" │  └────────────┘ └────────────┘       │
│ ▤  │ │ October   _╱╲__╱   │ │                              │  MOST PRODUCTIVE  Noon               │
│MON │ │ 61 ↑·3 →·2 ↓·18H05M│ │  WED 18                      │  MOST FOCUSED DAY Tuesday            │
│ TH │ └────────────────────┘ │  ▏ ▌▌█▌▌   ▌▌▏                │  TOTAL BREAK 3h 10m · RATIO 7.8:1    │
│    │ ┌────────────────────┐ │  4 ↑ · 1 → · 3H 50M           │  MOST DISTRACTED  Friday, 15:00      │
│    │ │ September ╱‾╲_     │ │                              │                                      │
│    │ └────────────────────┘ │                              │  🤩 84 Focused · 😐 0 · 😔 0           │
│    │                        │                              │  DAILY DISTRIBUTION                   │
│ ⌕  │ Filters: ● Deep ● Code │                              │   2h  3h     1h  4h                   │
│    │ mood ▾ type ▾ ✎ notes  │                              │   ▆   █  ▃   ▂   █  ▅ ...  (stacked)  │
│    │                        │                              │  BY CATEGORY   ◕ donut + list         │
│    │                        │                              │  HEATMAP  weekday × hour              │
│    │                        │                              │  STREAK  12 days                      │
╰────┴────────────────────────┴──────────────────────────────┴──────────────────────────────────────╯
```
- Barcode 24 h: 1 bar per 15 min slot, colored by dominant category, opacity = share of the slot focused.
- Deltas: green ▲ / red ▼ vs previous period of equal length; "—" when no previous data.
- Day view: list of sessions `23:52 → 01:06 · 74m` with notes rendered on the right side.

### 4.6 Mini player (260 × 56)

```
╭──────────────────────────────────────────────╮
│ ●  Redesign the onboarding…   27:14  ◔  ❚❚ ■ │
╰──────────────────────────────────────────────╯
```
Always on top, draggable, radius 14, adjustable opacity (40–100 %), double-click → main window.

### 4.7 Tray / menu bar

```
 macOS menu bar:  ◔ 27:14        Windows tray: ring icon in --cat, tooltip "Redesign… — 27 min left"

 ┌──────────────────────────────────┐
 │ ● Deep work — 27:14 left          │
 │   Redesign the onboarding flow    │
 ├──────────────────────────────────┤
 │ Pause                     ⌥⌘P     │
 │ Finish session            ⌥⌘F     │
 │ Start break               ⌥⌘B     │
 │ Start previous session            │
 ├──────────────────────────────────┤
 │ Open Momentum · Mini player       │
 │ Settings…                         │
 │ Quit                              │
 └──────────────────────────────────┘
```

### 4.8 Settings

```
╭───────────────────────┬──────────────────────────────────────────────────╮
│ General               │  Timer                                           │
│ Profile               │  ─────────────────────────────────────────────── │
│▸Timer                 │  Default focus          [ 45 min ]               │
│ Sounds & notifications│  Default break          [ 15 min ]               │
│ Ambient sounds        │  Long break every       [ 4 ] sessions → [15 min]│
│ Categories            │  Breathing before focus [●] 1 cycle ▾            │
│ Calendars             │  Auto-validate reflection [●] after [60 s]       │
│ Website blocker       │  Start break automatically [ ]                   │
│ App blocker           │                                                  │
│ Slack · To-do         │                                                  │
│ Automations           │                                                  │
│ Keyboard shortcuts    │                                                  │
│ Menu bar & Dock       │                                                  │
│ Window · Appearance   │                                                  │
│ Language              │                                                  │
│ Window tracker        │                                                  │
│ Data · Developer      │                                                  │
│ About                 │                                                  │
╰───────────────────────┴──────────────────────────────────────────────────╯
```
Every section has a deep link `momentum://settings/<section>` (slugs: `general`, `profile`, `timer`, `sounds`,
`ambient`, `categories`, `calendars`, `site-blocker`, `app-blocker`, `slack`, `todo`, `automations`,
`shortcuts`, `menubar`, `window`, `appearance`, `language`, `window-tracker`, `data`, `developer`, `about`).

### 4.9 Onboarding (≤ 4 screens, skippable)

```
 1  Welcome            2  Categories             3  Rhythm                  4  Extras (optional)
 ┌────────────────┐    ┌────────────────────┐    ┌──────────────────────┐   ┌──────────────────────┐
 │ Hi! What should│    │ Pick 2–3 to start  │    │ ( 25 / 5 )           │   │ [ ] Notifications    │
 │ we call you?   │    │ [● Deep work]      │    │ ( 45 / 15 ) ◀        │   │ [ ] Global shortcut  │
 │ [ Sam       ]  │    │ [● Meetings ]      │    │ ( custom )           │   │ [ ] Browser extension│
 │ Daily goal     │    │ [● Learning ]      │    │ There's no perfect   │   │                      │
 │ ● ● ● ● ○ ○    │    │ [+ new]            │    │ setting — adjust     │   │   ((( LET'S GO )))   │
 │ [ Continue ]   │    │ [ Continue ]       │    │ anytime.             │   │                      │
 └────────────────┘    └────────────────────┘    └──────────────────────┘   └──────────────────────┘
```
"Skip" on every screen. Lands on the timer with the cursor in the intention field; Enter starts.

### Blocked page (browser extension, reference 6)

```
                                FOCUSING ON
                    Redesign the onboarding flow
                         [ animated mini dial ]
                              27:14 left
          This tab will be restored when your session ends.
                     ( Unblock for 5 min — marks session distracted )
```
Full-bleed `--bg-app` with a faint radial `--cat-glow` behind the dial.

---

## 5. Iconography & brand (to finalize in phase 8)

- App icon: an original mark — a quarter-filled dial sector inside a rounded square, coral on graphite.
  Rendered from a single SVG source to all platform sizes (`tauri icon`).
- UI icons: Lucide, 1.5 px stroke, 16/20 px.
- Emoji for moods: Fluent Emoji flat SVGs (MIT), bundled for identical rendering on all OSes.
