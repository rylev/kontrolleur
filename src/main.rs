use parity_wasm::{deserialize_buffer, elements::Module};
use std::fs::read;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "kontrolleuer",
    about = "Inspecting what assumptions a wasm binary has about its environment"
)]
struct Options {
    /// Input file
    #[structopt()]
    file: String,
    /// Verbose output
    #[structopt(long = "verbose")]
    verbose: bool,
}

fn main() {
    let options = Options::from_args();
    let contents = read(options.file).expect("Failed to read file");
    let module = deserialize_buffer::<Module>(&contents).unwrap();
    let import_section = module.import_section();
    let mut assumptions = Assumptions::new();
    let entries = import_section.map(|s| s.entries());
    if let Some(entries) = entries {
        for import in entries {
            match import.module() {
                "wasi_unstable" => assumptions.add_wasi(import.field()),
                _ => assumptions.add_unknown(import.field()),
            }
        }
    }

    report(assumptions, options.verbose);
}

struct WasiAssumptions<'a> {
    file_system: Vec<&'a str>,
    environment: Vec<&'a str>,
    process: Vec<&'a str>,
    network: Vec<&'a str>,
    unknown: Vec<&'a str>,
}

impl<'a> WasiAssumptions<'a> {
    fn new() -> WasiAssumptions<'a> {
        WasiAssumptions {
            file_system: Vec::new(),
            environment: Vec::new(),
            process: Vec::new(),
            network: Vec::new(),
            unknown: Vec::new(),
        }
    }
    fn add(&mut self, name: &'a str) {
        match name {
            "args_get" | "args_sizes_get" | "clock_res_get" | "clock_time_get" | "random_get"
            | "environ_get" | "environ_sizes_get" => self.environment.push(name),
            "fd_advise"
            | "fd_close"
            | "fd_datasync"
            | "fd_fdstat_get"
            | "fd_fdstat_set_flags"
            | "fd_fdstat_set_rights"
            | "fd_filestat_get"
            | "fd_filestat_set_size"
            | "fd_filestat_set_times"
            | "fd_pread"
            | "fd_prestat_get"
            | "fd_prestat_dir_name"
            | "fd_pwrite"
            | "fd_read"
            | "fd_readdir"
            | "fd_renumber"
            | "fd_seek"
            | "fd_sync"
            | "fd_tell"
            | "fd_write"
            | "path_create_directory"
            | "path_filestat_get"
            | "path_filestat_set_times"
            | "path_link"
            | "path_open"
            | "path_readlink"
            | "path_remove_directory"
            | "path_rename"
            | "path_symlink"
            | "path_unlink_file"
            | "poll_oneoff" => self.file_system.push(name),
            "proc_exit" | "proc_raise" | "sched_yield" => self.process.push(name),
            "sock_recv" | "sock_send" | "sock_shutdown" => self.network.push(name),
            _ => self.unknown.push(name),
        }
    }

    fn count(&self) -> usize {
        self.file_system.len()
            + self.process.len()
            + self.environment.len()
            + self.network.len()
            + self.unknown.len()
    }
}

struct Assumptions<'a> {
    wasi: WasiAssumptions<'a>,
    unknown: Vec<&'a str>,
}

impl<'a> Assumptions<'a> {
    fn new() -> Assumptions<'a> {
        Assumptions {
            wasi: WasiAssumptions::new(),
            unknown: Vec::new(),
        }
    }

    fn add_wasi(&mut self, name: &'a str) {
        self.wasi.add(name)
    }

    fn add_unknown(&mut self, name: &'a str) {
        self.unknown.push(name)
    }

    fn count(&self) -> usize {
        self.unknown.len() + self.wasi.count()
    }
}
fn report<'a>(assumptions: Assumptions<'a>, verbose: bool) {
    let total_count = assumptions.count();
    println!(
        "There {} {} total external API call{}.",
        correct_to_be_form(total_count),
        total_count,
        optional_s(total_count)
    );

    let wasi = assumptions.wasi;
    let wasi_count = wasi.count();
    if wasi_count > 0 {
        println!("This binary is expecting a WASI compliant runtime.");
        println!(
            "\tThe binary uses {} WASI call{}",
            wasi_count,
            optional_s(wasi_count)
        );
        println!("\tThe following system resource types are used:");
        let mut types = Vec::new();
        if wasi.file_system.len() > 0 {
            types.push("file system");
        }
        if wasi.environment.len() > 0 {
            types.push("environment");
        }
        if wasi.process.len() > 0 {
            types.push("process");
        }
        println!("\t\t{}", types.join(", "));
        if verbose {
            if wasi.file_system.len() > 0 {
                println!("\tFile system calls:");
                for call in wasi.file_system {
                    println!("\t\t{}", call);
                }
            }
            if wasi.environment.len() > 0 {
                println!("\tEnivronent system calls:");
                for call in wasi.environment {
                    println!("\t\t{}", call);
                }
            }
            if wasi.process.len() > 0 {
                println!("\tProcess system calls:");
                for call in wasi.process {
                    println!("\t\t{}", call);
                }
            }
        }
        let unknown = wasi.unknown;
        let unknown_count = unknown.len();
        if unknown_count > 0 {
            println!(
                "There {} {} unknown wasi sys call{}:",
                correct_to_be_form(unknown_count),
                unknown_count,
                optional_s(unknown_count)
            );
            for call in unknown {
                println!("\t{}", call);
            }
        }
    }

    if assumptions.unknown.len() > 0 {
        println!("Unknown imports:");
        for unknown in assumptions.unknown {
            println!("\t{}", unknown);
        }
    }
}

fn correct_to_be_form(count: usize) -> &'static str {
    if count == 1 {
        "is"
    } else {
        "are"
    }
}
fn optional_s(count: usize) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}
