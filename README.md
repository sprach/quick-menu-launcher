# QikMenu (quick-menu-launcher)

**English** | [한국어](README_KO.md) | [日本語](README_JA.md)

A lightweight Windows System Tray application written in Rust. It allows you to launch applications and websites quickly from a customizable menu defined in an INI file.

## Background
This app was created to streamline specific workflows and improve accessibility:
- **Obsidian Vault Management**: I needed a convenient way to switch between different Obsidian Vaults depending on the context.
- **Portability**: While I normally use a Stream Deck for shortcuts, I wanted a software-only solution accessible from the System Tray for times when carrying the hardware device isn't possible.
- **Grouped Taskbar Management**: I wanted to keep using the standard Windows Taskbar but group specific apps together for cleaner access and selection.
- **Extensibility**: While implementing this for Obsidian, I expanded it to support launching any frequently used application or URL.

## Download
Download the latest version from the link below:
- [QikMenu Download Folder](refs/downloads/)

## Installation & Execution
1. Unzip the downloaded `7z` file to a desired folder.
2. Ensure `QikMenu.exe` is present.
3. Ensure `QikMenu.ini` configuration file is in the same folder.
4. Double-click `QikMenu.exe` to run.

## Usage
Once running, an icon will appear in the system tray area (bottom right) of the Windows taskbar.

![Tray Menu Screenshot](refs/cap-launchTrayMenu.jpg)

1.  **Open Menu**: Click (Left or Right) the tray icon to reveal the menu as shown above.
2.  **Launch App**: Click any item in the list to launch the corresponding application or website.
3.  **Edit Configuration**:
    - Click **"Edit Environment"** at the bottom of the menu.
    - The `QikMenu.ini` file will open in your default text editor.
    - Add or modify items as needed (see Configuration section below).
    - Save and close the file.
4.  **Reload Config**:
    - After saving changes, click **"Reload Config"** in the menu.
    - The menu will update immediately without restarting the application.
5.  **Exit**: Click **"Exit"** to terminate the application.

## Auto-start with Windows
To run this application automatically when Windows starts:

1. Press `Win` + `R` to open the **Run** dialog.
2. Type `shell:startup` and press Enter. This opens the **Startup** folder.
3. Create a **Shortcut** of `QikMenu.exe`.
4. Copy or move this **Shortcut** into the **Startup** folder you just opened.
5. The application will now launch automatically on startup.

## Configuration (`QikMenu.ini`)
```ini
[global]
# Available locales: ko (Korean), en (English), ja (Japanese)
locale=en

[apps]
# format: Label=Command
# Obsidian
Obsidian MyVault1=obsidian://open/?vault=MyWorks1
Obsidian MyVault2=obsidian://open/?vault=MyWorks2
Obsidian MyVault3=obsidian://open/?vault=MyWorks3
# Others
Command Prompt=cmd
Google=https://google.com
Notepad=notepad.exe
```
