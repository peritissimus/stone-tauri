use std::sync::Mutex;
use tauri::State;
use sysinfo::System;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupMetrics {
    pub app_start_time: f64, // using f64 for JS number compatibility (ms)
    // Optional fields can be added if we track them
    pub total_startup_time: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryMetrics {
    pub heap_used: u64,
    pub heap_total: u64,
    pub external: u64,
    pub rss: u64,
    pub array_buffers: u64,
    pub heap_used_mb: f64,
    pub rss_mb: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CPUMetrics {
    pub user: f64,
    pub system: f64,
    pub percent_cpu: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventLoopMetrics {
    pub lag_ms: f64,
    pub utilization_percent: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelStats {
    pub calls: u64,
    pub errors: u64,
    pub total_duration_ms: f64,
    pub avg_duration_ms: f64,
    pub min_duration_ms: f64,
    pub max_duration_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IPCMetrics {
    pub total_calls: u64,
    pub total_errors: u64,
    pub avg_duration_ms: f64,
    pub p50_duration_ms: f64,
    pub p95_duration_ms: f64,
    pub p99_duration_ms: f64,
    pub calls_by_channel: HashMap<String, ChannelStats>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationStats {
    pub count: u64,
    pub errors: u64,
    pub total_duration_ms: f64,
    pub avg_duration_ms: f64,
    pub min_duration_ms: f64,
    pub max_duration_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseMetrics {
    pub total_queries: u64,
    pub total_errors: u64,
    pub avg_duration_ms: f64,
    pub slow_queries: u64,
    pub queries_by_operation: HashMap<String, OperationStats>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceSnapshot {
    pub timestamp: f64,
    pub uptime: f64,
    pub startup: StartupMetrics,
    pub memory: MemoryMetrics,
    pub cpu: CPUMetrics,
    pub event_loop: EventLoopMetrics,
    pub ipc: IPCMetrics,
    pub database: DatabaseMetrics,
    // Renderer metrics are collected on frontend, can be null here
    pub renderer: Option<serde_json::Value>, 
}

// ============================================================================
// State
// ============================================================================

pub struct PerformanceState {
    sys: Mutex<System>,
    start_time: std::time::Instant,
}

impl PerformanceState {
    pub fn new() -> Self {
        Self {
            sys: Mutex::new(System::new_all()),
            start_time: std::time::Instant::now(),
        }
    }
}

// ============================================================================
// Commands
// ============================================================================

#[tauri::command]
pub async fn get_performance_snapshot(state: State<'_, PerformanceState>) -> Result<PerformanceSnapshot, String> {
    let mut sys = state.sys.lock().map_err(|_| "Failed to lock system state")?;
    
    // Refresh necessary components
    sys.refresh_cpu_all();
    sys.refresh_memory();
    
    let pid = sysinfo::get_current_pid().map_err(|_| "Failed to get PID")?;
    let process = sys.process(pid).ok_or("Failed to get process info")?;
    
    let uptime = state.start_time.elapsed().as_secs_f64();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as f64;

    // Memory
    let rss = process.memory(); // In bytes
    let rss_mb = rss as f64 / 1024.0 / 1024.0;
    
    // CPU
    let cpu_usage = process.cpu_usage(); // This is a percentage (0-100 * cores)
    
    Ok(PerformanceSnapshot {
        timestamp,
        uptime,
        startup: StartupMetrics {
            app_start_time: timestamp - (uptime * 1000.0),
            total_startup_time: None,
        },
        memory: MemoryMetrics {
            heap_used: 0, // Not easily accessible from outside JS VM without specialized crates
            heap_total: 0,
            external: 0,
            rss,
            array_buffers: 0,
            heap_used_mb: 0.0,
            rss_mb,
        },
        cpu: CPUMetrics {
            user: 0.0, // sysinfo provides aggregate usage
            system: 0.0,
            percent_cpu: cpu_usage as f64,
        },
        event_loop: EventLoopMetrics {
            lag_ms: 0.0,
            utilization_percent: 0.0,
        },
        ipc: IPCMetrics {
            total_calls: 0,
            total_errors: 0,
            avg_duration_ms: 0.0,
            p50_duration_ms: 0.0,
            p95_duration_ms: 0.0,
            p99_duration_ms: 0.0,
            calls_by_channel: HashMap::new(),
        },
        database: DatabaseMetrics {
            total_queries: 0,
            total_errors: 0,
            avg_duration_ms: 0.0,
            slow_queries: 0,
            queries_by_operation: HashMap::new(),
        },
        renderer: None,
    })
}

#[tauri::command]
pub async fn get_memory_metrics(state: State<'_, PerformanceState>) -> Result<MemoryMetrics, String> {
    let mut sys = state.sys.lock().map_err(|_| "Failed to lock system state")?;
    sys.refresh_memory();
    
    let pid = sysinfo::get_current_pid().map_err(|_| "Failed to get PID")?;
    let process = sys.process(pid).ok_or("Failed to get process info")?;
    
    let rss = process.memory();
    let rss_mb = rss as f64 / 1024.0 / 1024.0;

    Ok(MemoryMetrics {
        heap_used: 0,
        heap_total: 0,
        external: 0,
        rss,
        array_buffers: 0,
        heap_used_mb: 0.0,
        rss_mb,
    })
}

#[tauri::command]
pub async fn get_cpu_metrics(state: State<'_, PerformanceState>) -> Result<CPUMetrics, String> {
    let mut sys = state.sys.lock().map_err(|_| "Failed to lock system state")?;
    sys.refresh_cpu_all();
    
    let pid = sysinfo::get_current_pid().map_err(|_| "Failed to get PID")?;
    let process = sys.process(pid).ok_or("Failed to get process info")?;
    
    Ok(CPUMetrics {
        user: 0.0,
        system: 0.0,
        percent_cpu: process.cpu_usage() as f64,
    })
}

#[tauri::command]
pub async fn get_ipc_stats() -> Result<IPCMetrics, String> {
    Ok(IPCMetrics {
        total_calls: 0,
        total_errors: 0,
        avg_duration_ms: 0.0,
        p50_duration_ms: 0.0,
        p95_duration_ms: 0.0,
        p99_duration_ms: 0.0,
        calls_by_channel: HashMap::new(),
    })
}

#[tauri::command]
pub async fn get_db_stats() -> Result<DatabaseMetrics, String> {
    Ok(DatabaseMetrics {
        total_queries: 0,
        total_errors: 0,
        avg_duration_ms: 0.0,
        slow_queries: 0,
        queries_by_operation: HashMap::new(),
    })
}

#[tauri::command]
pub async fn get_startup_metrics(state: State<'_, PerformanceState>) -> Result<StartupMetrics, String> {
    let uptime = state.start_time.elapsed().as_secs_f64();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as f64;
        
    Ok(StartupMetrics {
        app_start_time: timestamp - (uptime * 1000.0),
        total_startup_time: None,
    })
}

#[tauri::command]
pub async fn clear_performance_history() -> Result<serde_json::Value, String> {
    // Implement clearing of any accumulated stats if we add them
    Ok(serde_json::json!({ "success": true }))
}
