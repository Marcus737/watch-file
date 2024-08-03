use std::{fs, io};
use std::path::PathBuf;
use std::process::Command;
use std::thread::{sleep, spawn};
use std::time::Duration;
use log::info;
use crate::recorder::Recorder;

fn read_statm(path: &PathBuf) -> io::Result<String>{
    fs::read_to_string(path)
}

pub fn watch_process(mut command: Command, time_out: Option<u128>, memory_limit: Option<u128>, kill_wait_time: Option<u128>) -> CTimeResult {

    let mut child = command.spawn().unwrap();

    let join_handle = spawn(move || {
        let path = PathBuf::from(format!("/proc/{}/statm", child.id()));
        info!("{:?}", path);
        
        let mut recorder = Recorder::new(time_out, memory_limit);
        recorder.start_record();
        
        let mut res: Option<CTimeResult> = None;
        let mut exit_code = 0;
        //0.5毫秒
        let delay = Duration::from_nanos(500000);
        loop {
            match read_statm(&path) {
                Ok(info) => {
                    match recorder.update(&info) {
                        None => {}
                        Some(s) => {
                            match s {
                                //结束超时进程
                                CTimeStatus::TimeOut | CTimeStatus::MemoryLimitExceed => {
                                    match kill_wait_time {
                                        None => {}
                                        Some(t) => {
                                            sleep(Duration::from_millis(t as u64));
                                        }
                                    }
                                    child.kill().unwrap();
                                }
                                _ => {}
                            }
                            res = Some(CTimeResult::build_with_status(s));
                            break;
                        }
                    }
                }
                Err(_) => {
                    break;
                }
            }
            match child.try_wait() {
                Ok(Some(status)) => {
                    exit_code = status.code().unwrap();
                    break            
                }
                Err(_e) => {
                    break
                }
                Ok(None) => {}
            }
            sleep(delay);
        }
        recorder.end_record();
        return match res {
            None => {
                let mut result = CTimeResult::build_ok_result(recorder.wall_time, recorder.memory);
                result.exit_code = exit_code;
                result
            }
            Some(mut r) => {
                r.exit_code = exit_code;
                r
            }
        }
    });
    
    join_handle.join().unwrap()
}


#[test]
fn test_watch_process(){
    let mut command = Command::new("ls");
    command.arg("-l");
    let result = watch_process(command, Some(1000), Some(10000), Some(100));
    assert_eq!(result.exit_code, 0);
}

#[derive(Debug)]
pub enum CTimeStatus{
    OK,
    TimeOut,
    MemoryLimitExceed
}


#[derive(Debug)]
pub struct CTimeResult{
    exit_code: i32,
    memory: u128,
    wall_time: u128,
    status: CTimeStatus
}

impl CTimeResult {

    pub fn build_with_status(ctime_status: CTimeStatus) -> CTimeResult {
        CTimeResult{
            exit_code: 1,
            memory: 0,
            wall_time: 0,
            status: ctime_status,
        }
    }

    pub fn build_ok_result(wall_time: u128, memory: u128) -> CTimeResult{
        CTimeResult{
            exit_code: 0,
            memory,
            wall_time,
            status: CTimeStatus::OK,
        }
    }

    pub fn serialize_the_results(&self) -> String {
        let mut string = String::new();
        string.push_str(&format!("exit_code={}\n", self.exit_code));
        string.push_str(&format!("memory={}\n", self.memory));
        string.push_str(&format!("wall_time={}\n", self.wall_time));
        string.push_str(&format!("status={:?}\n", self.status));
        string
    }
    
}