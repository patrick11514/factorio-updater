TODO
- [x] Config::save to async
- [x] Somehow propagate events from Screen -> App so we can switch screens for example

- [x] Add log window, with some basic log levels (info, warn, error)
- [x] Add login screen with some inputs? And password input should show stars. And the nsave it to config file :) 
- [x] Add some sort of picker, to download some specific platform (linux, windows, macos of latest version) and then patch installed versions which is old
- [x] Add some progress bars etc.. :)
- [ ] Add some only CLI mode, without TUI 


TODO real :) :
- [x] Change log list to -> Vec<Arc<Mutex<Log>>> So we can modify the logs from other threads :)
- [x] On open, if config exists, try to download list of versions, and verify if the config is valid, otherwise open popup -> redirect to login screen
- [x] Selectable area -> I -> Installed versions L -> Logs -> make border colored, like focused (yellow)
- [x] Scrollable logs -> on focus arrow up + down
- [x] Display some symbol next to each version, like its Up to date/needs update
- [x] Add details
- [ ] Perform patch updates/again full update if no patches to go stable exists
- [ ] Add some sort of deleting the installed versions (D key?)