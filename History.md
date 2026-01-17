# Version History

## 260117a
- **Hotkey Parsing Optimization**: Replaced manual `match` (switch) statements for alpha-numeric characters with calculation-based keycode mapping for improved efficiency and code maintainability.
- **Maintenance**: Added release binaries and updated version metadata.

 
## 251223a
- **Enhanced Configuration**: Renamed `short_key` to `hotkey` for clarity.
- **Documentation**: Added detailed 'Menu Invocation Method' section (Tray Icon & Hotkey).


## 251221a
- **Global Hotkey Support**: Added ability to trigger the menu using a configurable hotkey (e.g., `[Alt]+/`) defined in `QikMenu.ini`.

## 251215a
- **Enhanced Command Parsing**: Improvements to handle file or folder paths containing spaces by enclosing them in quotes (e.g., `"C:\My Folder\App.exe"`).

## 251208b
- **Initial Release**: First stable release of QikMenu.
