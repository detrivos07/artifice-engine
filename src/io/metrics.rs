use artifice_logging::{debug, info, trace, warn};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

/// Performance metrics for the event system
#[derive(Debug, Clone)]
pub struct EventSystemMetrics {
    /// Total number of events processed
    pub events_processed: u64,
    /// Total number of events dropped due to queue full
    pub events_dropped: u64,
    /// Average event processing time in microseconds
    pub avg_processing_time_us: f64,
    /// Peak event processing time in microseconds
    pub peak_processing_time_us: u64,
    /// Events processed per second
    pub events_per_second: f64,
    /// Queue utilization (0.0 to 1.0)
    pub queue_utilization: f64,
    /// Memory usage for events in bytes
    pub memory_usage_bytes: u64,
    /// Metrics by event type
    pub event_type_metrics: HashMap<String, EventTypeMetrics>,
}

/// Metrics for a specific event type
#[derive(Debug, Clone)]
pub struct EventTypeMetrics {
    pub count: u64,
    pub avg_processing_time_us: f64,
    pub peak_processing_time_us: u64,
    pub total_processing_time_us: u64,
}

impl EventTypeMetrics {
    pub fn new() -> Self {
        Self {
            count: 0,
            avg_processing_time_us: 0.0,
            peak_processing_time_us: 0,
            total_processing_time_us: 0,
        }
    }

    pub fn update(&mut self, processing_time_us: u64) {
        self.count += 1;
        self.total_processing_time_us += processing_time_us;
        self.avg_processing_time_us = self.total_processing_time_us as f64 / self.count as f64;
        if processing_time_us > self.peak_processing_time_us {
            self.peak_processing_time_us = processing_time_us;
        }
    }
}

/// Atomic counters for thread-safe metrics collection
struct AtomicMetrics {
    events_processed: AtomicU64,
    events_dropped: AtomicU64,
    total_processing_time_us: AtomicU64,
    peak_processing_time_us: AtomicU64,
    queue_size: AtomicUsize,
    queue_capacity: AtomicUsize,
    memory_usage_bytes: AtomicU64,
}

impl AtomicMetrics {
    fn new() -> Self {
        Self {
            events_processed: AtomicU64::new(0),
            events_dropped: AtomicU64::new(0),
            total_processing_time_us: AtomicU64::new(0),
            peak_processing_time_us: AtomicU64::new(0),
            queue_size: AtomicUsize::new(0),
            queue_capacity: AtomicUsize::new(0),
            memory_usage_bytes: AtomicU64::new(0),
        }
    }
}

/// Thread-safe metrics collector for the event system
pub struct MetricsCollector {
    atomic_metrics: Arc<AtomicMetrics>,
    event_type_metrics: Arc<RwLock<HashMap<String, EventTypeMetrics>>>,
    start_time: Instant,
    last_snapshot_time: Arc<Mutex<Instant>>,
    collection_enabled: Arc<std::sync::atomic::AtomicBool>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            atomic_metrics: Arc::new(AtomicMetrics::new()),
            event_type_metrics: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
            last_snapshot_time: Arc::new(Mutex::new(Instant::now())),
            collection_enabled: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        }
    }

    /// Enable or disable metrics collection
    pub fn set_enabled(&self, enabled: bool) {
        self.collection_enabled.store(enabled, Ordering::Relaxed);
        if enabled {
            info!("Event system metrics collection enabled");
        } else {
            info!("Event system metrics collection disabled");
        }
    }

    /// Check if metrics collection is enabled
    pub fn is_enabled(&self) -> bool {
        self.collection_enabled.load(Ordering::Relaxed)
    }

    /// Record an event being processed
    pub fn record_event_processed(&self, event_type: &str, processing_time: Duration) {
        if !self.collection_enabled.load(Ordering::Relaxed) {
            return;
        }

        let processing_time_us = processing_time.as_micros() as u64;
        
        // Update atomic counters
        self.atomic_metrics.events_processed.fetch_add(1, Ordering::Relaxed);
        self.atomic_metrics.total_processing_time_us.fetch_add(processing_time_us, Ordering::Relaxed);
        
        // Update peak processing time
        let mut current_peak = self.atomic_metrics.peak_processing_time_us.load(Ordering::Relaxed);
        while processing_time_us > current_peak {
            match self.atomic_metrics.peak_processing_time_us.compare_exchange_weak(
                current_peak,
                processing_time_us,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_current) => current_peak = new_current,
            }
        }

        // Update event type metrics
        if let Ok(mut metrics) = self.event_type_metrics.write() {
            let entry = metrics.entry(event_type.to_string()).or_insert_with(EventTypeMetrics::new);
            entry.update(processing_time_us);
        }

        trace!("Recorded event: {} ({:.2}μs)", event_type, processing_time_us);
    }

    /// Record an event being dropped
    pub fn record_event_dropped(&self, event_type: &str) {
        if !self.collection_enabled.load(Ordering::Relaxed) {
            return;
        }

        self.atomic_metrics.events_dropped.fetch_add(1, Ordering::Relaxed);
        warn!("Event dropped: {}", event_type);
    }

    /// Update queue metrics
    pub fn update_queue_metrics(&self, current_size: usize, capacity: usize) {
        if !self.collection_enabled.load(Ordering::Relaxed) {
            return;
        }

        self.atomic_metrics.queue_size.store(current_size, Ordering::Relaxed);
        self.atomic_metrics.queue_capacity.store(capacity, Ordering::Relaxed);
    }

    /// Update memory usage metrics
    pub fn update_memory_usage(&self, bytes: u64) {
        if !self.collection_enabled.load(Ordering::Relaxed) {
            return;
        }

        self.atomic_metrics.memory_usage_bytes.store(bytes, Ordering::Relaxed);
    }

    /// Get current metrics snapshot
    pub fn get_metrics(&self) -> EventSystemMetrics {
        let events_processed = self.atomic_metrics.events_processed.load(Ordering::Relaxed);
        let total_processing_time_us = self.atomic_metrics.total_processing_time_us.load(Ordering::Relaxed);
        let queue_size = self.atomic_metrics.queue_size.load(Ordering::Relaxed);
        let queue_capacity = self.atomic_metrics.queue_capacity.load(Ordering::Relaxed);

        // Calculate events per second
        let elapsed = self.start_time.elapsed();
        let events_per_second = if elapsed.as_secs() > 0 {
            events_processed as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        // Calculate average processing time
        let avg_processing_time_us = if events_processed > 0 {
            total_processing_time_us as f64 / events_processed as f64
        } else {
            0.0
        };

        // Calculate queue utilization
        let queue_utilization = if queue_capacity > 0 {
            queue_size as f64 / queue_capacity as f64
        } else {
            0.0
        };

        // Get event type metrics
        let event_type_metrics = self.event_type_metrics
            .read()
            .map(|metrics| metrics.clone())
            .unwrap_or_default();

        EventSystemMetrics {
            events_processed,
            events_dropped: self.atomic_metrics.events_dropped.load(Ordering::Relaxed),
            avg_processing_time_us,
            peak_processing_time_us: self.atomic_metrics.peak_processing_time_us.load(Ordering::Relaxed),
            events_per_second,
            queue_utilization,
            memory_usage_bytes: self.atomic_metrics.memory_usage_bytes.load(Ordering::Relaxed),
            event_type_metrics,
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.atomic_metrics.events_processed.store(0, Ordering::Relaxed);
        self.atomic_metrics.events_dropped.store(0, Ordering::Relaxed);
        self.atomic_metrics.total_processing_time_us.store(0, Ordering::Relaxed);
        self.atomic_metrics.peak_processing_time_us.store(0, Ordering::Relaxed);
        self.atomic_metrics.memory_usage_bytes.store(0, Ordering::Relaxed);

        if let Ok(mut metrics) = self.event_type_metrics.write() {
            metrics.clear();
        }

        debug!("Event system metrics reset");
    }

    /// Get a cloneable handle for multi-threaded access
    pub fn get_handle(&self) -> MetricsHandle {
        MetricsHandle {
            atomic_metrics: self.atomic_metrics.clone(),
            event_type_metrics: self.event_type_metrics.clone(),
            enabled: self.collection_enabled.clone(),
        }
    }

    /// Print metrics summary to log
    pub fn log_metrics_summary(&self) {
        let metrics = self.get_metrics();
        
        info!("=== Event System Metrics Summary ===");
        info!("Events Processed: {}", metrics.events_processed);
        info!("Events Dropped: {}", metrics.events_dropped);
        info!("Events/Second: {:.2}", metrics.events_per_second);
        info!("Avg Processing Time: {:.2}μs", metrics.avg_processing_time_us);
        info!("Peak Processing Time: {}μs", metrics.peak_processing_time_us);
        info!("Queue Utilization: {:.1}%", metrics.queue_utilization * 100.0);
        info!("Memory Usage: {:.2}KB", metrics.memory_usage_bytes as f64 / 1024.0);
        
        if !metrics.event_type_metrics.is_empty() {
            info!("--- Event Type Breakdown ---");
            for (event_type, type_metrics) in &metrics.event_type_metrics {
                info!(
                    "{}: {} events, {:.2}μs avg, {}μs peak",
                    event_type,
                    type_metrics.count,
                    type_metrics.avg_processing_time_us,
                    type_metrics.peak_processing_time_us
                );
            }
        }
        info!("=====================================");
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Lightweight handle for metrics collection in multi-threaded contexts
#[derive(Clone)]
pub struct MetricsHandle {
    atomic_metrics: Arc<AtomicMetrics>,
    event_type_metrics: Arc<RwLock<HashMap<String, EventTypeMetrics>>>,
    enabled: Arc<std::sync::atomic::AtomicBool>,
}

impl MetricsHandle {
    /// Record an event being processed
    pub fn record_event_processed(&self, event_type: &str, processing_time: Duration) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }

        let processing_time_us = processing_time.as_micros() as u64;
        
        self.atomic_metrics.events_processed.fetch_add(1, Ordering::Relaxed);
        self.atomic_metrics.total_processing_time_us.fetch_add(processing_time_us, Ordering::Relaxed);
        
        let mut current_peak = self.atomic_metrics.peak_processing_time_us.load(Ordering::Relaxed);
        while processing_time_us > current_peak {
            match self.atomic_metrics.peak_processing_time_us.compare_exchange_weak(
                current_peak,
                processing_time_us,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_current) => current_peak = new_current,
            }
        }

        if let Ok(mut metrics) = self.event_type_metrics.write() {
            let entry = metrics.entry(event_type.to_string()).or_insert_with(EventTypeMetrics::new);
            entry.update(processing_time_us);
        }
    }

    /// Record an event being dropped
    pub fn record_event_dropped(&self, _event_type: &str) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }

        self.atomic_metrics.events_dropped.fetch_add(1, Ordering::Relaxed);
    }
}

/// Automatic metrics timer that records processing time when dropped
pub struct MetricsTimer {
    handle: Option<MetricsHandle>,
    event_type: String,
    start_time: Instant,
}

impl MetricsTimer {
    /// Create a new metrics timer
    pub fn new(handle: MetricsHandle, event_type: impl Into<String>) -> Self {
        Self {
            handle: Some(handle),
            event_type: event_type.into(),
            start_time: Instant::now(),
        }
    }

    /// Create a disabled timer that doesn't record metrics
    pub fn disabled() -> Self {
        Self {
            handle: None,
            event_type: String::new(),
            start_time: Instant::now(),
        }
    }

    /// Manually finish the timer and record the result
    pub fn finish(mut self) {
        if let Some(handle) = self.handle.take() {
            let elapsed = self.start_time.elapsed();
            handle.record_event_processed(&self.event_type, elapsed);
        }
    }
}

impl Drop for MetricsTimer {
    fn drop(&mut self) {
        if let Some(handle) = &self.handle {
            let elapsed = self.start_time.elapsed();
            handle.record_event_processed(&self.event_type, elapsed);
        }
    }
}

/// Periodic metrics reporter that logs metrics at regular intervals
pub struct MetricsReporter {
    collector: Arc<MetricsCollector>,
    report_interval: Duration,
    last_report: Instant,
    enabled: bool,
}

impl MetricsReporter {
    pub fn new(collector: Arc<MetricsCollector>, report_interval: Duration) -> Self {
        Self {
            collector,
            report_interval,
            last_report: Instant::now(),
            enabled: false,
        }
    }

    /// Enable periodic reporting
    pub fn enable(&mut self) {
        self.enabled = true;
        self.last_report = Instant::now();
        info!("Metrics reporting enabled (interval: {:?})", self.report_interval);
    }

    /// Disable periodic reporting
    pub fn disable(&mut self) {
        self.enabled = false;
        info!("Metrics reporting disabled");
    }

    /// Check if reporting is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set the reporting interval
    pub fn set_interval(&mut self, interval: Duration) {
        self.report_interval = interval;
        debug!("Metrics reporting interval set to {:?}", interval);
    }

    /// Update the reporter (call this periodically, e.g., in the main loop)
    pub fn update(&mut self) {
        if !self.enabled {
            return;
        }

        if self.last_report.elapsed() >= self.report_interval {
            self.collector.log_metrics_summary();
            self.last_report = Instant::now();
        }
    }

    /// Force an immediate report
    pub fn report_now(&mut self) {
        self.collector.log_metrics_summary();
        self.last_report = Instant::now();
    }
}

/// Configuration for metrics collection
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Enable/disable metrics collection
    pub enabled: bool,
    /// Enable automatic periodic reporting
    pub auto_reporting: bool,
    /// Interval for automatic reporting
    pub report_interval: Duration,
    /// Maximum number of event types to track (to prevent memory growth)
    pub max_event_types: usize,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_reporting: false,
            report_interval: Duration::from_secs(60),
            max_event_types: 100,
        }
    }
}

/// Factory for creating metrics components with configuration
pub struct MetricsFactory;

impl MetricsFactory {
    /// Create a metrics collector with the given configuration
    pub fn create_collector(config: &MetricsConfig) -> MetricsCollector {
        let mut collector = MetricsCollector::new();
        collector.set_enabled(config.enabled);
        collector
    }

    /// Create a metrics reporter with the given configuration
    pub fn create_reporter(
        collector: Arc<MetricsCollector>,
        config: &MetricsConfig,
    ) -> MetricsReporter {
        let mut reporter = MetricsReporter::new(collector, config.report_interval);
        if config.auto_reporting {
            reporter.enable();
        }
        reporter
    }

    /// Create a complete metrics system with the given configuration
    pub fn create_system(config: &MetricsConfig) -> (Arc<MetricsCollector>, MetricsReporter) {
        let collector = Arc::new(Self::create_collector(config));
        let reporter = Self::create_reporter(collector.clone(), config);
        (collector, reporter)
    }
}