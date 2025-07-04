# ej-dispatcher

The EJ Dispatcher (EJD) application for managing distributed testing infrastructure.

## Overview

`ejd` is the central coordination application in the EJ framework. It handles job queuing, distribution across multiple builders, result storage, and authentication. The dispatcher acts as the root of the testing infrastructure tree, managing multiple builders and their associated boards.

## Features

- Job queuing and distribution management
- Result storage and retrieval
- Authentication and authorization system
- Builder registration and management
- Multi-builder coordination
- Database integration for persistent storage
- RESTful API and Unix Socket Interface for job management
- Real-time job status tracking

## Installation

Follow the README on [this repository](https://github.com/embj-org/ejd-deployment) to deploy it to your server.

## Part of EJ Framework

This crate is part of the [EJ Framework](https://github.com/embj-org/ej) - a modular and scalable framework for automated testing on physical embedded boards.
