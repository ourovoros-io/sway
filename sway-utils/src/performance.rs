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
    ($description:expr, $key:expr, $expression:expr, $build_config:expr, $data:expr) => {{
        use std::io::{BufRead, Read, Write};
        #[cfg(feature = "profile")]
        if let Some(cfg) = $build_config {
            if cfg.profile_phases {
                let profiler_path = "TODO: Replace the with the proper path here";

                let mut command = std::process::Command::new(profiler_path)
                    .arg(std::process::id().to_string())
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .spawn().unwrap();

                let mut stdout = std::io::BufReader::new(command.stdout.take().unwrap());

                let mut command_output = String::new();
                stdout.read_line(&mut command_output).unwrap();
                command_output = command_output.trim_end().into();

                assert!(command_output.is_empty(), "Expected empty command output line, got: \"{command_output}\"; Decide how to handle this");

                let start_time = std::time::Instant::now();

                let output = { $expression };

                let elapsed = start_time.elapsed();

                println!("  Time elapsed to {}: {:?}", $description, elapsed);

                let mut stdin = command.stdin.take().unwrap();
                writeln!(stdin).unwrap();
                drop(stdin);

                assert!(command.wait().unwrap().success(), "Command failed, decide how to handle that here");

                let mut command_output = vec![];
                stdout.read_to_end(&mut command_output).unwrap();

                const HEADER_SIZE: usize = std::mem::size_of::<subprocess_profiler_types::Header>();

                let header = unsafe {
                    &*(command_output[..HEADER_SIZE].as_ptr() as *const subprocess_profiler_types::Header)
                };

                let frames_size = header.frame_count as usize * std::mem::size_of::<subprocess_profiler_types::Frame>();

                let frames = unsafe {
                    std::slice::from_raw_parts(
                        command_output[HEADER_SIZE..HEADER_SIZE + frames_size].as_ptr() as *const subprocess_profiler_types::Frame,
                        header.frame_count as _,
                    )
                };

                let cpu_usage_frames = unsafe {
                    std::slice::from_raw_parts(
                        command_output[HEADER_SIZE + frames_size..].as_ptr() as *const subprocess_profiler_types::CpuUsage,
                        header.cpu_usage_count as _,
                    )
                };

                println!("  Frames : {:?}", frames);
                println!("  CPU usage frames: {:?}", cpu_usage_frames);


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
