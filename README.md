# compact-waitgroup

[![Crates.io](https://img.shields.io/crates/v/compact-waitgroup.svg)](https://crates.io/crates/compact-waitgroup)
[![Docs.rs](https://img.shields.io/docsrs/compact-waitgroup.svg)](https://docs.rs/compact-waitgroup)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-informational.svg)](#license)
[![Build status](https://github.com/ChieloNewctle/compact-waitgroup/actions/workflows/ci.yml/badge.svg)](https://github.com/ChieloNewctle/compact-waitgroup/actions)

A compact asynchronous `WaitGroup` synchronization primitive.

This crate is designed to be lightweight and executor-agnostic. It works with
any `async` runtime and supports `no_std` environments (requires `alloc`).

## Usage

### `MonoWaitGroup`

Using `MonoWaitGroup` for a single task:

```rust
use std::{thread, time::Duration};

use compact_waitgroup::MonoWaitGroup;
use futures_executor::block_on;

fn main() {
    let (wg, token) = MonoWaitGroup::new();

    thread::spawn(move || {
        println!("Worker started");
        // Long-running task...
        thread::sleep(Duration::from_secs(1));
        println!("Worker finished");
        // Token is released here, signaling completion
        token.release();
    });

    block_on(async {
        // Wait for the task to complete
        wg.await;
        println!("All done!");
    });
}
```

### `WaitGroup`

Using `WaitGroup` for multiple tasks:

```rust
use std::{iter::repeat_n, thread, time::Duration};

use compact_waitgroup::{GroupTokenFuncExt, WaitGroup};
use futures_executor::block_on;

fn main() {
    let (wg, factory) = WaitGroup::new();

    for (i, token) in repeat_n(factory.into_token(), 8).enumerate() {
        let task = move || {
            println!("Task {i} started");
            // Long-running task...
            thread::sleep(Duration::from_secs(1));
            println!("Task {i} finished");
        };
        // Token will be released when the task is done
        thread::spawn(task.release_on_return(token));
    }

    block_on(async {
        // Wait for the tasks to complete
        wg.await;
        println!("All done!");
    });
}
```

### Tokio Example

Works seamlessly with Tokio:

```rust
use std::{iter::repeat_n, time::Duration};

use compact_waitgroup::{GroupTokenExt, WaitGroup};
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let (wg, factory) = WaitGroup::new();

    factory.scope(|token| {
        for (i, token) in repeat_n(token, 8).enumerate() {
            let task = async move {
                println!("Task {i} started");
                // Long-running task...
                sleep(Duration::from_secs(1)).await;
                println!("Task {i} finished");
            };
            // Token will be released when the future is ready
            tokio::spawn(task.release_on_ready(token));
        }
    });

    // Wait for the tasks to complete
    tokio::pin!(wg);
    loop {
        tokio::select! {
            _ = sleep(Duration::from_millis(200)) => {
                println!("Running...");
            }
            _ = &mut wg => {
                break;
            }
        };
    }
    println!("All done!");
}
```

## Memory Layout

This crate is optimized for size. By enabling the `compact-mono` feature,
`MonoWaitGroup` becomes even smaller by removing the unnecessary reference
counter.

| Component           | Default (64-bit) | With `compact-mono` | Saving      |
| ------------------- | ---------------- | ------------------- | ----------- |
| **`WaitGroup`**     | 32 bytes         | 32 bytes            | 0 bytes     |
| **`MonoWaitGroup`** | **32 bytes**     | **24 bytes**        | **8 bytes** |

## License

- &copy; 2026 Chielo Newctle
  \<[ChieloNewctle@gmail.com](mailto:ChieloNewctle@gmail.com)\>

Licensed under either of

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
  ([`LICENSE-APACHE`](LICENSE-APACHE))
- [MIT license](https://opensource.org/licenses/MIT)
  ([`LICENSE-MIT`](LICENSE-MIT))

at your option.
