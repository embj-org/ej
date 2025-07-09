# Guide 04: Building custom tools with the Dispatcher SDK

## Overview

In the previous guide, we successfully set up an EJ Dispatcher and connected builders to create a centralized testing infrastructure. While we can now submit jobs and see results through the `ejcli` tool, we haven't yet explored how to programmatically interact with the dispatcher or analyze the results it produces.

This guide demonstrates how to use the EJ Dispatcher SDK to build custom applications that can submit jobs, retrieve results, and perform automated analysis. You'll create a Rust application that connects to your dispatcher, fetches test results, and validates that different configurations of your embedded application produce consistent outputs.

By the end of this guide, you'll understand how to build custom tooling around EJ's dispatcher infrastructure, enabling powerful automation and analysis workflows for your embedded testing pipeline.

## Prerequisites

Before starting this guide, ensure you have:

- **Completed Guide 03**: This guide builds directly on the EJ Dispatcher setup from the previous guide
- **Working EJ Dispatcher setup**: Including:
  - EJD running and accessible
  - At least one builder connected and working
  - Successful job submissions using `ejcli`
  - The `kmer` project results from previous guides
- **Rust toolchain installed**: You'll need `cargo` and the Rust compiler for building the SDK application
  - Install via [rustup.rs](https://rustup.rs/) if you haven't already
- **Basic Rust knowledge helpful**: While we'll explain the code, familiarity with these concepts will be useful:
  - Error handling with `Result` and `?` operator
  - JSON parsing and data structures
  - Basic async programming concepts
  - Package management with `cargo`
- **Understanding of your test results**: You should know what output format your `kmer` application produces and what constitutes a "correct" result

## Step 1: Setting up your rust project

Let's create a custom CLI tool called `ejkmer-dispatcher` that will interact with EJD to submit jobs, retrieve results, and perform analysis on the results produced by the `kmer` project.

```bash
cd ~/ej-workspace
cargo init --bin ejkmer-dispatcher
cd ejkmer-dispatcher
cargo add ej-dispatcher-sdk
cargo add ej-config
cargo add clap -F derive
cargo add tokio -F macros -F rt-multi-thread
```

## Step 2: Using the dispatcher sdk to start a new job

This is pretty straightforward:

```rust
use ej_dispatcher_sdk::prelude::*;
async fn do_run(
    socket: PathBuf,
    seconds: u64,
    commit_hash: String,
    remote_url: String,
) -> Result<()> {
    let job_result = ej_dispatcher_sdk::dispatch_run(
        &socket,
        commit_hash,
        remote_url,
        None,
        Duration::from_secs(seconds),
    )
    .await?;
    println!("{}", job_result);
}
```

The `dispatch_run` function will connect to EJD using the Unix Socket and maintain a connection until the job either finishes or is cancelled.

The job can either be immediately dispatched or put into a queue if there are already running jobs.
Additionnally, the jobs can be cancelled if, by the time the job leaves the queue there are no builders available or if the job times out.

Once we get to this line :

```rust
    println!("{}", job_result);
```

We know the job has finished successfully and we're ready to start parsing the results.

## Step 3: Parse the results

You may be wondering what the type of `job_result` is. If so, here it goes:

```rust
/// Run operation result.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EjRunResult {
    /// Run logs per board configuration.
    pub logs: Vec<(EjBoardConfigApi, String)>,
    /// Run results per board configuration.
    pub results: Vec<(EjBoardConfigApi, String)>,
    /// Whether the run was successful.
    pub success: bool,
}
```

We're mostly interested in the `results: Vec<(EjBoardConfigApi, String)>` which is a dynamic array of `Board Config` and `String` pairs.

- The `EjBoardConfigApi` holds the config ID, name and tags
- The `String` holds the job results. For our specific use case each `String` will have the following format. 
  This is the content of the `results_path` when the `run_script` finishes.

For our example, the results follow this format:

```txt
Results:
ABC: 2
BCD: 1
CDA: 1
DAB: 1

real	0m0.005s
user	0m0.000s
sys	0m0.005s
```

We'll focus on parsing the sequences found and their occurrences, we'll then check to make sure that every config finds the same sequences and the same
number of occurrences per sequence.

Here's an example of a function that does just that:

```rust
struct ConfigResult {
    config: EjBoardConfigApi,
    data: HashMap<String, usize>,
}

fn parse_results(job_result: &EjRunResult) -> Vec<ConfigResult> {
    let mut parsed_results: Vec<ConfigResult> = Vec::new();
    for (board_config, result) in job_result.results.iter() {
        let mut occurrences_map: HashMap<String, usize> = HashMap::new();
        let mut found_start_of_results = false;
        for line in result.lines() {
            if line.contains("Results:") {
                found_start_of_results = true;
                continue;
            }
            if !found_start_of_results {
                continue;
            }
            if line.contains(':') {
                let splitted: Vec<&str> = line.split(": ").collect();
                assert_eq!(splitted.len(), 2);
                let sequence = splitted[0];
                let n_occurences = splitted[1]
                    .parse()
                    .expect("Expected number on right side of ':'");
                occurrences_map.insert(sequence.to_string(), n_occurences);
            }
        }
        parsed_results.push(ConfigResult {
            config: board_config.clone(),
            data: occurrences_map,
        });
    }
    parsed_results
}
```

## Step 4: Check the results

Once we have the results parsed, it makes it easier to reason with the code that actually checks that the results are valid:

```rust
fn check_results(parsed_results: &Vec<ConfigResult>) {
    for i in 0..parsed_results.len() {
        for j in (i + 1)..parsed_results.len() {
            let config_i = &parsed_results[i].config;
            let config_j = &parsed_results[j].config;
            let data_i = &parsed_results[i].data;
            let data_j = &parsed_results[j].data;

            assert_eq!(
                data_i.len(),
                data_j.len(),
                "Different number of sequences for {} and {} {} vs {}",
                config_i.name,
                config_j.name,
                data_i.len(),
                data_j.len(),
            );

            for (sequence, expected) in parsed_results[i].data.iter() {
                let actual = data_j.get(sequence);
                assert!(
                    actual.is_some(),
                    "Couldn't find {} in {}",
                    sequence,
                    config_j.name
                );

                let actual = actual.unwrap();

                assert_eq!(
                    expected, actual,
                    "Expected {} occurrences for {}. Got {} ",
                    expected, sequence, actual
                );
            }
        }
    }
}
```

## Step 5: Completing our `do_run` function


Once we can parse and check the results, the only thing left to do is to use these functions to make sure our run was successful:

```rust
async fn do_run(
    socket: PathBuf,
    seconds: u64,
    commit_hash: String,
    remote_url: String,
) -> Result<()> {
    let job_result = ej_dispatcher_sdk::dispatch_run(
        &socket,
        commit_hash,
        remote_url,
        None,
        Duration::from_secs(seconds),
    )
    .await?;
    println!("{}", job_result);

    if !job_result.success {
        return Err(Error::RunError);
    }
    let parsed_results = parse_results(&job_result);
    check_results(&parsed_results);

    Ok(())
}
```

## Step 6: Add a CLI interface

We'll also add a basic CLI interface that will make it easier to add new commands down the line.
For this we'll use `clap` that we've already added to our project.
The following code should be pretty straight forward to reason with:

```rust
use clap::{Parser, Subcommand};
#[derive(Parser)]
#[command(name = "ejkmer-dispatcher")]
#[command(about = "EJ Kmer Dispatcher - Job dispatcher and result handler for the Kmer project")]
struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    DispatchRun {
        /// Path to the EJD's unix socket
        #[arg(short, long)]
        socket: PathBuf,
        /// The maximum job duration in seconds
        #[arg(long)]
        seconds: u64,
        /// Git commit hash
        #[arg(long)]
        commit_hash: String,
        /// Git remote url
        #[arg(long)]
        remote_url: String,
    },
}
```

## Step 7: Putting everything together

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::DispatchRun {
            socket,
            seconds,
            commit_hash,
            remote_url,
        } => do_run(socket, seconds, commit_hash, remote_url).await,
    }
}
```

`clap` will automatically parse the arguments for you to make sure that everything works correctly.
You can now try your new application with the same arguments as the `ej-cli`

## Step 8: Try it out


First, remove the `infinite-loop` config as it doesn't do anything useful.

Remove this entry from `~/ej-workspace/config.toml`:

```toml
[[boards.configs]]
name = "infinite-loop"
tags = ["arm64", "infinite-loop"]
build_script = "ejkmer-builder"
run_script = "ejkmer-builder"
results_path = "/home/<user>/ej-workspace/results_infinite-loop.txt"
library_path = "/home/<user>/ej-workspace/kmer"
```

And now run your new program:

```bash
cd ~/ej-workspace/ejkmer-dispatcher/
cargo run -- dispatch-run --socket ~/ejd-deployment/ejd/tmp/ejd.sock --seconds 60 --commit-hash eb7c6cbe6249aff4df82455bbadf4898b0167d09 --remote-url https://github.com/embj-org/kmer
```

```bash
=======================================
Run finished successfully with 3 log entries:
=======================================
84b19f1e-66c1-4182-a428-c4357be4d9a4 - k-mer [arm64,kmer optimized]
=======================================
From https://github.com/embj-org/kmer
 * [new branch]      main       -> ejupstream/main
HEAD is now at eb7c6cb feat: add infinite loop example
-- Configuring done (0.0s)
-- Generating done (0.0s)
-- Build files have been written to: /home/andre/ej-workspace/kmer/build-pi
[ 50%] Built target infinite-loop
[ 50%] Built target k-mer-original
[ 75%] Built target k-mer-omp
[100%] Built target k-mer
Results:
ABC: 2
BCD: 1
CDA: 1
DAB: 1

real	0m0.003s
user	0m0.003s
sys	0m0.000s

=======================================
51ba26d0-b71e-444a-9732-9a33e51dd4dd - k-mer-omp [arm64,kmer multi-threaded optimized]
=======================================
From https://github.com/embj-org/kmer
 * [new branch]      main       -> ejupstream/main
HEAD is now at eb7c6cb feat: add infinite loop example
-- Configuring done (0.0s)
-- Generating done (0.0s)
-- Build files have been written to: /home/andre/ej-workspace/kmer/build-pi
[ 50%] Built target k-mer-original
[ 50%] Built target infinite-loop
[100%] Built target k-mer-omp
[100%] Built target k-mer
Results:
ABC: 2
BCD: 1
CDA: 1
DAB: 1

real	0m0.004s
user	0m0.004s
sys	0m0.003s

=======================================
734cdfb0-85aa-4405-af5a-752d68f5c003 - k-mer-original [arm64,kmer unoptimized]
=======================================
From https://github.com/embj-org/kmer
 * [new branch]      main       -> ejupstream/main
HEAD is now at eb7c6cb feat: add infinite loop example
-- Configuring done (0.0s)
-- Generating done (0.0s)
-- Build files have been written to: /home/andre/ej-workspace/kmer/build-pi
[ 75%] Built target k-mer-original
[ 75%] Built target k-mer
[ 75%] Built target infinite-loop
[100%] Built target k-mer-omp
Results:
ABC: 2
BCD: 1
CDA: 1
DAB: 1

real	0m0.003s
user	0m0.003s
sys	0m0.000s

=======================================

=======================================
Run finished successfully with 3 result entries:
=======================================
734cdfb0-85aa-4405-af5a-752d68f5c003 - k-mer-original [arm64,kmer unoptimized]
=======================================
Results:
ABC: 2
BCD: 1
CDA: 1
DAB: 1

real	0m0.005s
user	0m0.005s
sys	0m0.001s

=======================================
84b19f1e-66c1-4182-a428-c4357be4d9a4 - k-mer [arm64,kmer optimized]
=======================================
Results:
ABC: 2
BCD: 1
CDA: 1
DAB: 1

real	0m0.005s
user	0m0.000s
sys	0m0.005s

=======================================
51ba26d0-b71e-444a-9732-9a33e51dd4dd - k-mer-omp [arm64,kmer multi-threaded optimized]
=======================================
Results:
ABC: 2
BCD: 1
CDA: 1
DAB: 1

real	0m0.007s
user	0m0.000s
sys	0m0.010s

=======================================

Results OK!
```

It's useful to print the job results as it makes it easier to debug later on.

## What's Next

Congratulations! You've now built a complete embedded testing infrastructure with EJ, from basic builder setup to advanced dispatcher integration with custom analysis tools. However, this is just the beginning of what's possible with EJ.

### Beyond Simple Applications

Throughout these guides, we've used the `kmer` application as our example a relatively simple C program that processes text files.
The real power of EJ becomes apparent when you apply it to more complex embedded applications:

- **Multi-component systems**: Applications with multiple executables, libraries, and configuration files
- **Real-time systems**: Programs that interact with hardware peripherals, sensors, or communication protocols
- **Performance-critical applications**: Code that needs to be tested across different optimization levels and compiler flags
- **Cross-platform applications**: Software that must run on multiple embedded architectures (ARM, RISC-V, x86 embedded, etc.)

Since EJ works with any application that can be built and deployed without manual intervention, you can integrate virtually any embedded project.
The key is writing appropriate build and deployment scripts (whether shell scripts or using the Builder SDK) that handle your specific application's requirements.

### Advanced Result Analysis and Integration

Our result validation example simply checked that different configurations produced identical outputs.
In real-world scenarios, you can implement much more sophisticated analysis and integration workflows:

#### Communication and Notifications

- **Slack integration**: Send notifications when tests complete, fail, or show performance regressions
- **Email reports**: Generate detailed test summaries and email them to your team
- **GitHub/GitLab PR comments**: Automatically comment on pull requests with test results and performance metrics

#### Continuous Integration Workflows

- **Performance trend analysis**: Compare current results with historical data to detect regressions
- **Automated benchmarking**: Track performance metrics over time and alert on significant changes
- **Cross-platform validation**: Ensure your application behaves consistently across different hardware platforms

#### Result Presentation and Documentation

- **HTML report generation**: Create rich, interactive web pages showing test results with charts and graphs
- **Dashboard creation**: Build real-time dashboards showing the health of your embedded systems
- **Automated documentation**: Generate performance reports and system specifications based on test results

### Final Thoughts

EJ transforms embedded testing from a manual, error-prone process into a reliable, automated pipeline.
By centralizing job management, providing proper cancellation handling, and offering programmatic access to results,
EJ enables the same level of testing sophistication in embedded development that web developers take for granted.

The journey from a simple shell script to a full distributed testing infrastructure demonstrates how EJ scales with your needs - start simple, then add complexity as your requirements grow.
From a solo developer with a single Raspberry Pi to a company managing hundreds of embedded devices EJ provides the tools to build a testing infrastructure that grows with your project.

Happy testing!

---

**Community**: Share your custom tools and integrations with the EJ community to help others solve similar challenges.
