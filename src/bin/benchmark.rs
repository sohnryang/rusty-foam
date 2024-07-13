use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

use clap::{Parser, Subcommand};
use rusty_foam::byte_stream::ByteStream;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Command {
    /// Benchmark byte stream implementation
    ByteStream {
        /// Capacity of byte stream
        #[arg(long, default_value_t = 4096)]
        capacity: usize,

        /// Number of read/write cycles
        #[arg(long, default_value_t = 1024)]
        cycles: usize,

        /// Write size
        #[arg(long, default_value_t = 1024)]
        write_size: usize,

        /// Number of writes per cycle
        #[arg(long, default_value_t = 1)]
        writes_per_cycle: usize,

        /// Read size
        #[arg(long, default_value_t = 1024)]
        read_size: usize,

        /// Number of reads per cycle
        #[arg(long, default_value_t = 1)]
        reads_per_cycle: usize,
    },
}

fn benchmark_byte_stream(
    mut byte_stream: ByteStream,
    cycles: usize,
    write_size: usize,
    writes_per_cycle: usize,
    read_size: usize,
    reads_per_cycle: usize,
) -> io::Result<Duration> {
    // Initialize corpus
    let corpus: Vec<u8> = (b'A'..=b'Z')
        .chain(b'a'..=b'z')
        .chain(b'0'..=b'9')
        .cycle()
        .take(write_size * writes_per_cycle * cycles)
        .collect();

    let mut read_buffer = vec![0u8; read_size * reads_per_cycle * cycles];
    let start_time = Instant::now();

    // Perform write-read cycles
    for cycle in 0..cycles {
        // Writes for this cycle
        for write in 0..writes_per_cycle {
            let start = (cycle * writes_per_cycle + write) * write_size;
            let end = start + write_size;
            byte_stream.write_all(&corpus[start..end])?;
        }

        // Reads for this cycle
        for read in 0..reads_per_cycle {
            let read_start = (cycle * reads_per_cycle + read) * read_size;
            let read_end = read_start + read_size;
            byte_stream.read_exact(&mut read_buffer[read_start..read_end])?;
        }
    }

    let duration = start_time.elapsed();

    // Consistency check (outside of timed section)
    let expected_data: Vec<u8> = corpus
        .iter()
        .cycle()
        .take(read_size * reads_per_cycle * cycles)
        .cloned()
        .collect();

    if read_buffer != expected_data {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Inconsistency detected in read data",
        ));
    }

    Ok(duration)
}

fn main() -> io::Result<()> {
    let args = Cli::parse();
    match args.command {
        Command::ByteStream {
            capacity,
            cycles,
            write_size,
            writes_per_cycle,
            read_size,
            reads_per_cycle,
        } => {
            let byte_stream = ByteStream::new(capacity);
            let duration = benchmark_byte_stream(
                byte_stream,
                cycles,
                write_size,
                writes_per_cycle,
                read_size,
                reads_per_cycle,
            )?;
            println!("Elapsed: {duration:?}");
        }
    }

    Ok(())
}
