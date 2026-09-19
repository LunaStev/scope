//! Opt-in measurement output; normal sessions do not log every frame.
use crate::RenderStats;
impl RenderStats {
    pub fn trace_perf(self) {
        if std::env::var_os("SCOPE_PERF").is_some() && self.visible_nodes>0 {
            eprintln!("scope perf: cpu_ms={:.4} built={} reused={} pending={} submitted_runs={} geometry_bytes={}",self.cpu_submit_ms,self.built_tiles,self.reused_tiles,self.pending_tiles,self.submitted_runs,self.retained_bytes);
        }
    }
}
