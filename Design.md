---
name: Kavus Professional HUD
colors:
  surface: '#141314'
  surface-dim: '#141314'
  surface-bright: '#3a3939'
  surface-container-lowest: '#0e0e0e'
  surface-container-low: '#1c1b1c'
  surface-container: '#201f20'
  surface-container-high: '#2a2a2a'
  surface-container-highest: '#353435'
  on-surface: '#e5e2e1'
  on-surface-variant: '#c6c6cb'
  inverse-surface: '#e5e2e1'
  inverse-on-surface: '#313030'
  outline: '#909095'
  outline-variant: '#45474b'
  surface-tint: '#c6c6cc'
  primary: '#c6c6cc'
  on-primary: '#2f3035'
  primary-container: '#0f1115'
  on-primary-container: '#7b7c82'
  inverse-primary: '#5d5e63'
  secondary: '#c3c6d0'
  on-secondary: '#2d3138'
  secondary-container: '#43474f'
  on-secondary-container: '#b2b5be'
  tertiary: '#d1c4bb'
  on-tertiary: '#372f29'
  tertiary-container: '#16100b'
  on-tertiary-container: '#857b73'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#e2e2e8'
  primary-fixed-dim: '#c6c6cc'
  on-primary-fixed: '#1a1c20'
  on-primary-fixed-variant: '#45474b'
  secondary-fixed: '#dfe2ec'
  secondary-fixed-dim: '#c3c6d0'
  on-secondary-fixed: '#181c23'
  on-secondary-fixed-variant: '#43474f'
  tertiary-fixed: '#eee0d7'
  tertiary-fixed-dim: '#d1c4bb'
  on-tertiary-fixed: '#211a15'
  on-tertiary-fixed-variant: '#4e453f'
  background: '#141314'
  on-background: '#e5e2e1'
  surface-variant: '#353435'
typography:
  display-mono:
    fontFamily: JetBrains Mono
    fontSize: 14px
    fontWeight: '500'
    lineHeight: 20px
    letterSpacing: -0.01em
  code-sm:
    fontFamily: JetBrains Mono
    fontSize: 12px
    fontWeight: '400'
    lineHeight: 18px
  ui-header:
    fontFamily: Geist
    fontSize: 13px
    fontWeight: '600'
    lineHeight: 16px
  ui-body:
    fontFamily: Geist
    fontSize: 13px
    fontWeight: '400'
    lineHeight: 18px
  ui-label-xs:
    fontFamily: JetBrains Mono
    fontSize: 10px
    fontWeight: '500'
    lineHeight: 12px
rounded:
  sm: 0.125rem
  DEFAULT: 0.25rem
  md: 0.375rem
  lg: 0.5rem
  xl: 0.75rem
  full: 9999px
spacing:
  unit: 4px
  panel-gap: 1px
  sidebar-width-min: 48px
  sidebar-width-max: 280px
  container-padding: 12px
  element-gap: 8px
---

## Brand & Style

The design system is engineered for high-performance AI orchestration. The brand personality is **Technical, Precise, and Authoritative**, positioning the product as a mission-critical instrument rather than a consumer application. The target audience consists of senior software engineers and AI researchers who require high information density and low visual latency.

The design style is **Industrial Minimalism**. It draws heavily from high-end IDEs like Zed and terminal emulators, utilizing a "Chrome-less" philosophy where the UI recedes to prioritize code and data. The aesthetic is defined by:
- **Structural Rigidity:** Strict adherence to a grid with zero unnecessary ornamentation.
- **Functional Density:** Maximizing screen real estate through compact controls and collapsed states.
- **Tactile Precision:** Interaction feedback that mimics professional hardware—fast, sharp, and predictable.

## Colors

The palette is optimized for long-duration focus in low-light environments. 

- **The Foundation:** A deep **Slate/Carbon** base (#0F1115) serves as the primary surface, providing a near-black canvas that reduces eye strain.
- **Interactive Accents:** **Electric Blue** (#00E5FF) is reserved for active states, focus rings, and primary action indicators. **Cyberpunk Amber** (#FFB86C) is used sparingly for AI-generated suggestions or "cautionary" interactive elements.
- **Syntax System:** Based on Monokai Pro, the palette uses high-vibrancy, distinct hues for rapid code scanning. Each color is calibrated to maintain a contrast ratio of at least 4.5:1 against the Carbon background.
- **Neutrals:** Grays are strictly neutral (no blue/warm tints) to ensure they remain subordinate to the syntax and accent colors.

## Typography

Typography in this design system is divided into two functional roles: **System UI** and **Data/Code.**

1.  **System UI (Geist):** Used for navigation, menus, and high-level structural labels. It provides a clean, neutral sans-serif counterpoint to the technical data.
2.  **Data/Code (JetBrains Mono):** Used for code editors, terminal outputs, HUD metrics, and any AI-generated logs. The monospaced nature ensures perfect vertical alignment of characters, critical for scanning logs and multi-line AI diffs.

All sizes are intentionally small to support high-density layouts. Hierarchy is established through weight shifts (Medium to Semibold) and case transformations (Uppercase for category headers) rather than significant size increases.

## Layout & Spacing

This design system employs a **Fixed-Grid Panel System** inspired by IDE layouts. The screen is treated as a modular workspace divided by 1px borders rather than margins.

- **The 4px Rhythm:** All spacing (padding, gaps, margins) must be a multiple of 4px.
- **Panel Layout:** The interface is composed of a Central Viewport (Editor/Canvas) flanked by collapsible Sidebars (File tree, Agent Chat) and a Bottom Tray (Terminal/Logs).
- **Gutterless Alignment:** Panels should "butt" against each other, separated only by a 1px `border-subtle`. This creates a seamless, integrated tool feel.
- **Density:** Padding inside components is minimal (8px or 12px) to ensure that the maximum amount of information is visible without scrolling.

## Elevation & Depth

Depth is conveyed through **Tonal Layering** and **Border Contours** rather than traditional shadows.

- **Base Layer:** The darkest shade (#0F1115) is used for the background and "void" areas.
- **In-set Surfaces:** Panels and input fields use a slightly lighter shade (#1E2229) to appear "raised" or distinct.
- **Flat Elevation:** No drop shadows are used for UI panels. Instead, active panels are highlighted with a 1px accent border or a subtle interior glow.
- **Overlays:** Modals or floating command palettes use a 1px solid border (#3E4451) and a heavy backdrop blur (20px) to distinguish them from the workspace, creating a "glass-on-metal" effect without becoming decorative.

## Shapes

The shape language is **Strictly Geometric.**

- **Corner Radius:** A universal 2px-4px radius is applied to buttons and panels to prevent the UI from feeling "sharp" or dangerous, while maintaining a professional, engineered look.
- **Interactive Elements:** Buttons and tags should be rectangular with minimal softening.
- **Nodes:** For node-based workflows, use hard-edged rectangles with 45-degree chamfered corners on active ports to emphasize the industrial aesthetic.

## Components

- **Buttons:** Small, height-constrained (24px or 28px). Ghost style by default with a solid Cyan fill on hover. Use Monospaced font for button labels to maintain the technical aesthetic.
- **Input Fields:** Flush with the background, defined by a bottom border or a subtle 1px frame. Focus state uses the primary Electric Blue accent with a 0px spread glow.
- **Chips/Badges:** Monospaced text inside a subtle bordered box. Use syntax colors (e.g., green for "Running", red for "Error", purple for "AI Thinking").
- **Terminal:** True black background with a 10% opacity Cyberpunk Amber overlay for "active AI" logs. No padding between lines of text.
- **Sidebars:** Collapsible to an icon-only state (48px width). Icons are 16px, line-weight based, and strictly monochromatic unless active.
- **Cards/Modules:** Used within the HUD to group AI metrics. They should have a 1px border and a header bar with a different background tone to clearly define the boundary of the module.