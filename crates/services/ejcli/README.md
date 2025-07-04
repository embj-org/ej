# ej-cli

Command-line interface for EJ dispatcher setup and job management.

## Overview

`ejcli` is a command-line tool for setting up and managing EJ dispatcher instances. It provides essential setup functionality like creating the initial root user and builder registration, as well as job management capabilities for testing and monitoring your EJ infrastructure.

## Features

- Initial dispatcher setup and configuration
- Root user creation for new EJ dispatcher instances
- Builder creation and registration
- Job dispatching for both build and run operations
- Job result retrieval and display
- Infrastructure testing and validation
- Unix socket communication with EJ dispatcher

## Installation

Install as a binary:

```bash
cargo install ejcli
```

## Use Cases

### Initial Setup
Use EJ CLI to set up a new EJ dispatcher instance by creating the first root user and configuring initial builders.

### Testing and Monitoring
Dispatch test jobs and retrieve results to validate your EJ infrastructure setup and monitor job execution.

## Part of EJ Framework

This crate is part of the [EJ Framework](https://github.com/embj-org/ej) - a modular and scalable framework for automated testing on physical embedded boards.
