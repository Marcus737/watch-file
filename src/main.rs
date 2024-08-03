use std::fs;
use std::path::PathBuf;
use std::process::Command;

use clap::Parser;
use log::info;

mod process_util;
mod recorder;


#[derive(Parser, Debug)]
#[command(name = "watch_file", version ="1.0", about="Records and limits the memory and time used by commands", long_about = None)]
struct Cli{
    #[arg(short, long, help = "After reaching the specified timeout duration, how long to wait before forcibly ending the process, in millisecond")]
    kill_wait_time: Option<u128>,
    #[arg(short, long,help = "Limit the running time, in millisecond")]
    time_out: Option<u128>,
    #[arg(short, long, help = "The file path to write the output results of this tool")]
    output_file: Option<PathBuf>,
    #[arg(short, long, help = "Limit memory size, in KB")]
    memory_limit: Option<u128>,
    #[arg(last = true, required = true)]
    command: Vec<String>,
}

///根据command运行一个新进程
fn build_command(command: &Vec<String>) -> Command {
    let program = &command[0];
    let args = &command[1..];
    let mut cmd = Command::new(program);
    cmd.args(args);
    cmd
}

#[test]
fn test_build_command(){
    let cmd = vec![
        String::from("ls"),
        String::from("-l")
    ];
    let command = build_command(&cmd);
    assert_eq!(command.get_program(), "ls");
    assert_eq!(command.get_args().nth(0).unwrap(), "-l")
}

fn main() {
    // set_var("RUST_LOG", "debug");
    env_logger::init();
    
    let cli = Cli::parse();
    
    let cmd = build_command(&cli.command);
    
    let time_result = process_util::watch_process(cmd, cli.time_out, cli.memory_limit, cli.kill_wait_time);
    
    info!("{:?}", time_result);
    
    match cli.output_file {
        None => {
            println!("{}", time_result.serialize_the_results());
        }
        Some(path) => {
            match fs::write(path, time_result.serialize_the_results()) {
                Ok(_) => {}
                Err(err) => {
                    println!("unable write to file: {}", err.to_string());
                }
            }
        }
    }
}


