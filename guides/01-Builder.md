# Guide 01: Setting Up Your First EJ Builder

This guide will walk you through setting up your first EJ Builder (EJB) and deploying a simple application to a Raspberry Pi.
By the end of this guide, you'll have a working EJ Builder that can build and run applications on physical hardware.

## Overview

In this guide, we'll:

1. Install and configure EJB
2. Set up a Raspberry Pi target board
3. Configure board settings and deployment scripts
4. Test the complete build and deployment workflow

## Prerequisites

Before starting, ensure you have:

- A Raspberry Pi with Raspberry Pi OS installed
- SSH access to your Raspberry Pi
- [Cargo](https://rustup.rs/) installed on your host machine
- The [AArch64 GNU/Linux toolchain](https://developer.arm.com/-/media/Files/downloads/gnu-a/10.3-2021.07/binrel/gcc-arm-10.3-2021.07-x86_64-aarch64-none-linux-gnu.tar.xz?rev=1cb9c51b94f54940bdcccd791451cec3&hash=A56CA491FA630C98F7162BC1A302F869) in your PATH.
- Basic familiarity with SSH and shell scripting is a bonus

## Application example: K-mer Algorithm Performance Benchmark

For this guide, we'll use a k-mer counting application that:

- Processes the digits of PI to count k-mer occurrences
- Measures execution time and memory usage
- Outputs performance metrics to stdout
- Demonstrates computational differences between platforms

**Note**: K-mer algorithms are typically used in bioinformatics for analyzing DNA sequences ([Wikipedia: K-mer](https://en.wikipedia.org/wiki/K-mer)).
For this guide, we use the digits of PI as our input sequence since it provides a deterministic, easily reproducible dataset that still demonstrates the
algorithm's computational characteristics.

This example showcases:

- **Cross-platform deployment** (development machine to Raspberry Pi)
- **Performance measurement** (timing and resource usage)
- **Result collection** (stdout capture)
- **Real computational workload** (pattern counting algorithm)

## Step 1: Clone the kmer application

### Application Code

We provide multiple versions of a kmer application

- An unoptimized version
- A single-threaded optimized version
- A multi-threaded optimized version

```bash
mkdir -p ~/ej-workspace
cd ~/ej-workspace
git clone https://github.com/embj-org/kmer.git
cd kmer
```

### Cross compile the application

This is to ensure everything is working properly

```bash
cmake -B build-pi -DCMAKE_TOOLCHAIN_FILE=aarch64_toolchain.cmake
cmake --build build-pi -j$(nproc)
```

### Test the application

```bash
PI_USERNAME=<your_pi_username>
PI_ADDRESS=<your_pi_ip_address>
scp -r build-pi/k-mer-omp inputs ${PI_USERNAME}@${PI_ADDRESS}:~
ssh ${PI_USERNAME}@${PI_ADDRESS} "./k-mer-omp inputs/pi_dec_1k.txt 3"
```

If any of these steps fail, ensure you have the correct toolchain installed and available in your PATH.

--- 

Now that we've ensured everything is working, it's now time to use EJ.

## Step 2: Install EJB

```bash
cargo install ejb
```

## Step 3: Create a build and run script

Inside `~/ej-workspace`, create the following scripts:

### Build Script (`build.sh`)

This script is responsible for building the application.
We already did this previously so we can simply copy the same steps as before.

```bash
#!/bin/bash
set -e

SCRIPT=$(readlink -f $0)
SCRIPTPATH=$(dirname $SCRIPT)

cmake -B ${SCRIPTPATH}/kmer/build-pi -S ${SCRIPTPATH}/kmer -DCMAKE_TOOLCHAIN_FILE=${SCRIPTPATH}/kmer/aarch64_toolchain.cmake
cmake --build ${SCRIPTPATH}/kmer/build-pi -j$(nproc)
```

### Run Script (`run.sh`)

Same thing for the run script but right now what we'll be doing is only testing the original implementation
Additionnally, we need to output the program results to a file so they can be used later. 
Finally, besides the results we'll actually time the application

```bash
#!/bin/bash
set -e

PI_USERNAME=<your_pi_username>
PI_ADDRESS=<your_pi_ip_address>

SCRIPT=$(readlink -f $0)
SCRIPTPATH=$(dirname $SCRIPT)

scp -r ${SCRIPTPATH}/kmer/build-pi/k-mer-original ${SCRIPTPATH}/kmer/inputs ${PI_USERNAME}@${PI_ADDRESS}:~
ssh ${PI_USERNAME}@${PI_ADDRESS} "time ./k-mer-original inputs/input.txt 3" 2>&1 | tee ${SCRIPTPATH}/results.txt
```

### Making Scripts Executable

```bash
cd ~/ej-workspace
chmod +x build.sh run.sh
```

## Step 4: Configuring EJB

### Board Configuration

Inside `~/ej-workspace`

Create your `config.toml`. This file is responsible for describing every board and every board configuration EJB should handle.

We'll start with a very simple config file that describes our single board and a single config. 

**NOTE**: Replace `<user>` with your username.

```toml
[global]
version = "1.0.0"

[[boards]]
name = "Raspberry Pi"
description = "Raspberry Pi with Raspberry OS 64 bits"

[[boards.configs]]
name = "k-mer-original"
tags = ["arm64", "kmer unoptimized"]
build_script = "/home/<user>/ej-workspace/build.sh"
run_script = "/home/<user>/ej-workspace/run.sh"
results_path = "/home/<user>/ej-workspace/results_k-mer-original.txt"
library_path = "/home/<user>/ej-workspace/kmer"
```

You may notice the board config name, the results path and the executable all have the same name.
This is NOT necessary but will make it easier to build upon later on.

### Configuration Explanation

- **Board Definition**: Describes your Raspberry Pi hardware
- **Config Section**: Defines how to build and run the k-mer benchmark
- **Scripts**: Point to your build and run scripts
- **Results Path**: Where EJB will look for captured stdout output
- **Tags**: Help categorize and filter boards


## Step 5: Testing the config

### Parse the config

EJB can be used to parse the file to make sure the config is correct

```bash
ejb --config config.toml parse
```

You should see the following output:

```bash
Configuration parsed successfully
Global version: 1.0.0
Number of boards: 1

Board 1: Raspberry Pi
  Description: Raspberry Pi with Raspberry OS 64 bits
  Configurations: 1
    Config 1: k-mer-original
      Tags: ["arm64", "kmer unoptimized"]
      Build script: "/home/andre/ej-workspace/build.sh"
      Run script: "/home/andre/ej-workspace/run.sh"
      Results path: "/home/andre/ej-workspace/results_k-mer-original.txt"
      Library path: "/home/andre/ej-workspace/kmer"
```

### Test the config

You can now use EJB to run the described tests for you:

```bash
ejb --config config.toml validate
```

You should see this output followed by the test results
```bash
Validating configuration file: "config.toml"
2025-07-10T12:30:08.401673Z  INFO ejb::build: Board 1/1: Raspberry Pi
2025-07-10T12:30:08.401682Z  INFO ejb::build: Config 1: k-mer-original
2025-07-10T12:30:08.401882Z  INFO ejb::build: Raspberry Pi - k-mer-original Build started
2025-07-10T12:30:08.512101Z  INFO ejb::build: Raspberry Pi - k-mer-original Build ended successfully
2025-07-10T12:30:08.512427Z  INFO ejb::run: k-mer-original - Run started
2025-07-10T12:30:10.054405Z  INFO ejb::run: k-mer-original - Run ended successfully
2025-07-10T12:30:10.054516Z  INFO ejb::run: Found results for Raspberry Pi - k-mer-original
========================
Log outputs for Raspberry Pi k-mer-original
========================
-- Configuring done (0.0s)
-- Generating done (0.0s)
-- Build files have been written to: /home/andre/ej-workspace/kmer/build-pi
[ 50%] Built target k-mer
[ 66%] Built target k-mer-original
[100%] Built target k-mer-omp
Results:
ABC: 2
BCD: 1
CDA: 1
DAB: 1

real	0m0.005s
user	0m0.000s
sys	0m0.005s

========================
Result for Raspberry Pi k-mer-original
========================
Results:
ABC: 2
BCD: 1
CDA: 1
DAB: 1

real	0m0.005s
user	0m0.000s
sys	0m0.005s
```

## Step 6: Adding more configs

We showcased a pretty simple example to see how to setup one board with one config. In reality, EJ is equipped to handle multiple 
boards with multiple configs each.

Like we discussed before, there are actually 3 versions of this software, so let's use EJ to actually test the three versions.


### Script Arguments

The scripts we created were pretty basic and only do one thing. We can easily imagine this becoming cumbersome very quickly.

EJ solves this problem by passing arguments to your build and run scripts:

- `argv[1]`: Action (`build` or `run`)
- `argv[2]`: Config file path
- `argv[3]`: Board name
- `argv[4]`: Board config name
- `argv[5]`: Socket path for EJB communication. We'll be discussing this one further in a following guide.


With these arguments, we can actually create a more sophisticated script to handle every config for us.

Let's modify our `run` script to handle this for us:

```bash
set -e
PI_USERNAME=<your_pi_username>
PI_ADDRESS=<your_pi_ip_address>

SCRIPT=$(readlink -f $0)
SCRIPTPATH=$(dirname $SCRIPT)

BOARD_CONFIG_NAME=$4

scp -r ${SCRIPTPATH}/kmer/build-pi/${BOARD_CONFIG_NAME} ${SCRIPTPATH}/kmer/inputs ${PI_USERNAME}@${PI_ADDRESS}:~
ssh ${PI_USERNAME}@${PI_ADDRESS} "time ./${BOARD_CONFIG_NAME} inputs/pi_dec_1k.txt 3" 2>&1 | tee ${SCRIPTPATH}/results_${BOARD_CONFI_NAME}.txt
```

Here by using the board config name that is automatically passed by EJB,
we can now use the same script for every board config.

Add these new config entries at the bottom of your `~/ej-workspace/config.toml`:

**NOTE**: Replace `<user>` with your username.

```toml
[[boards.configs]]
name = "k-mer"
tags = ["arm64", "kmer optimized"]
build_script = "/home/<user>/ej-workspace/build.sh"
run_script = "/home/<user>/ej-workspace/run.sh"
results_path = "/home/<user>/ej-workspace/results_k-mer.txt"
library_path = "/home/<user>/ej-workspace/kmer"

[[boards.configs]]
name = "k-mer-omp"
tags = ["arm64", "kmer multi-threaded optimized"]
build_script = "/home/<user>/ej-workspace/build.sh"
run_script = "/home/<user>/ej-workspace/run.sh"
results_path = "/home/<user>/ej-workspace/results_k-mer-omp.txt"
library_path = "/home/<user>/ej-workspace/kmer"
```

Don't hesitate to use `ejb` to `parse` your config and make sure it's correctly written.

With this new config we can now run `ejb` again and we'll see that it runs all three configs:

```bash
ejb --config config.toml validate
```

```bash
ejb --config config.toml validate
Validating configuration file: "config.toml"
2025-07-10T12:33:02.582019Z  INFO ejb::build: Board 1/1: Raspberry Pi
2025-07-10T12:33:02.582045Z  INFO ejb::build: Config 1: k-mer-original
2025-07-10T12:33:02.582278Z  INFO ejb::build: Raspberry Pi - k-mer-original Build started
2025-07-10T12:33:02.692504Z  INFO ejb::build: Raspberry Pi - k-mer-original Build ended successfully
2025-07-10T12:33:02.692524Z  INFO ejb::build: Config 2: k-mer
2025-07-10T12:33:02.692779Z  INFO ejb::build: Raspberry Pi - k-mer Build started
2025-07-10T12:33:02.802979Z  INFO ejb::build: Raspberry Pi - k-mer Build ended successfully
2025-07-10T12:33:02.803001Z  INFO ejb::build: Config 3: k-mer-omp
2025-07-10T12:33:02.803285Z  INFO ejb::build: Raspberry Pi - k-mer-omp Build started
2025-07-10T12:33:02.913480Z  INFO ejb::build: Raspberry Pi - k-mer-omp Build ended successfully
2025-07-10T12:33:02.913827Z  INFO ejb::run: k-mer-original - Run started
2025-07-10T12:33:04.675982Z  INFO ejb::run: k-mer-original - Run ended successfully
2025-07-10T12:33:04.676299Z  INFO ejb::run: k-mer - Run started
2025-07-10T12:33:06.328239Z  INFO ejb::run: k-mer - Run ended successfully
2025-07-10T12:33:06.328546Z  INFO ejb::run: k-mer-omp - Run started
2025-07-10T12:33:07.870387Z  INFO ejb::run: k-mer-omp - Run ended successfully
2025-07-10T12:33:07.870464Z  INFO ejb::run: Found results for Raspberry Pi - k-mer-omp
2025-07-10T12:33:07.870479Z  INFO ejb::run: Found results for Raspberry Pi - k-mer-original
2025-07-10T12:33:07.870484Z  INFO ejb::run: Found results for Raspberry Pi - k-mer
========================
Log outputs for Raspberry Pi k-mer-original
========================
-- Configuring done (0.0s)
-- Generating done (0.0s)
-- Build files have been written to: /home/andre/ej-workspace/kmer/build-pi
[ 33%] Built target k-mer-original
[100%] Built target k-mer-omp
[100%] Built target k-mer
Results:
ABC: 2
BCD: 1
CDA: 1
DAB: 1

real	0m0.005s
user	0m0.000s
sys	0m0.005s

========================
Result for Raspberry Pi k-mer-original
========================
Results:
ABC: 2
BCD: 1
CDA: 1
DAB: 1

real	0m0.005s
user	0m0.000s
sys	0m0.005s

========================
Log outputs for Raspberry Pi k-mer
========================
-- Configuring done (0.0s)
-- Generating done (0.0s)
-- Build files have been written to: /home/andre/ej-workspace/kmer/build-pi
[ 33%] Built target k-mer-omp
[100%] Built target k-mer-original
[100%] Built target k-mer
Results:
ABC: 2
BCD: 1
CDA: 1
DAB: 1

real	0m0.005s
user	0m0.004s
sys	0m0.001s

========================
Result for Raspberry Pi k-mer
========================
Results:
ABC: 2
BCD: 1
CDA: 1
DAB: 1

real	0m0.005s
user	0m0.004s
sys	0m0.001s

========================
Log outputs for Raspberry Pi k-mer-omp
========================
-- Configuring done (0.0s)
-- Generating done (0.0s)
-- Build files have been written to: /home/andre/ej-workspace/kmer/build-pi
[ 33%] Built target k-mer-original
[ 66%] Built target k-mer
[100%] Built target k-mer-omp
Results:
ABC: 2
BCD: 1
CDA: 1
DAB: 1

real	0m0.008s
user	0m0.005s
sys	0m0.006s

========================
Result for Raspberry Pi k-mer-omp
========================
Results:
ABC: 2
BCD: 1
CDA: 1
DAB: 1

real	0m0.008s
user	0m0.005s
sys	0m0.006s
```

### Procedure

When you run a job, EJB follows this process:

1. **Build phase**: EJB executes each build script sequentially
  - Build scripts are run sequentially to allow the build script to use every available core to speed up each individual building process.
2. **Execution phase**: EJB executes each run script
  - Run scripts are run in parallel accross different boards and sequentially for each board config.
3. **Result collection**: EJB collects the results from the `results_path`.
  - You can use whatever you want as a way to represent your test results, EJ will simply collect what's inside the `results_path` at the moment the `run_script` ends.

## Next Steps

Congratulations! You now have a very simple but working EJ Builder setup which can already be used to automate your testing environnement. 
The same way we created our Raspberry PI board, we could've just as easily added more board descriptions, there's no limit at how many boards and configs EJB can manage.

The simple shell script approach altough easy to setup, it has some limitations, even for this simple example. If you can't think of any, don't worry, we'll dive into those in [Guide 02 - Builder SDK](02-BuilderSDK.md) which will present these issues and how the `Builder SDK` solves them.
