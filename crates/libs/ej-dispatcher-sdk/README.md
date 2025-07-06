# ej-dispatcher-sdk

SDK for creating applications that interface with EJD.

## Overview

`ej-dispatcher-sdk` provides the interface library for creating applications that communicate with EJD. It handles job dispatching, result retrieval, and provides a clean API for interacting with the dispatcher's job management system.

## Features

- Job dispatching to available builders
- Result retrieval and status monitoring
- Authentication with dispatcher
- Builder registration and management
- Real-time job progress tracking

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ej-dispatcher-sdk = "0.3.3"
```

## Part of EJ Framework

This crate is part of the [EJ Framework](https://github.com/embj-org/ej) - a modular and scalable framework for automated testing on physical embedded boards.
