<!--
Copyright (C) 2022 Nitrokey GmbH
SPDX-License-Identifier: CC0-1.0
-->

# Changelog

## Unreleased

## [v0.1.3][] (2026-11-12)

- Fix variance of `RefSwap` and `OptionRefSwap`.
- Fix `OptionRefSwap::into_inner` returning a reference instead of an option.

These changes are technically breaking, and might break compilation, however dependencies where compilation is now broken were most likely experiencing undefined behaviour.

[v0.1.3]: https://github.com/Nitrokey/ref-swap/releases/tag/v0.1.3
