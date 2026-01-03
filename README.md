# Factorio Updater

Factorio Updater is a simple TUI tool written in [Ratatui](https://ratatui.rs/). It allows you to install new factorio versions and manager that installations. 

Showcase:
![showcase](https://upload.patrick115.eu/raw/images/e178bc1c-20b1-4cda-b040-dea48da1b3a5.png)

## Features
- Needs you to login via your factorio account to download versions
- Install new versions
- Manage installed versions (remove, update - incremental updates via patches on supported OSes, or reinstall to latest version of that branch)

## Tutorial
1. Clone and build the repository
2. Run the binary
3. Login with your factorio username and token
    - You can find your username and token [here](https://factorio.com/profile)
    - The token is used to authenticate your downloads
4. Keybindings:
    - `q`/`Esc`/`Ctrl+C` - Quit the application
    - `I` - Select installalled version list
    - `L` - Select Logs list
    - `Up`/`Down` - Navigate through the current list
    - `A` - to Add new version to install
    - `D` - to Delete selected installed version
    - `U` - to Update selected installed version (if update available)

### Adding new version
1. Press `A` to open the add new version dialog
2. Select Platform (Linux, Windows, MacOS)
3. Select Version branch (Stable, Space Age (DLC), Headless (Server with no GUI))
4. Select Version to install
5. Select Path to install the version to
6. Confirm installation

### Removing installed version
1. Select installed version from the installed versions list
2. Press `D` to delete the selected version
3. Confirm deletion

### Updating installed version
Updating version can appear in 3 ways:
- New patch available, and you are on supported OS (You want to update Linux version on Linux OS or Windows version on Windows OS)
- You have older patch, and factorio removed some patch between, so you need to download full game again
- You are on unsupported OS (You want to update Linux version on Windows OS or Windows version on Linux OS) then you need to download full game again (or switch to supported OS for patch updates)

1. Select installed version from the installed versions list
2. Press `U` to update the selected version
3. Confirm update

## Info
This is a personal project, made as a part of masters study on [VŠB - Technical University of Ostrava](https://www.vsb.cz/en/) in [Programming in Rust](edison.sso.vsb.cz/cz.vsb.edison.edu.study.prepare.web/SubjectVersion.faces?version=460-4157/01) course.