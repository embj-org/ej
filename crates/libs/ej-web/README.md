# ej-web

Private web utilities and components for the EJ dispatcher.

## Overview

`ej-web` contains internal web-related functionality used by the EJ dispatcher. This includes private API endpoints, web server utilities, and internal web components that are not intended for external use. Applications that need to interface with the dispatcher should use `ej-dispatcher-sdk` instead.

## Features

- Internal web server components
- Private API endpoint implementations
- Web middleware and utilities
- Internal routing and handlers
- Dispatcher-specific web functionality

## Installation

```bash
cargo add ej-web
```

## Note

This crate contains internal implementation details for the EJ dispatcher. For external applications that need to interface with the dispatcher, use `ej-dispatcher-sdk` instead.

## Part of EJ Framework

This crate is part of the [EJ Framework](https://github.com/embj-org/ej) - a modular and scalable framework for automated testing on physical embedded boards.
