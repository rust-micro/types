# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased] - DESCRIPTION - YYYY-MM-DD

- 

## 0.2 - Lock and ADT - 2023-09-22

- add Mutex and Guard
- rename lib to dtypes
- strip the string `redis` from all types and place them in a module named `redis`
- add List and ListCache type
- add SetLoad Type, which implements a counter based clock ordering

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
