# photo_backup
Compatibility:
<ul><b>Windows</b></ul>
Currently does some of the things I want:
It takes 2 inputs, generates a date-time stamp and copies all files/folder structure from source to destination

TODO: verify integrity of files with hashing, and delete files from source that match jp(e)g and RAW files

A rust program for backing up photos from an SD card to an external media

Goals: Copy files from SD card to an external drive and ensure integrity by confirming hashes of copied files, finally delete files from SD card to free up space.

Future: would like to target for Raspberry Pi 4, OLED display showing progress w/ progress bar, push buttons for selecting source and destination devices, maybe enable/disable hash checking depending on copy and hash speed

Commit test