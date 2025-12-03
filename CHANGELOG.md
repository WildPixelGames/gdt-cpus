# Changelog

All notable changes to this project will be documented in this file.

## [2025.12.0] - 2025-12-03

### 🚀 Features

- Implement AffinityMask
- Adds core affinity masks in CpuInfo
- Adds iterator and debug/display impls
- Add set_thread_affinity API for multi-core affinity masks
- Add union and intersection ops to AffinityMask

### 🐛 Bug Fixes

- Correct core type detection on non-hybrid CPUs
- Core affinity setting logic on Linux

### 🚜 Refactor

- Drop thread pinning on macOS

### 📚 Documentation

- Fix clippy warnings
- Add comprehensive documentation for AffinityMask
- Update platform affinity docs for AffinityMask API

### 🧪 Testing

- Add unit tests for AffinityMask

### ⚙️ Miscellaneous Tasks

- Cargo fmt
- Bump deps
- Bump version to 25.12
- Adds git-cliff configuration file
- Add CHANGELOG.md

## [2025.5.0] - 2025-05-22

### ⚙️ Miscellaneous Tasks

- Import from private repo
- Last polish before open sourcing
- Fix ffi
