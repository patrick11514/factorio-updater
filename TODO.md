TODO
- [ ] Config::save to async
- [x] Somehow propagate events from Screen -> App so we can switch screens for example

- [x] Add log window, with some basic log levels (info, warn, error)
- [x] Add login screen with some inputs? And password input should show stars. And the nsave it to config file :) 
- [ ] On main screen, there should be FACTORIO :gear: LOGO + some options. So for example list of installed versions and their versions
- [ ] Add some sort of picker, to download some specific platform (linux, windows, macos of latest version) and then patch installed versions which is old
- [ ] Add some progress bars etc.. :)
- [ ] Add some only CLI mode, without TUI 


TODO real :) :
- [ ] Change log list to -> Vec<Arc<Mutex<Log>>> So we can modify the logs from other threads :)
- [ ] On open, if config exists, try to download list of versions, and verify if the config is valid, otherwise open popup -> redirect to login screen