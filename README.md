## fat32-music-shuffler
Simple utility for managing music for dumb MP3 players that cannot shuffle on their own

It works by using [hardlinks](https://en.wikipedia.org/wiki/Hard_link), which are not supported by FAT32 so corruption warnings will be present

### Guide
The utility requires linux environment, so use a VM/WSL if you are not a linux user

> **TODO**

### Why
Most dumb MP3 players play music by the order they were transfered to the storage device, this means reordering requires deleting and transfering the files over again which is not great for a memory card

The goal of this utility is to create long enough repeating playlist that it feels like its shuffling without actually duplicating the files multiple times, because hardlinks don't take much data it is blazing fast too!

### Credits
Huge thanks to creators of [fatfs](https://github.com/rafalh/rust-fatfs) crate, i've built this on top of their code
