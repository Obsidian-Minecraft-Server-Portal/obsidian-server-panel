# Installer UI Structure

This directory contains the modular Slint UI components for the Obsidian Installer.

## File Organization

### Main Application
- **`app.slint`** - Main application window, page routing, and state management (231 lines)

### Common Components & Theme
- **`common.slint`** - Shared components and theme definitions (197 lines)
  - `Theme` - Global color palette following Obsidian Style Guide
  - `CustomButton` - Styled button with primary/secondary variants
  - `TitleBar` - Custom window title bar with controls
  - `FeatureItem` - List item with pixel-style bullet point

### Page Components
- **`welcome_page.slint`** - Welcome screen with logo and features (106 lines)
- **`terms_page.slint`** - Terms of Service acceptance page (76 lines)
- **`location_page.slint`** - Installation path selection (97 lines)
- **`options_page.slint`** - Service and startup configuration (101 lines)
- **`installing_page.slint`** - Progress tracking during installation (138 lines)
- **`complete_page.slint`** - Installation completion screen (88 lines)

## Architecture

### Component Hierarchy
```
app.slint (Main Window)
├── common.slint
│   ├── Theme (global colors)
│   ├── TitleBar (window chrome)
│   ├── CustomButton (reusable button)
│   └── FeatureItem (list item)
├── welcome_page.slint → uses FeatureItem, CustomButton
├── terms_page.slint → uses CustomButton
├── location_page.slint → uses CustomButton
├── options_page.slint → uses CustomButton
├── installing_page.slint → uses Theme
└── complete_page.slint → uses CustomButton
```

### Page Flow
```
Welcome → Terms → Location → Options → Installing → Complete
   ↓        ↓         ↓          ↓          ↓           ↓
 Get      Accept    Choose     Config    Progress   Launch/
Started    TOS      Path      Options   Tracking    Exit
```

## Style Guide Compliance

All components follow the [Obsidian Style Guide](../../../style-guide.md):

- ✅ Primary color: `#1bd96a` (Obsidian green)
- ✅ Sharp corners (`border-radius: 0px`)
- ✅ Dark theme (`#0b0b0e` background)
- ✅ Bold typography for headers (font-weight: 900)
- ✅ Pixel-aesthetic UI elements
- ✅ Smooth page transitions (300ms fade)

## Animations

### Page Transitions
- **Fade in/out**: 300ms with ease-in-out easing
- **Welcome logo**: 800ms fade-in on page load
- **Success icon**: 600ms fade-in with 200ms delay

### Interactive States
- **Button hover**: Color change with smooth transition
- **Progress bar**: Width animates over 300ms

## Properties & Callbacks

### App Properties (in app.slint)
```slint
in-out property <image> app-icon;
in-out property <Page> current-page;
in-out property <string> tos-content;
in-out property <bool> tos-accepted;
in-out property <string> install-path;
in-out property <bool> install-as-service;
in-out property <bool> start-with-windows;
in-out property <string> install-status;
in-out property <float> install-progress;
in-out property <bool> install-success;
in-out property <string> complete-message;
```

### App Callbacks
```slint
callback request-exit-app;
callback minimize-window;
callback toggle-maximize-window;
callback drag-window(length, length);
callback browse-folder;
callback start-installation;
callback launch-app;
```

## Modifying the UI

### Adding a New Page
1. Create `new_page.slint` in this directory
2. Import necessary components from `common.slint`
3. Define page component with required callbacks
4. Add page to `Page` enum in `app.slint`
5. Import page in `app.slint`
6. Add routing logic in main content area

### Customizing Theme
Edit `common.slint` → `Theme` global to change colors:
```slint
global Theme {
    in-out property <color> accent: #1bd96a;  // Change primary color
    in-out property <color> bg-primary: #0b0b0e;  // Change background
    // ... other theme properties
}
```

### Adding Reusable Components
1. Add component to `common.slint`
2. Export with `export component MyComponent { ... }`
3. Import in pages: `import { MyComponent } from "common.slint";`

## Building

The UI is compiled during the build process via `slint-build` in `build.rs`.

```bash
cargo build -p obsidian-installer
```

## Font Support

The UI is designed for Minecraft pixel fonts but uses system fonts as fallback:
- **Primary**: MinecraftBody (if installed system-wide)
- **Fallback**: Segoe UI

See `../res/fonts/README.md` for font installation instructions.

## Total Lines of Code
- **Total UI**: ~1,034 lines (down from 955 in monolithic file)
- **Modular Structure**: 8 files
- **Average per file**: ~129 lines
- **Largest file**: `app.slint` (231 lines)
- **Smallest file**: `terms_page.slint` (76 lines)

## Benefits of Modular Structure

1. **Maintainability**: Each page is self-contained and easy to modify
2. **Reusability**: Common components shared across pages
3. **Clarity**: Clear separation of concerns
4. **Scalability**: Easy to add new pages or components
5. **Collaboration**: Multiple developers can work on different pages
6. **Testing**: Individual pages can be tested in isolation
