# ej-builder

The EJ Builder (EJB) application for managing build processes and board communication.

## Overview

`ejb` is one of the two main applications in the EJ framework. It manages build processes, handles communication with physical boards, and can operate either as part of a distributed system with a dispatcher or in standalone mode for local testing.

## Features

- Build process management across multiple boards
- Board communication and control
- Integration with EJ dispatcher for distributed testing
- Standalone mode for local testing workflows
- Configuration-based board management
- Job cancellation and status reporting
- Custom build and run script execution

## Installation

```bash
cargo install ejb
```

## Usage Modes

### Distributed Mode
Connect to an EJD for distributed testing across multiple builders and boards.

### Standalone Mode
Run independently for local testing workflows without requiring a dispatcher instance.

## Configuration

EJ Builder uses TOML configuration files to define board setups, build scripts, and connection settings. Multiple board configurations can be managed by a single builder instance.

## Part of EJ Framework

This crate is part of the [EJ Framework](https://github.com/embj-org/ej) - a modular and scalable framework for automated testing on physical embedded boards.
