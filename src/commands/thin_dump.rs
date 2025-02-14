extern crate clap;

use atty::Stream;
use clap::{App, Arg};
use std::path::Path;
use std::process;
use std::sync::Arc;

use crate::commands::utils::*;
use crate::report::*;
use crate::thin::dump::{dump, ThinDumpOptions};
use crate::thin::metadata_repair::SuperblockOverrides;

pub fn run(args: &[std::ffi::OsString]) {
    let parser = App::new("thin_dump")
        .version(crate::version::tools_version())
        .about("Dump thin-provisioning metadata to stdout in XML format")
        // flags
        .arg(
            Arg::with_name("ASYNC_IO")
                .help("Force use of io_uring for synchronous io")
                .long("async-io")
                .hidden(true),
        )
        .arg(
            Arg::with_name("QUIET")
                .help("Suppress output messages, return only exit code.")
                .short("q")
                .long("quiet"),
        )
        .arg(
            Arg::with_name("REPAIR")
                .help("Repair the metadata whilst dumping it")
                .short("r")
                .long("repair"),
        )
        .arg(
            Arg::with_name("SKIP_MAPPINGS")
                .help("Do not dump the mappings")
                .long("skip-mappings"),
        )
        // options
        .arg(
            Arg::with_name("DATA_BLOCK_SIZE")
                .help("Provide the data block size for repairing")
                .long("data-block-size")
                .value_name("SECTORS"),
        )
        .arg(
            Arg::with_name("METADATA_SNAPSHOT")
                .help("Access the metadata snapshot on a live pool")
                .short("m")
                .long("metadata-snapshot")
                .value_name("METADATA_SNAPSHOT"),
        )
        .arg(
            Arg::with_name("NR_DATA_BLOCKS")
                .help("Override the number of data blocks if needed")
                .long("nr-data-blocks")
                .value_name("NUM"),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .help("Specify the output file rather than stdout")
                .short("o")
                .long("output")
                .value_name("FILE"),
        )
        .arg(
            Arg::with_name("TRANSACTION_ID")
                .help("Override the transaction id if needed")
                .long("transaction-id")
                .value_name("NUM"),
        )
        // arguments
        .arg(
            Arg::with_name("INPUT")
                .help("Specify the input device to dump")
                .required(true)
                .index(1),
        );

    let matches = parser.get_matches_from(args);
    let input_file = Path::new(matches.value_of("INPUT").unwrap());
    let output_file = if matches.is_present("OUTPUT") {
        Some(Path::new(matches.value_of("OUTPUT").unwrap()))
    } else {
        None
    };

    let report = std::sync::Arc::new(mk_simple_report());
    check_input_file(input_file, &report);

    let transaction_id = matches.value_of("TRANSACTION_ID").map(|s| {
        s.parse::<u64>().unwrap_or_else(|_| {
            eprintln!("Couldn't parse transaction_id");
            process::exit(1);
        })
    });

    let data_block_size = matches.value_of("DATA_BLOCK_SIZE").map(|s| {
        s.parse::<u32>().unwrap_or_else(|_| {
            eprintln!("Couldn't parse data_block_size");
            process::exit(1);
        })
    });

    let nr_data_blocks = matches.value_of("NR_DATA_BLOCKS").map(|s| {
        s.parse::<u64>().unwrap_or_else(|_| {
            eprintln!("Couldn't parse nr_data_blocks");
            process::exit(1);
        })
    });

    let report;

    if matches.is_present("QUIET") {
        report = std::sync::Arc::new(mk_quiet_report());
    } else if atty::is(Stream::Stdout) {
        report = std::sync::Arc::new(mk_progress_bar_report());
    } else {
        report = Arc::new(mk_simple_report());
    }

    let opts = ThinDumpOptions {
        input: input_file,
        output: output_file,
        async_io: matches.is_present("ASYNC_IO"),
        report: report.clone(),
        repair: matches.is_present("REPAIR"),
        overrides: SuperblockOverrides {
            transaction_id,
            data_block_size,
            nr_data_blocks,
        },
    };

    if let Err(reason) = dump(opts) {
        report.fatal(&format!("{}", reason));
        process::exit(1);
    }
}
