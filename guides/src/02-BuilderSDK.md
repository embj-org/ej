# Guide 02: Understanding and Using the Builder SDK

You can find the SDK's documentation in [crates.io](https://crates.io/crates/ej-builder-sdk)

## Overview

In the previous guide, we successfully set up a basic EJ Builder using shell scripts to deploy and test applications on embedded hardware. While this approach works well for simple scenarios, we may have encountered some limitations - particularly around handling long-running processes or cleaning up after interrupted tests.

This guide explores those limitations and demonstrates how the EJ Builder SDK provides robust solutions for production deployments. We'll learn how to convert our shell scripts into a proper Rust application that can handle job cancellation, manage resources properly, and integrate seamlessly with advanced EJ features.

By the end of this guide, we'll have a production-ready builder setup that can handle complex deployment scenarios with confidence.

## Prerequisites

Before starting this guide, ensure you have:

- **Completed Guide 01**: This guide builds directly on the EJ Builder setup from the previous guide
- **Rust toolchain installed**: We'll need `cargo` and the Rust compiler
  - Install via [rustup.rs](https://rustup.rs/) if we haven't already
  - No prior Rust experience required - the guide explains all concepts as we go
- **Our working EJ Builder setup**: From the previous guide, including:
  - The `kmer` project configured and working
  - SSH access to your target device (Raspberry Pi)
  - The `config.toml` file with your board configurations
- **Understanding of the shell script approach**: You should have successfully run the previous guide's shell scripts

## The Problem with Basic Script Deployment

In the previous guide, we set up a basic EJ Builder using shell scripts. While this approach works for simple scenarios, you may have noticed some limitations.

Let's revisit what happens when we deploy applications using basic SSH and shell scripts, particularly with our Raspberry Pi example from Guide 01.

For this, let's add this new config to our `~/ej-workspace/config.toml`:

**NOTE**: Replace `<user>` with your username.

```toml
[[boards.configs]]
name = "infinite-loop"
tags = ["arm64", "infinite-loop"]
build_script = "/home/<user>/ej-workspace/build.sh"
run_script = "/home/<user>/ej-workspace/run.sh"
results_path = "/home/<user>/ej-workspace/results_infinite-loop.txt"
library_path = "/home/<user>/ej-workspace/kmer"
```

The application we will run enters an infinite loop, meaning the application will never exit.

```bash
ejb --config config.toml validate
```

It won't take long to see the problem:

```bash
Validating configuration file: "config.toml"
2025-07-10T13:19:55.029647Z  INFO ejb::build: Board 1/1: Raspberry Pi
2025-07-10T13:19:55.029655Z  INFO ejb::build: Config 1: k-mer-original
2025-07-10T13:19:55.029879Z  INFO ejb::build: Raspberry Pi - k-mer-original Build started
2025-07-10T13:19:55.140067Z  INFO ejb::build: Raspberry Pi - k-mer-original Build ended successfully
2025-07-10T13:19:55.140084Z  INFO ejb::build: Config 2: k-mer
2025-07-10T13:19:55.140310Z  INFO ejb::build: Raspberry Pi - k-mer Build started
2025-07-10T13:19:55.250487Z  INFO ejb::build: Raspberry Pi - k-mer Build ended successfully
2025-07-10T13:19:55.250504Z  INFO ejb::build: Config 3: k-mer-omp
2025-07-10T13:19:55.250722Z  INFO ejb::build: Raspberry Pi - k-mer-omp Build started
2025-07-10T13:19:55.360908Z  INFO ejb::build: Raspberry Pi - k-mer-omp Build ended successfully
2025-07-10T13:19:55.360933Z  INFO ejb::build: Config 4: infinite-loop
2025-07-10T13:19:55.361238Z  INFO ejb::build: Raspberry Pi - infinite-loop Build started
2025-07-10T13:19:55.471432Z  INFO ejb::build: Raspberry Pi - infinite-loop Build ended successfully
2025-07-10T13:19:55.471698Z  INFO ejb::run: k-mer-original - Run started
2025-07-10T13:19:56.903312Z  INFO ejb::run: k-mer-original - Run ended successfully
2025-07-10T13:19:56.903571Z  INFO ejb::run: k-mer - Run started
2025-07-10T13:19:58.114931Z  INFO ejb::run: k-mer - Run ended successfully
2025-07-10T13:19:58.115205Z  INFO ejb::run: k-mer-omp - Run started
2025-07-10T13:19:59.326567Z  INFO ejb::run: k-mer-omp - Run ended successfully
2025-07-10T13:19:59.326831Z  INFO ejb::run: infinite-loop - Run started
```

The underlying application entered an infinite loop and thus both EJB and our `run.sh` script are stuck forever waiting for it to end.

To quit it, we can press `CTRL+C` essentially killing EJB and the `run.sh` script process that is holding the ssh connection.

Now if we run the validation again:

```bash
ejb --config config.toml validate
```

Something we may not expect happens - EJB (or rather the underlying `run.sh` script) fails!

Taking a look at the logs, we can see that `scp` failed because the `infinite-loop`
file is locked as it's still being executed inside our target device.

```bash
scp: dest open "./infinite-loop": Failure
scp: failed to upload file /home/andre/ej-workspace/kmer/build-pi/infinite-loop to ~
```

To actually stop it we need to connect to our Raspberry Pi and kill it:

```bash
ssh ${PI_USERNAME}@${PI_ADDRESS} "killall infinite-loop"
```

This poses a real problem when deploying this to a production environment as we'd like to
make sure that if one job fails, we want to be able to simply consider this job as a failure and run a new job later without having to manually connect to our target board to clean up failed job leftovers.

EJ solves this problem by providing an SDK - called `EJ Builder SDK` - that handles communicating with EJB through an exposed Unix Socket.
The Builder SDK abstracts all of this for us with some pretty simple boilerplate code. Let's create a script to see what it looks like.

## Step 1: Setup an application with the EJB SDK

```bash
cd ~/ej-workspace
cargo init --bin ejkmer-builder
cd ejkmer-builder
cargo add ej-builder-sdk # EJB SDK
cargo add tokio -F macros -F rt-multi-thread -F process # Async runtime
cargo add num_cpus # (Optional) Used to be able to write -j$(nprocs) during the build phase
```

Now let's add this boilerplate code to our `src/main.rs` file:

```rust
use ej_builder_sdk::{Action, BuilderEvent, BuilderSdk, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    let sdk = BuilderSdk::init(|sdk, event| async move {
        match event {
            BuilderEvent::Exit => todo!("Handle exit command"),
        }
    })
    .await?;

    match sdk.action() {
        Action::Build => todo!("Handle build command"),
        Action::Run => todo!("Handle run command"),
    }
}
```

Let's go through each line of code to understand what it's going on:

```rust
use ej_builder_sdk::{Action, BuilderEvent, BuilderSdk, prelude::*};
```

This line is including stuff we need from the `ej_builder_sdk` that we added to our project when we ran the `cargo add ej-builder-sdk` command.

- `Action`: is a Rust Enum used to describe the action this script should take (either `Build` or `Run`). This lets us use the same script as our build and run script - although this isn't mandatory.
- `BuilderEvent`: is a Rust Enum that describe an _Event_ received by EJB. For now, the only event we can expect is `Exit` but there may be others in the future as EJ evolves.
- `BuilderSDK`: is the main BuilderSDK data structure, it will contain every information passed by EJB, this includes:

  - The action to take (`Build` or `Run`)
  - The path to the `config.toml` file
  - The current board name
  - The current board config name

  These informations allow us to use a single script to handle building and testing our application throughout multiple boards and configs.

- `prelude::*`: is the BuilderSDK _crate_ prelude that imports a `Result` and a common `Error` type that can be used by your script.

```rust
#[tokio::main]
async fn main() -> Result<()> {
```

These two lines allow us to describe our main function as Asynchronous - [Wikipedia](<https://en.wikipedia.org/wiki/Asynchrony_(computer_programming)>).

BuilderSDK uses _async_ tasks under the hood to manage the connection with EJB in a transparent way so this will allow us to call these functions.

The return type of our main function is the `Result` type. This `Result` type is pulled from the BuilderSDK `prelude` and uses its internal `Error` type as the `Result` error type.
This allows us to use the `?` operator to easily handle errors in our application.

```rust
    let sdk = BuilderSdk::init(|sdk, event| async move {
        match event {
            BuilderEvent::Exit => todo!("Handle exit command"),
        }
    })
    .await?;
```

This portion of code initializes the `BuilderSDK`. The return type will be a `BuilderSDK` or an error if something went wrong during the initialization process.

The `BuilderSDK::init` function takes in an `async` function callback that will be called when it receives a new event from EJB.

This lets us handle these events the way we see fit (e.g., by killing the process in our target board when we receive an exit request).

The `.await` is necessary because the init function is `async`, this essentially tells the program to wait for the
execution of this call instead of deferring it for later.

The `?` operator will return from the main function (and thus the application) if the `BuilderSDK::init` function returns an error. In this case, the exit code of the application will be non-zero.

```rust
    match sdk.action() {
        Action::Build => todo!("Handle build command"),
        Action::Run => todo!("Handle run command"),
    }
```

Now that we've initialized everything, the `sdk` variable holds every information passed by EJB.
We can use it to query the action to take, the path to the config file, the board name and the board config name allowing us to create generic scripts that handle our build and deployment needs.

Here we are _matching_ on the action to take (either `Action::Build` or `Action::Run`).

Let's now write what the application should do when asked to build and run our application.

## Step 2: Convert our build shell script to Rust code

As a reminder, our current build script looks like this:

```bash
cmake -B ${SCRIPTPATH}/kmer/build-pi \
      -S ${SCRIPTPATH}/kmer \
      -DCMAKE_TOOLCHAIN_FILE=${SCRIPTPATH}/kmer/aarch64_toolchain.cmake

cmake --build ${SCRIPTPATH}/kmer/build-pi -j$(nproc)
```

First off let's write some utility functions to manage the paths we need for this.
As a reminder, EJB provides us with the absolute path to our `config.toml`,
following the directory structure we set up for our project we can find the workspace folder as the parent of our `config.toml` file:

```rust
use std::path::{Path, PathBuf}

fn workspace_folder(config_path: &Path) -> PathBuf {
    config_path
        .parent()
        .expect(&format!(
            "Failed to get folder containing `config.toml` - Config path is: {}",
            config_path.display()
        ))
        .to_path_buf()
}
```

The source folder sits inside our workspace folder:

```rust
fn source_folder(config_path: &Path) -> PathBuf {
    workspace_folder(config_path).join("kmer")
}
```

And our build folder and toolchain file sit inside the source folder:

```rust
fn build_folder(config_path: &Path) -> PathBuf {
    source_folder(config_path).join("build-pi")
}

fn toolchain_file(config_path: &Path) -> PathBuf {
    source_folder(config_path).join("aarch64_toolchain.cmake")
}
```

Once we have these helper functions we can write a very elegant build function:

```rust
use tokio::process::Command;

async fn build_application(sdk: &BuilderSdk) -> Result<()> {
    let config_path = &sdk.config_path();
    let status = Command::new("cmake")
        .arg("-B")
        .arg(build_folder(config_path))
        .arg("-S")
        .arg(source_folder(config_path))
        .arg(&format!(
            "-DCMAKE_TOOLCHAIN_FILE={}",
            toolchain_file(config_path).display()
        ))
        .spawn()?
        .wait()
        .await?;

    assert!(status.success(), "CMake execution failed");

    Command::new("cmake")
        .arg("--build")
        .arg(build_folder(config_path))
        .arg("-j")
        .arg(num_cpus::get().to_string())
        .spawn()?
        .wait()
        .await?;

    assert!(status.success(), "Build failed");
    Ok(())
}
```

**NOTE**: We use the `tokio::process` module instead of `std::process` to keep our code async.
Be careful calling sync functions from async code. We recommend this [tokio guide](https://tokio.rs/tokio/topics/bridging)
that explains how to bridge the two if you're interested.

## Step 3: Convert our run shell script to Rust code

Following the same process with the helper functions we can write similar to our original shell script:

```bash
scp -r ${SCRIPTPATH}/kmer/build-pi/${BOARD_CONFIG_NAME} \
    ${SCRIPTPATH}/kmer/inputs ${PI_USERNAME}@${PI_ADDRESS}:~

ssh ${PI_USERNAME}@${PI_ADDRESS} \
    "time ./${BOARD_CONFIG_NAME} inputs/input.txt 3" 2>&1 | tee ${SCRIPTPATH}/results_${BOARD_CONFIG_NAME}.txt
```

```rust
const PI_USERNAME: &str = "";
const PI_ADDRESS: &str = "";

fn application_path(config_path: &Path, application_name: &str) -> PathBuf {
    build_folder(config_path).join(application_name)
}

fn inputs_path(config_path: &Path) -> PathBuf {
    source_folder(config_path).join("inputs")
}

fn results_path(config_path: &Path, application_name: &str) -> PathBuf {
    workspace_folder(config_path).join(format!("results_{}", application_name))
}
async fn run_application(sdk: &BuilderSdk) -> Result<()> {
    let config_path = &sdk.config_path();
    let app_name = &sdk.board_config_name();

    let result = Command::new("scp")
        .arg("-r")
        .arg(application_path(config_path, app_name))
        .arg(inputs_path(config_path))
        .arg(&format!("{PI_USERNAME}@{PI_ADDRESS}:~"))
        .spawn()?
        .wait()
        .await?;

    assert!(result.success(), "SCP execution failed");

    let result = Command::new("ssh")
        .arg(&format!("{}@{}", PI_USERNAME, PI_ADDRESS))
        .arg(&format!("time ./{} inputs/input.txt 3", app_name))
        .spawn()?
        .wait_with_output()
        .await?;

    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);

    assert!(result.status.success(), "SSH execution failed");

    std::fs::write(
        results_path(config_path, app_name),
        format!("{}\n{}", stdout, stderr),
    )?;

    Ok(())
}
```

## Step 4: Handling cancellation using the EJ Builder SDK

Finally, the reason we started the journey of writing a Rust program instead of a shell script was to be able to handle cancelling our job correctly to not leave a process running forever in our Raspberry Pi.

Here we can open a new SSH connection to kill the process running on our target board, the same way we did manually before:

```rust
async fn kill_application_in_rpi(sdk: &BuilderSdk) -> Result<()> {
    let result = Command::new("ssh")
        .arg(format!("{PI_USERNAME}@{PI_ADDRESS}"))
        .arg(format!("killall {}", sdk.board_config_name()))
        .spawn()?
        .wait()
        .await?;
    assert!(result.success(), "Failed to kill process in RPI");
    Ok(())
}
```

## Step 5: Putting it all together

Using our new functions, we can finish off writing our main application:

**NOTE**: Replace `PI_USERNAME` and `PI_ADDRESS` with their corresponding values.

```rust
use std::path::{Path, PathBuf};
use tokio::process::Command;

use ej_builder_sdk::{Action, BuilderEvent, BuilderSdk, prelude::*};

const PI_USERNAME: &str = "";
const PI_ADDRESS: &str = "";

async fn kill_application_in_rpi(sdk: &BuilderSdk) -> Result<()> {
    let result = Command::new("ssh")
        .arg(format!("{PI_USERNAME}@{PI_ADDRESS}"))
        .arg(format!("killall {}", sdk.board_config_name()))
        .spawn()?
        .wait()
        .await?;
    assert!(result.success(), "Failed to kill process in RPI");
    Ok(())
}
fn workspace_folder(config_path: &Path) -> PathBuf {
    config_path
        .parent()
        .expect(&format!(
            "Failed to get folder containing `config.toml` - Config path is: {}",
            config_path.display()
        ))
        .to_path_buf()
}

fn source_folder(config_path: &Path) -> PathBuf {
    workspace_folder(config_path).join("kmer")
}

fn build_folder(config_path: &Path) -> PathBuf {
    source_folder(config_path).join("build-pi")
}
fn toolchain_file(config_path: &Path) -> PathBuf {
    source_folder(config_path).join("aarch64_toolchain.cmake")
}
fn application_path(config_path: &Path, application_name: &str) -> PathBuf {
    build_folder(config_path).join(application_name)
}

fn inputs_path(config_path: &Path) -> PathBuf {
    source_folder(config_path).join("inputs")
}

fn results_path(config_path: &Path, application_name: &str) -> PathBuf {
    workspace_folder(config_path).join(format!("results_{}", application_name))
}
async fn run_application(sdk: &BuilderSdk) -> Result<()> {
    let config_path = &sdk.config_path();
    let app_name = &sdk.board_config_name();

    let result = Command::new("scp")
        .arg("-r")
        .arg(application_path(config_path, app_name))
        .arg(inputs_path(config_path))
        .arg(&format!("{PI_USERNAME}@{PI_ADDRESS}:~"))
        .spawn()?
        .wait()
        .await?;

    assert!(result.success(), "SCP execution failed");

    let result = Command::new("ssh")
        .arg(&format!("{}@{}", PI_USERNAME, PI_ADDRESS))
        .arg(&format!("time ./{} inputs/input.txt 3", app_name))
        .spawn()?
        .wait_with_output()
        .await?;

    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);

    assert!(result.status.success(), "SSH execution failed");

    std::fs::write(
        results_path(config_path, app_name),
        format!("{}\n{}", stdout, stderr),
    )?;

    Ok(())
}
async fn build_application(sdk: &BuilderSdk) -> Result<()> {
    let config_path = &sdk.config_path();

    let status = Command::new("cmake")
        .arg("-B")
        .arg(build_folder(config_path))
        .arg("-S")
        .arg(source_folder(config_path))
        .arg(&format!(
            "-DCMAKE_TOOLCHAIN_FILE={}",
            toolchain_file(config_path).display()
        ))
        .spawn()?
        .wait()
        .await?;

    assert!(status.success(), "CMake execution failed");

    Command::new("cmake")
        .arg("--build")
        .arg(build_folder(config_path))
        .arg("-j")
        .arg(num_cpus::get().to_string())
        .spawn()?
        .wait()
        .await?;

    assert!(status.success(), "Build failed");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let sdk = BuilderSdk::init(|sdk, event| async move {
        match event {
            BuilderEvent::Exit => kill_application_in_rpi(&sdk).await,
        }
    })
    .await?;

    match sdk.action() {
        Action::Build => build_application(&sdk).await,
        Action::Run => run_application(&sdk).await,
    }
}

```

Now, whenever a job is cancelled by either EJB or EJD (Guide 03) the script will receive the `Exit` event and will clean the necessary resources.

## Step 6: Build your application

```bash
cd ~/ej-workspace/ejkmer-builder
cargo build --release
```

The application is now available inside the `~/ej-workspace/ejkmer-builder/target/release` folder.

## Step 7: Update your EJB config

We can use this new application to handle every build and run configuration so now we need to tell EJB, through its config, to use it.

We can use some `sed` magic to avoid having to change every line manually:

```bash
sed -i 's/\/[b|r].*.sh/ejkmer-builder\/target\/release\/ejkmer-builder/g' ~/ej-workspace/config.toml
```

Here's the final result:

**NOTE**: Replace `<user>` with your username

```toml
[global]
version = "1.0.0"

[[boards]]
name = "Raspberry Pi"
description = "Raspberry Pi with Raspberry OS 64 bits"

[[boards.configs]]
name = "k-mer-original"
tags = ["arm64", "kmer unoptimized"]
build_script = "/home/<user>/ej-workspace/ejkmer-builder/target/release/ejkmer-builder"
run_script = "/home/<user>/ej-workspace/ejkmer-builder/target/release/ejkmer-builder"
results_path = "/home/<user>/ej-workspace/results_k-mer-original.txt"
library_path = "/home/<user>/ej-workspace/kmer"

[[boards.configs]]
name = "k-mer"
tags = ["arm64", "kmer optimized"]
build_script = "/home/<user>/ej-workspace/ejkmer-builder/target/release/ejkmer-builder"
run_script = "/home/<user>/ej-workspace/ejkmer-builder/target/release/ejkmer-builder"
results_path = "/home/<user>/ej-workspace/results_k-mer.txt"
library_path = "/home/<user>/ej-workspace/kmer"

[[boards.configs]]
name = "k-mer-omp"
tags = ["arm64", "kmer multi-threaded optimized"]
build_script = "/home/<user>/ej-workspace/ejkmer-builder/target/release/ejkmer-builder"
run_script = "/home/<user>/ej-workspace/ejkmer-builder/target/release/ejkmer-builder"
results_path = "/home/<user>/ej-workspace/results_k-mer-omp.txt"
library_path = "/home/<user>/ej-workspace/kmer"

[[boards.configs]]
name = "infinite-loop"
tags = ["arm64", "infinite-loop"]
build_script = "/home/<user>/ej-workspace/ejkmer-builder/target/release/ejkmer-builder"
run_script = "/home/<user>/ej-workspace/ejkmer-builder/target/release/ejkmer-builder"
results_path = "/home/<user>/ej-workspace/results_infinite-loop.txt"
library_path = "/home/<user>/ej-workspace/kmer"
```

#### TIP

Putting the application in our `$PATH` will make it easier to invoke it, for this we recommend
installing it in our PC directly:

```bash
cargo install --path ~/ej-workspace/ejkmer-builder
```

With the application installed you can set every build and run scripts in your config file like this:

```toml
build_script = "ejkmer-builder"
run_script = "ejkmer-builder"
```

And of course a `sed` command to avoid having to do it manually:

```bash
sed -i 's/script = .*/script = "ejkmer-builder"/g' ~/ej-workspace/config.toml
```

This makes our `config.toml` easier to read and allows us to freely move our source code if we wish so.

## Step 8: Test the new script

Make sure you've cleaned up the running process in your raspberry pi:

```bash
ssh ${PI_USERNAME}@${PI_ADDRESS} "killall infinite-loop"
```

```bash
cd ~/ej-workspace
ejb --config config.toml validate
```

We can again quit EJB with `CTRL+C` and we'll be able to see that the `infinite-loop`
is not running on our Raspberry Pi even after abruptly quitting the whole process.

```bash
ssh ${PI_USERNAME}@${PI_ADDRESS} "killall infinite-loop"
infinite-loop: no process found
```

## Advantages of using the EJ Builder SDK

- Proper cancellation handling. When EJB sends an exit signal, your script can clean up running processes on target devices instead of leaving them orphaned
- Single binary approach. One application handles both building and running (though you could do this with shell scripts too, it's just arguably harder)
- Custom result formats. Our example just saves program output to a file, but we can collect and format results however makes sense for our use case
- Easy integration testing. Write tests that spawn TCP listeners, launch our program on the target device, and verify the results in real-time
- Unlimited possibilities. Once we're using a real programming language, we can do things like:
  - Monitor system resources (CPU, memory, network) during test execution
  - Send notifications to Slack when tests complete
  - Generate detailed HTML reports with charts and graphs
- Job cancellation support with EJD (Guide 03)
- You get to write rust code

## Disadvantages of using the EJ Builder SDK

- Setup overhead. It takes longer to get started compared to throwing together a quick shell script
- Compile-test cycle: Every change requires a `cargo build` before you can test it, which slows down rapid iteration
  - Can be minimized by tools like `cargo-watch`
- Rust knowledge required: You need to be comfortable with Rust syntax, and async programming
  - Though the SDK could be ported to other languages very easily. Contributions are welcome
- Binary management: Need to keep track of compiled binaries and make sure they're available where EJB expects them.
  - Installing the application with `cargo install` solves this
- Overkill for simple tasks: If we're just running basic commands and don't need to clean up any resources when a job fails, a shell script might be simpler
  - E.g., when running tests in an MCU where every deployment overwrites the board's flash memory.
- You get to write rust code

## Next Steps

At this point, we have a fully functional EJ Builder setup that can handle complex deployments with proper cancellation handling. EJB works perfectly fine as a standalone tool - we can integrate it into CI/CD pipelines or use it on our development machine to spin up integration tests in the background while we work on other tasks.

You may have noticed that throughout this guide, we haven't stored results anywhere and we've only worked with a single builder. This is completely fine for many use cases, but if you're looking at larger-scale deployments with multiple builders, you might want something more robust.

In [Guide 03 - Dispatcher](03-Dispatcher.md), we'll explore the EJ Dispatcher (EJD) a tool that can:

- Manage multiple builders simultaneously
- Queue and distribute jobs across your hardware fleet
- Store and organize results from multiple test runs
- Provide authentication and access control
- Enable remote job submission and monitoring

The dispatcher transforms EJ from a single-builder tool into a powerful distributed testing platform, but it's entirely optional depending on our needs.

---

**Best Practice**: Always use the Builder SDK for production deployments, especially for long-running applications.
