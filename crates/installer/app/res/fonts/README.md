# Minecraft Fonts

This directory contains the custom Minecraft-inspired fonts used in the Obsidian Installer, following the [Obsidian Style Guide](../../../style-guide.md).

## Font Files

### WOFF2 (Original)
- **Minecraft-Tenv2.woff2** - Headers and brand elements
- **Minecraft-Seven_v2.woff2** - Body text, buttons, and UI components

### TTF (Required for Embedding)
- **Minecraft-Tenv2.ttf** - Converted header font (create this)
- **Minecraft-Seven_v2.ttf** - Converted body font (create this)

## ⚠️ IMPORTANT: Font Conversion Required

The installer embeds fonts using Windows GDI, which **requires TrueType (TTF)** format. The WOFF2 files must be converted to TTF for the fonts to work in the standalone application.

## Converting WOFF2 to TTF

### Option 1: Using woff2 Command-Line Tool

Install and use Google's woff2 tools:

```bash
# Install woff2-tools (Linux/macOS)
# Ubuntu/Debian:
sudo apt-get install woff2

# macOS:
brew install woff2

# Convert fonts
woff2_decompress Minecraft-Tenv2.woff2
woff2_decompress Minecraft-Seven_v2.woff2
```

### Option 2: Using FontForge

1. Download [FontForge](https://fontforge.org/)
2. Open each WOFF2 file
3. Go to **File → Generate Fonts**
4. Select **TrueType** format
5. Save with `.ttf` extension

### Option 3: Online Converter

Use a trusted online converter:
- [CloudConvert](https://cloudconvert.com/woff2-to-ttf)
- [FontSquirrel Webfont Generator](https://www.fontsquirrel.com/tools/webfont-generator) (upload WOFF2, generate TTF)
- [Convertio](https://convertio.co/woff2-ttf/)

**Steps:**
1. Upload `Minecraft-Tenv2.woff2` → Convert to TTF
2. Upload `Minecraft-Seven_v2.woff2` → Convert to TTF
3. Download and place TTF files in this directory
4. Rebuild the application

### Option 4: Using Python (fonttools)

```bash
# Install fonttools
pip install fonttools brotli

# Convert fonts
pyftsubset Minecraft-Tenv2.woff2 --output-file=Minecraft-Tenv2.ttf --flavor=truetype
pyftsubset Minecraft-Seven_v2.woff2 --output-file=Minecraft-Seven_v2.ttf --flavor=truetype
```

## Updating the Code

Once you have the TTF files, update `app/src/fonts.rs` to load them:

```rust
// Change from WOFF2 to TTF
const MINECRAFT_BODY_FONT: &[u8] = include_bytes!("../res/fonts/Minecraft-Seven_v2.ttf");
const MINECRAFT_HEADER_FONT: &[u8] = include_bytes!("../res/fonts/Minecraft-Tenv2.ttf");
```

## Font Family Names

After conversion, the fonts will be registered with these family names:
- **Body**: `Minecraft` (or the name embedded in the font)
- **Header**: `MinecraftTen` (or the name embedded in the font)

To find the exact font family name:
```bash
# Using fc-query (Linux/macOS)
fc-query -f '%{family}\n' Minecraft-Seven_v2.ttf

# Using PowerShell (Windows)
Add-Type -AssemblyName PresentationCore
[Windows.Media.GlyphTypeface]::new("Minecraft-Seven_v2.ttf").FamilyNames
```

## Verifying Embedded Fonts

After building with TTF fonts:

1. Run the installer: `cargo run -p obsidian-installer`
2. Check the debug output for: `"Successfully registered Minecraft body font"`
3. The UI should display with pixel-style fonts
4. If you see "Failed to register" warnings, the fonts are still WOFF2 format

## Fallback Behavior

If fonts fail to load:
- **Windows**: Falls back to Segoe UI (system default)
- **Linux/macOS**: Falls back to system sans-serif font

The application will still function correctly, but won't have the pixel aesthetic.

## File Structure

```
app/res/fonts/
├── Minecraft-Tenv2.woff2       # Original header font (web format)
├── Minecraft-Seven_v2.woff2    # Original body font (web format)
├── Minecraft-Tenv2.ttf         # Converted header font (REQUIRED for embedding)
├── Minecraft-Seven_v2.ttf      # Converted body font (REQUIRED for embedding)
└── README.md                   # This file
```

## License

These fonts are included for use with the Obsidian Minecraft Server Panel. Please ensure compliance with the original font licenses.

## Style Guide Compliance

- **Headers**: MinecraftTen (Minecraft-Tenv2) - Bold, large, with letter-spacing
- **Body**: Minecraft (Minecraft-Seven) - Medium weight for readability
- **Fallback**: System fonts maintained for accessibility

For more details, see the [Style Guide](../../../style-guide.md).

## Troubleshooting

### Fonts Don't Appear in Application

**Cause**: WOFF2 files haven't been converted to TTF

**Solution**: Follow conversion steps above and rebuild

### Build Fails with "file not found"

**Cause**: TTF files are referenced in code but don't exist

**Solution**: Either convert WOFF2 to TTF, or temporarily comment out TTF loading in `fonts.rs`

### Fonts Look Blurry or Wrong

**Cause**: Font hinting or scaling issues

**Solution**: Try different conversion tools, or adjust font rendering in the UI

## Quick Start (TL;DR)

```bash
# 1. Convert fonts (pick one method)
woff2_decompress *.woff2

# 2. Update code to reference .ttf instead of .woff2
# Edit app/src/fonts.rs and change file extensions

# 3. Rebuild
cargo build --release -p obsidian-installer

# 4. Run and verify
cargo run -p obsidian-installer
```
