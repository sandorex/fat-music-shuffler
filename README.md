## fat32-music-shuffler
Simple utility for managing music for dumb MP3 players that cannot shuffle on their own

It works by using [hardlinks](https://en.wikipedia.org/wiki/Hard_link), which are not supported by FAT32 so corruption warnings will be present

### Guide
The utility requires linux environment, so use a VM/WSL if you are not a linux user

1. Plug in your Flash drive or SD Card
2. Run `f32ms format` and select your device like so
   ```
   ...
   /dev/sdb "SD/MMC" 7.5G
   ...
   Enter device path: /dev/sdb
   ...
   Formatting done, for any other commands please use "/dev/sdb1" as the device path
   ```
3. Import the music by running
   ```
   f32ms /dev/sdb1 import song1.mp3 ./album/ ...
   ```
4. Shuffle the music and repeat the songs until they fill at least 2 days worth of playtime
   ```
   f32ms /dev/sdb1 shuffle --repeat-fill '2 days'
   ```
5. Insert into the mp3 player and enjoy!

#### Manually adding or removing files
Manually transfering files is not that complicated but can take a while

> **WARNING Make sure to 'safely eject' the device after transfering the files**

Assuming you formatted the device already
1. Run following command to clean up the filesystem to allow modification, this will not delete your files or music only clear out `LINKS/` directory
   ```
   f32ms /dev/sdb1 clean
   ```
2. Mount the device
3. Add / remove MP3 files in `ORIG/` directory
4. Rename all the MP3 files so they all end with `.mp3.x` extension
5. Safely eject / remove the device

### Why
Most dumb MP3 players play music by the order they were transfered to the storage device, this means reordering requires deleting and transfering the files over again which is not great for a memory card

The goal of this utility is to create long enough repeating playlist that it feels like its shuffling without actually duplicating the files multiple times, because hardlinks don't take much data it is blazing fast too!

### Credits
Huge thanks to creators of [fatfs](https://github.com/rafalh/rust-fatfs) crate, i've built this on top of their code
