# Getting Started with EJ

Welcome to EJ - a modular and scalable framework for automated testing on physical embedded boards!

## What is EJ?

EJ enables dispatching tests to real hardware, collecting logs, test results, and detect hardware-specific issues.
The framework is designed to support diverse board architectures and simplify distributed hardware testing for embedded projects.

### Key Components

EJ consists of two main applications and several supporting libraries:

**Core Applications:**

- **EJB (EJ Builder)** - Manages build processes and board communication
- **EJD (EJ Dispatcher)** - Handles job queuing, distribution, and result collection
- **EJCli (EJ CLI)** - Helper cli tool to interface with EJD

**Libraries:**

- **ej-builder-sdk** - Interface library for creating custom builder applications
- **ej-dispatcher-sdk** - Interface library for interfacing with dispatchers
- **ej-auth** - Authentication utilities (JWT management, password hashing)
- **ej-config** - Shared configuration structures and utilities
- **ej-io** - Program management utilities
- **ej-models** - Database models for EJ
- **ej-requests** - HTTP request handling utilities
- **ej-web** - Internal web utilities for the dispatcher

## Architecture Overview

EJ follows a tree-like distributed architecture:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Git Repo      │    │   CI/CD         │    │   Developer     │
│                 │    │                 │    │                 │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          │                      │                      │
          └──────────────────────┼──────────────────────┘
                                 │
                    ┌────────────▼────────────┐
                    │   EJD (Dispatcher)      │
                    │                         │
                    │  - Job Queuing          │
                    │  - Result Storage       │
                    │  - Authentication       │
                    └────────────┬────────────┘
                                 │
                 ┌───────────────┼───────────────┐
                 │               │               │
    ┌────────────▼────────────┐  │  ┌────────────▼────────────┐
    │   EJB (Builder 1)       │  │  │   EJB (Builder 2)       │
    │                         │  │  │                         │
    │  - Build Management     │  │  │  - Build Management     │
    │  - Board Communication  │  │  │  - Board Communication  │
    └────────────┬────────────┘  │  └────────────┬────────────┘
                 │               │               │
         ┌───────┼─────────┐     │       ┌───────┼────────┐
         │       │         │     │       │       │        │
    ┌────▼──┐┌───▼───┐ ┌───▼───┐ │  ┌────▼──┐┌───▼───┐┌───▼───┐
    │Board 1││Board 2│ │Board 3│ │  │Board 4││Board 5││Board 6│
    │(RPi4) ││(ESP32)│ │(PC)   │ │  │(RPi3) ││(STM32)││(x86)  │
    └───────┘└───────┘ └───────┘ │  └───────┘└───────┘└───────┘
```

## Design Philosophy

EJ doesn't make assumptions about how to build, run, or manage your test results. This flexibility is achieved through:

- **Builder SDK** - Create custom build and run scripts with seamless builder communication and job cancellation support
- **Dispatcher SDK** - Interface with the dispatcher to dispatch jobs and retrieve results

This gives us complete control over how tests are built and deployed, how results are parsed, and board-specific configurations and requirements.

## Guide Structure

This guide series will walk us through setting up and using EJ from basic to advanced scenarios:

### [01 - Builder](01-Builder.md)

Learn how to set up our first EJ Builder with a basic configuration. We'll deploy a simple application to a Raspberry Pi as a practical example, covering:

- Installing and configuring EJB
- Creating your first board configuration
- Writing build and run scripts
- Deploying and testing a simple application

### [02 - Builder SDK](02-BuilderSDK.md)

Discover why the Builder SDK exists and how it solves common deployment issues. We'll explore:

- The problems that can arise if we aren't careful with automatic deployments
- How the Builder SDK provides better control and monitoring
- Migrating from basic scripts to SDK-based solutions

### [03 - Dispatcher](03-Dispatcher.md)

Set up a centralized job management system with EJD. This guide covers:

- Installing and configuring the EJ Dispatcher
- Connecting builders to the dispatcher
- Managing jobs, queues, and results

### [04 - Dispatcher SDK](04-DispatcherSDK.md)

Create custom tools to interface with your dispatcher. Learn how to:

- Build a custom CLI tool using the Dispatcher SDK
- Submit jobs programmatically
- Parse and analyze results

## Prerequisites

Before starting with the guides, ensure you have:

- **Rust toolchain** (latest stable version)
- **Target hardware** (Raspberry Pi recommended for examples)
- **SSH access** to your target boards
- **Basic command line familiarity**
- **Git** for version control

## Next Steps

Ready to get started? Head over to [Guide 01 - Builder](01-Builder.md) to set up your first EJ Builder and deploy your first application!

## Getting Help

- **Issues**: Report bugs and request features on our [GitHub repository](https://github.com/embj-org/ej)
- **Documentation**: Check the README.md and our crates in [crates.io](https://crates.io/search?page=2&q=ej) for API references
- **Examples**: Explore the `examples/` directory for configuration templates

---

_EJ was originally designed and built for a bachelor thesis to provide LVGL with automatic performance measurement in CI.
The architecture supports both small-scale local testing and large-scale distributed testing scenarios._
