# Dsync
Cross machine _data-synchronization_ tool.

## Features
- [x] Background syncing
- [x] Cross-platform
- [x] Authoritative server
  - All data is copied to master 
- [ ] Encryption
- [ ] V1 Insecure!!!!
- [x] Speed! (Rolling hash chunking)
  - [ ] Auto hash chunk size negotiation
- [x] Guarantees always in sync

# Assumptions
- Single user(YOU!), multi-machine (Laptop, and desktop)
  - This program is intended to keep game saves synced
- First change must be true!
  - Working on multiple clients at once is discouraged & untested?... just use git or something else atp.

# v1 INSECURE
**Do not expose on internet!!**
- 100% Has exploits 
- Plus any one can connect and sync stuff... so malware syncing.
    
# Usage
Setup a Dysnc/Server somewhere accessible, on LAN minipc/pi/..? (Or Self)

Run client at least once for each machine.
Find .dsync directory
- `Linux`: `~/.dysnc`
- `Windows`: `%USERPROFILE%\.dsync`
- Modify `config.toml`, to point to the correct master_ip & _port if changed_,
- Modify `hostname`, determines how file hashing works (**Important**)

Run ``./client --help``:
- `add`: [REMOTE] Add a file or directory to VCS
- `remove`: [REMOTE] Removes a file or directory to VCS
- `sync`: [LOCAL] Syncs a file/directory to disk
- `dsync`: [LOCAL] Stops a file/directory from being synced
- `list`: List all synced files and directories

## Other settings
- `polling_interval`: The rate at which you want to check if any changes have occurred on the server's end.
- `allow_local_deletion`: If a file is deleted on the server it also deletes it locally (Be careful!)

# V1 -> V2:
Problems:
- The database implementation is really bad in hindsight.
- ``reddb`` kinda sucks.. **for this use case**.
- Client is "dumb" only stores local path tied to remote id, nothing more. This sucks...
- ID Generation is terrible, need to figure out a new way to generate id's.
  - If new ID system is created we need to check duplicate files manually
- Lots of duplicated code between client and server.
- Should probably use a linked tree structure internally for the server backend.
  - Path should in no way be used on the Server only client
- Chunk size auto negotiation
- Server sync_rate config is useless since we don't negotiate...
- Needs 100% refactoring; not my best work. Needed this tool really bad.

# License
This project is licensed under either MIT or Apache-2.0, you choose.