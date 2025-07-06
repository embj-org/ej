# ej-config

Configuration structures and utilities for EJ builder configurations.

## Overview

`ej-config` provides the configuration structures and utilities used to define EJ builder configurations. Since builder configurations are used by both the builder and dispatcher components, this crate serves as a shared dependency to ensure consistency across the framework.

## Features

- Builder configuration data structures
- Board configuration definitions
- Configuration serialization and deserialization
- Configuration validation utilities
- Shared configuration types used across EJ components

## Installation

```bash
cargo add ej-config
```

## Usage

This crate is primarily used as a shared dependency by other EJ components. It ensures that builder configurations are consistently defined and handled across the builder and dispatcher.

## Part of EJ Framework

This crate is part of the [EJ Framework](https://github.com/embj-org/ej) - a modular and scalable framework for automated testing on physical embedded boards.
