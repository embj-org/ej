# EJ

## Introduction

EJ is a modular and scalable framework for automated testing on physical embedded boards

It enables dispatching tests to real hardware, collecting logs and performance metrics, and detecting hardware-specific issues.
The framework is designed to support diverse board architectures and simplify distributed hardware testing for embedded projects.

## Architecture

EJ is built with modularity in mind, consisting of multiple libraries and two main applications:

### Core applications
- EJB (EJ Builder) - Manages build processes and board communication
- EJD (EJ Dispatcher) - Handles job queuing, distribution, and result collection

### Libraries
- ej-auth - Authentication utilities (JWT management, password hashing)
- ej-builder-sdk - Interface library for creating builder applications
- ej-config - Shared configuration structures and utilities for EJ builder configurations, ensuring consistency between builder and dispatcher components.
- ej-dispatcher-sdk - Interface library for creating dispatcher applications  
- ej-io - Program management utilities
- ej-models - Database models for EJ
- ej-requests - HTTP request handling utilities
- ej-web - Private web utilities and components used internally by the EJ dispatcher, not intended for external use.

## Design philosophy

EJ doesn't make assumptions about how to build, run, or manage your test results. This flexibility is achieved through the SDK architecture:

- Builder SDK - Create custom build and run scripts with seamless builder communication and job cancellation support
- Dispatcher SDK - Interface with the dispatcher to dispatch jobs and retrieve results

This gives you complete control over how tests are built and deployed, how results are parsed, and board-specific configurations and requirements.

## Key Features

### Flexible hardware support

EJ lets you handle the building and running of your application, you can very easily create a script that compiles and runs locally,
on an MPU like Raspberry Pi or simpler MCUs like ESP32.
As long as you can automate the process of building and running your application, you can use EJ.

### Distributed Testing

EJ's architecture follows a tree format with the EJD at its root, and any number of branches extending from it. Do you need to deploy another builder? Simply install it, configure it and launch it.
Do you need another board? Simply add a config entry to your builder and you're done.

### CI/CD Integration

EJ was built with CI/CD integration in mind. 
You can either deploy a dispatcher and let it handle the jobs for you, or you can set up a GitHub/GitLab runner on your EJB machine and integrate it into GitHub Actions / GitLab CI easily.

### Authentication & Security

EJD authentication is built-in. Only builders with a valid token can connect to it.

## Getting Started

### Deploying EJD (Optional)

We provide a ready to use [repository](https://github.com/embj-org/ejd-deployment) to launch an EJD on the server you want.
This step is optional if you don't need job orchestration and results storage.

### Deploying EJB

TODO

#### EJB Configuration

You can find an example of an EJB configuration in the [examples folder](examples/config.toml).
We recommend using the example as a template to create your own.

## Background

EJ was originally designed and built for a bachelor thesis to provide LVGL with automatic performance measurement in CI.
Some architectural decisions were made to accommodate the needs of a popular open-source project, which required special handling of authentication and scalable testing infrastructure.

## Architecture Diagram

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
                                 │
                    ┌────────────▼────────────┐
                    │   EJB (Builder N)       │
                    │                         │
                    │  - Standalone Mode      │
                    │  - Local Testing        │
                    └─────────────────────────┘
```
