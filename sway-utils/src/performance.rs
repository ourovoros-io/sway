use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PerformanceMetric {
    pub phase: String,
    pub elapsed: f64,
    pub memory_usage: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PerformanceData {
    pub bytecode_size: usize,
    pub metrics: Vec<PerformanceMetric>,
    pub reused_programs: u64,
}

#[macro_export]
// Time the given expression and print/save the result.
macro_rules! time_expr {
    ($pkg_name:expr, $description:expr, $key:expr, $expression:expr, $build_config:expr, $data:expr) => {{
        use std::io::{BufRead, Read, Write};
        #[cfg(feature = "profile")]
        if let Some(cfg) = $build_config {
            if cfg.profile_phases {
                println!("/forc-perf start {} {}", $pkg_name, $description);

                let output = { $expression };

                println!("/forc-perf stop {} {}", $pkg_name, $description);

                output
            } else {
                $expression
            }
        } else {
            $expression
        }

        #[cfg(not(feature = "profile"))]
        if let Some(cfg) = $build_config {
            if cfg.time_phases || cfg.metrics_outfile.is_some() {
                let expr_start = std::time::Instant::now();

                let output = { $expression };

                let elapsed = expr_start.elapsed();

                if cfg.time_phases {
                    println!("  Time elapsed to {}: {:?}", $description, elapsed);
                }

                if cfg.metrics_outfile.is_some() {
                    #[cfg(not(target_os = "macos"))]
                    let memory_usage = {
                        let mut sys = sysinfo::System::new_with_specifics(
                            sysinfo::RefreshKind::new()
                                .with_memory(sysinfo::MemoryRefreshKind::everything()),
                        );
                        sys.refresh_memory();

                        Some(sys.used_memory())
                    };

                    #[cfg(target_os = "macos")]
                    let memory_usage = None;

                    $data.metrics.push(PerformanceMetric {
                        phase: $key.to_string(),
                        elapsed: elapsed.as_secs_f64(),
                        memory_usage,
                    });
                }

                output
            } else {
                $expression
            }
        } else {
            $expression
        }
    }};
}
