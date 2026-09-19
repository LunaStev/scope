pub const DEFAULT_EXCLUDES:&[&str]=&["target","node_modules","build","dist",".venv","venv","__pycache__",".cache"];
#[derive(Clone,Debug)]
pub struct ScanOptions{pub respect_ignore:bool,pub default_excludes:bool,pub excludes:Vec<String>,pub max_file_bytes:u64,pub threads:usize}
impl Default for ScanOptions{
    fn default()->Self{Self{threads:0,respect_ignore:true,default_excludes:true,excludes:Vec::new(),max_file_bytes:8*1024*1024}}
}
impl ScanOptions{
    pub fn worker_count(&self)->usize{if self.threads==0{std::thread::available_parallelism().map_or(2,usize::from).clamp(1,8)}else{self.threads.clamp(1,32)}}
}
