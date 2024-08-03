use std::cmp::max;
use std::time::SystemTime;
use log::info;
use crate::process_util::CTimeStatus;
use crate::process_util::CTimeStatus::MemoryLimitExceed;

///
/// A recorder that records the running time and peak memory of the program and needs to update the data by constantly calling the Update method.
// An error is returned when both a given memory and a given time are exceeded
pub struct Recorder {
    pub memory: u128,
    pub wall_time: u128,
    system_time: SystemTime,
    time_out: Option<u128>,
    memory_limit: Option<u128>,
    factor: u128
}

impl Recorder {
    
    pub fn new(time_out: Option<u128>, memory_limit: Option<u128>) -> Recorder{
        info!("factor:{}", (page_size::get() / 1024) as u128);
        Recorder{
            memory: 0,
            wall_time :0,
            system_time: SystemTime::now(),
            time_out,
            memory_limit,
            factor: (page_size::get() / 1024) as u128
        }
    }

    pub fn update(&mut self, statm: &str)-> Option<CTimeStatus>{
        //按空格分割
        let split:Vec<&str> = statm.split(" ").collect();
        //获取第二个
        let rss:u128 = split[1].parse::<u128>().unwrap() * self.factor;
        //更新最大值
        self.memory = max(self.memory, rss);
        match self.time_out {
            None => {}
            Some(t) => {
                let now = SystemTime::now();
                let duration = now.duration_since(self.system_time).unwrap();
                if duration.as_millis() > t {
                    return Some(CTimeStatus::TimeOut);
                }
            }
        }
        match self.memory_limit {
            None => {}
            Some(m) => {
                if self.memory > m {
                    return Some(MemoryLimitExceed);
                }
            }
        }
        None
    }

    pub fn start_record(&mut self){
        self.system_time = SystemTime::now()
    }

    pub fn end_record(&mut self){
        let now = SystemTime::now();
        let duration = now.duration_since(self.system_time).unwrap();
        self.wall_time =  duration.as_millis()
    }

}