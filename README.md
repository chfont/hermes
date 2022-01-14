# Hermes, A Reminder Daemon for Linux

Hermes (and Caduceus, its CLI application included in this repository) is a notification [daemon](https://en.wikipedia.org/wiki/Daemon_(computing)) for Linux, which is useful for setting reminders, and other notifications that may need to be sent on a schedule. For more details, including build instructions, see the subfolders containing either the daemon itself (Hermes), or the client (Caduceus).

## Organization of this Repository

In this repository, the folder hermes contains the source code and build files for the daemon component. The other folder, caduceus, contains the source code and build files for a cli program to communicate with the daemon, for adding / removing notifications. For more details on each, please see the respective README files in each folder.
