# Obsidian Server Panel - UX Style Guide

## Overview

This style guide defines the design system for the Obsidian Server Panel, a Minecraft server management application. The design emphasizes accessibility, gaming aesthetics, and functional clarity while maintaining a dark theme optimized for server administrators.

## Design Philosophy

- **Gaming-First**: Minecraft-inspired typography and iconography
- **Accessibility-First**: Multiple colorblind-friendly themes and high contrast ratios
- **Minimalist**: Clean, functional interfaces without unnecessary decoration
- **Consistent**: Standardized spacing, sizing, and interaction patterns

## Typography

### Font Stack

#### Primary Fonts
- **Headers**: `MinecraftHeader` (Minecraft-Tenv2.woff2)
- **Body Text**: `MinecraftBody` (Minecraft-Seven_v2.woff2)
- **Fallback**: `Noto Sans`, `sans-serif`

#### Usage Guidelines
```css
/* Headers (h1-h6) */
font-family: 'MinecraftHeader', sans-serif;
/* Applied via: font-minecraft-header */

/* Body text, buttons, navigation */
font-family: 'MinecraftBody', sans-serif;
/* Applied via: font-minecraft-body */
```

### Typography Classes
- `.font-minecraft-header` - For headings and brand elements
- `.font-minecraft-body` - For UI components and body text
- `.text-tiny` - Small secondary text (footer, metadata)

## Color System

### Theme Architecture
The application uses HeroUI's theming system with custom accessibility-focused themes:

#### Default Dark Theme
```css
--heroui-primary: #1bd96a (Green)
--heroui-primary-foreground: #000000
--heroui-secondary: #eaeaea
--heroui-background: #0b0b0e
```

#### Accessibility Themes

**Deuteranopia Friendly** (Red-Green Colorblind)
- Primary: `#0ea5e9` (Blue)
- Secondary: `#6366f1` (Purple)
- Success: `#0ea5e9` (Blue instead of green)
- Danger: `#8b5cf6` (Purple instead of red)

**Tritanopia Friendly** (Blue-Yellow Colorblind)
- Primary: `#1bd96a` (Green)
- Secondary: `#06b6d4` (Cyan)

**Monochrome** (Maximum Accessibility)
- Primary: `#ffffff` (White)
- Secondary: `#666666` (Gray)
- Grayscale palette for all status colors

### Color Usage Guidelines
- **Primary Green**: Brand elements, active states, success indicators
- **Foreground**: Always ensure proper contrast ratios
- **Hover States**: Use `hover:bg-default/40` for subtle interactions
- **Active States**: Use `data-[active=true]:text-primary` pattern

## Layout & Spacing

### Grid System
- **Container Padding**: `px-8` (32px horizontal padding)
- **Component Gaps**: `gap-4` (16px), `gap-2` (8px), `gap-1` (4px)
- **Section Margins**: `mt-4` (16px top margin)

### Responsive Breakpoints
- **Mobile First**: Base styles target mobile
- **Desktop Enhanced**: Additional features for larger screens
- **Graph Sizing**: `sm | md | lg | fullWidth` options for data visualization

## Components

### Buttons

#### Base Button Component
```tsx
<Button radius="none" className="font-minecraft-body">
```

#### Variants
- **Primary**: Default styling with primary color
- **Light**: Transparent background with primary text
- **Ghost**: Subtle background on hover
- **Icon Only**: Square buttons for actions (`isIconOnly`)

#### Sizing
- **Default**: Standard touch target
- **Small**: `size="sm"` for compact spaces
- **Icon**: 16px icons in icon-only buttons

### Form Controls

#### Input Fields
```tsx
<Input radius="none" />
```
- **No Border Radius**: All form controls use sharp corners
- **Consistent Styling**: Inherits theme colors automatically

#### Selects & Dropdowns
```tsx
<Select radius="none">
  <SelectItem key="value" textValue="value">Label</SelectItem>
</Select>
```

### Navigation

#### Navbar Structure
- **Brand**: Left-aligned with `font-minecraft-header` and primary color
- **Center**: Main navigation items with hover states
- **Right**: User actions and notifications

#### Navigation Items
```css
/* Hover state */
.hover:bg-default/40 py-2 px-4 transition-background duration-200

/* Active state */
.data-[active=true]:text-primary
```

### Cards & Containers

#### Card Component Pattern
```tsx
<Card radius="none" className="font-minecraft-body">
  <CardBody>
    {/* Content */}
  </CardBody>
</Card>
```

#### Server Items
- **Hover Effect**: `hover:bg-gray-600/20 transition-all duration-200`
- **Icon Integration**: Small server icons (16px) with consistent spacing
- **Action Buttons**: Ghost variant with danger color for destructive actions

### Data Visualization

#### Resource Graphs
- **Size Variants**: `sm | md | lg | fullWidth`
- **Responsive**: Charts adapt to container size
- **Color Coding**: Uses theme-appropriate colors for data series
- **Tooltips**: Consistent formatting with theme colors

## Iconography

### Icon Library
- **Primary**: Pixelart Icons (`pixelarticons:*`)
- **Secondary**: Pixel Icons (`pixel:*`)
- **Style**: 8-bit/pixel art aesthetic matching Minecraft theme

### Common Icons
- `pixelarticons:home` - Home/Dashboard
- `pixelarticons:user` - User account
- `pixelarticons:users` - User management
- `pixelarticons:sliders` - Settings
- `pixelarticons:edit` - Edit actions
- `pixel:home-solid` - Alternative home icon

### Icon Sizing
- **Standard**: 16px width for most UI icons
- **Navigation**: Default size for navbar icons
- **Tiny**: Server icons in lists

## Animation & Transitions

### Motion Patterns
```tsx
// Page transitions
initial={{ opacity: 0, y: 20 }}
animate={{ opacity: 1, y: 0 }}
transition={{ duration: 0.3 }}

// Staggered navigation items
transition={{ duration: 0.2, delay: 0.1 }} // Increasing delay per item
```

### Hover Transitions
```css
transition-background duration-200
transition-all duration-200
```

### Loading States
- **Fade In**: Components fade in with slight upward motion
- **Staggered**: Navigation items appear with sequential delays

## Accessibility Features

### Color Accessibility
- **Multiple Themes**: Four accessibility-focused color themes
- **High Contrast**: Monochrome option for maximum readability
- **Colorblind Support**: Specific themes for different types of color vision deficiency

### Keyboard Navigation
- **Focus States**: All interactive elements support keyboard focus
- **Tab Order**: Logical tab progression through interfaces
- **Shortcuts**: Arrow key navigation where appropriate

### Screen Readers
- **Semantic HTML**: Proper heading hierarchy and landmarks
- **ARIA Labels**: Descriptive labels for complex interactions
- **Tooltips**: Additional context for icon-only buttons

## Interactive States

### Hover States
```css
/* Subtle background */
.hover:bg-default/40

/* Server list items */
.hover:bg-gray-600/20
```

### Active States
```css
/* Navigation active */
.data-[active=true]:text-primary

/* Focus states */
/* Automatically handled by HeroUI components */
```

### Loading States
- **Skeleton Loading**: For data-heavy components
- **Fade Transitions**: Smooth content appearance
- **Progress Indicators**: For long-running operations

## Code Patterns

### Component Structure
```tsx
// Extended components follow this pattern
export const ComponentName = forwardRef<HTMLElement, Props>((props, ref) => {
    return <OriginalComponent radius="none" className="font-minecraft-body" {...props} ref={ref} />;
});
```

### Theme Usage
```tsx
// Theme switching
const { theme, setTheme } = useTheme();
setTheme("deuteranopia-friendly" as Themes);
```

### Animation Integration
```tsx
<AnimatePresence>
  <motion.div
    initial={{ opacity: 0, y: 20 }}
    animate={{ opacity: 1, y: 0 }}
    transition={{ duration: 0.3 }}
  >
    {content}
  </motion.div>
</AnimatePresence>
```

## Best Practices

### Do's
✅ Use `radius="none"` on all HeroUI components for consistency
✅ Apply `font-minecraft-body` to interactive elements
✅ Include hover states with subtle opacity changes
✅ Provide tooltips for icon-only buttons
✅ Test with all accessibility themes
✅ Use pixel art icons for consistency
✅ Implement proper loading states

### Don'ts
❌ Don't use rounded corners (`radius` should be "none")
❌ Don't use colors outside the defined theme system
❌ Don't forget keyboard navigation support
❌ Don't use non-pixel art icons in the main interface
❌ Don't implement custom animations without considering accessibility
❌ Don't hardcode colors (use theme variables)

## File Organization

### Component Structure
```
src/components/
├── extended/          # Wrapped HeroUI components
├── authentication/    # Auth-specific components
├── server-components/ # Server management UI
├── navigation/        # Navigation components
└── icons/            # Custom SVG icons
```

### Styling Files
```
src/css/
├── index.css         # Global styles & Tailwind imports
├── minecraft-font.css # Font face definitions
└── *.css            # Feature-specific styles
```

## Maintenance Guidelines

### Theme Updates
- All theme modifications must maintain accessibility contrast ratios
- New colors should be tested with colorblind simulation tools
- Theme changes require updates to all accessibility variants

### Component Updates
- Extended components should maintain the `radius="none"` pattern
- New components should follow the established font and spacing patterns
- Interactive elements must include proper hover and focus states

### Testing Checklist
- [ ] Component works in all accessibility themes
- [ ] Proper keyboard navigation
- [ ] Appropriate hover and focus states
- [ ] Icons are pixel art style
- [ ] Typography uses correct font families
- [ ] Animations respect user preferences
- [ ] Mobile responsiveness maintained
