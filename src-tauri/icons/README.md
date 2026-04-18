# Icons

Drop the following files here before building. Tauri requires all of them:

- `32x32.png`
- `128x128.png`
- `128x128@2x.png` (256x256)
- `icon.icns` (macOS bundle)
- `icon.ico` (Windows bundle)
- `tray.png` (16x16 or 32x32, monochrome with alpha for `iconAsTemplate: true`)

Generate from a single 1024x1024 source PNG:

```bash
npm run tauri icon path/to/macrospend-logo.png
```

This creates every required size automatically.
