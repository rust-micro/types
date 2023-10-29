# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## 0.2.4 - 2023-10-29

- add Barrier type, which implements a barrier for sync based on Redis

## 0.2.3 - 2023-10-29

- add RwLock Type, which implements a reader-writer lock based on Redis

## 0.2.2 - 2023-10-29

- add ClockOrdered Type, which implements a counter based clock ordering
- minor additions to documentation

## 0.2 - Lock and ADT - 2023-09-22

- add Mutex and Guard
- rename lib to dtypes
- strip the string `redis` from all types and place them in a module named `redis`
- add List and ListCache type

## 0.1 - Initial release - 2023-09-13

### Added

- add LICENSE.md file
- add README.md file
- add .gitignore file
- add CHANGELOG.md file
- add compose.yaml file for docker
- add Redis & Serde as dependencies
- add Makefile.toml for cargo-make as task runner
- add Generic type RedisGeneric as a wrapper for any type
- add signed and unsigned integers type
- add String type
- add Bool type
- add Github Actions
