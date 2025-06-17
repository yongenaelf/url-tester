use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::time::Instant;

// ANSI escape codes for text colors
const COLOR_GREEN: &str = "\x1b[32m";
const COLOR_RED: &str = "\x1b[31m";
const COLOR_RESET: &str = "\x1b[0m"; // Resets text color to default

/// A simple CLI tool to test URLs from a configuration file.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the configuration file (e.g., config.toml)
    #[arg(short, long)]
    config: String,
    /// Optional path to an output CSV file (e.g., report.csv)
    #[arg(short, long)]
    output: Option<String>,
    /// Optional: Run tests only for a specific environment name defined in the config (e.g., "dev", "staging")
    #[arg(long)]
    env: Option<String>,
}

/// Represents a single environment with its base URL.
#[derive(Debug, Deserialize)]
struct Environment {
    baseurl: String,
}

/// Represents the structure of our configuration file.
#[derive(Debug, Deserialize)]
struct Config {
    environments: HashMap<String, Environment>,
    paths: Vec<String>,
    // Optional application error key to search for (e.g., "code", "errorCode")
    // Defaults to "code" if not specified in the TOML.
    #[serde(default = "default_app_error_key")]
    app_error_key_to_fail: String,
    // Optional application error code to fail on, e.g., "50000"
    // Using #[serde(default)] allows this field to be omitted in the TOML,
    // in which case it will default to None.
    #[serde(default)]
    app_error_code_to_fail: Option<String>,
}

// Helper function to provide a default value for app_error_key_to_fail
fn default_app_error_key() -> String {
    "code".to_string()
}

/// Struct to parse the relevant part of the API response, focusing only on the message.
#[derive(Debug, Deserialize)]
struct ApiResponse {
    message: String,
}

/// Represents the result of a single URL test.
#[derive(Debug, Serialize)]
struct UrlTestResult {
    environment_name: String,
    url: String,
    // Fix for UnequalLengths: Removed #[serde(skip_serializing_if = "Option::is_none")]
    // This ensures all records have the same number of columns in CSV,
    // with None values appearing as empty fields.
    status_code: Option<u16>,
    response_body_preview: String,
    passed: bool,
    // Fix for UnequalLengths: Removed #[serde(skip_serializing_if = "Option::is_none")]
    error_message: Option<String>,
    duration_secs: f64,
    // Fix for UnequalLengths: Removed #[serde(skip_serializing_if = "Option::is_none")]
    state_param: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("Loading configuration from: {}", args.config);
    let config_content = fs::read_to_string(&args.config)?;
    let config: Config = toml::from_str(&config_content)?;

    if config.environments.is_empty() {
        println!("No environments found in the configuration file. Exiting.");
        return Ok(());
    }

    if config.paths.is_empty() {
        println!("No paths found in the configuration file. Exiting.");
        return Ok(());
    }

    let mut all_results: Vec<UrlTestResult> = Vec::new();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let total_test_start_time = Instant::now();

    let environments_to_run: HashMap<String, Environment> = if let Some(env_name) = &args.env {
        let mut filtered_envs = HashMap::new();
        if let Some(env_data) = config.environments.get(env_name) {
            filtered_envs.insert(
                env_name.clone(),
                Environment {
                    baseurl: env_data.baseurl.clone(),
                },
            );
            println!("\nRunning tests for specific environment: {}", env_name);
        } else {
            eprintln!(
                "Error: Environment '{}' not found in config.toml.",
                env_name
            );
            return Err(format!("Environment '{}' not found.", env_name).into());
        }
        filtered_envs
    } else {
        println!("\nRunning tests for ALL environments found in config.");
        config.environments
    };

    // Clone both the configured key and code for use in the async tasks
    let configured_app_error_key = config.app_error_key_to_fail.clone();
    let configured_app_error_code = config.app_error_code_to_fail.clone();

    for (env_name, env_data) in environments_to_run {
        println!(
            "\n--- Testing Environment: {} (Base URL: {}) ---",
            env_name, env_data.baseurl
        );

        let mut handles = Vec::new();
        let total_paths_for_env = config.paths.len();

        println!("\nInitiating requests for environment '{}'...", env_name);

        for path in &config.paths {
            let client = client.clone();
            let env_name_clone = env_name.clone();
            let path_clone = path.clone();
            // Clone configured key and code for each spawned task
            let app_error_key_for_task = configured_app_error_key.clone();
            let app_error_code_for_task = configured_app_error_code.clone();

            let state_param = path_clone
                .split_once("State=")
                .and_then(|(_, rest)| rest.split_once('&'))
                .map(|(state, _)| state.to_string())
                .or_else(|| {
                    path_clone
                        .split_once("State=")
                        .map(|(_, state)| state.to_string())
                });

            let full_url = format!("{}{}", env_data.baseurl, path_clone);
            let url_clone = full_url.clone();

            let handle = tokio::spawn(async move {
                let start_time = Instant::now();
                let mut result = UrlTestResult {
                    environment_name: env_name_clone,
                    url: url_clone.clone(),
                    status_code: None,
                    response_body_preview: String::new(),
                    passed: false,
                    error_message: None,
                    duration_secs: 0.0,
                    state_param: state_param,
                };

                match client.get(&url_clone).send().await {
                    Ok(response) => {
                        result.status_code = Some(response.status().as_u16());
                        let status = response.status();

                        let body_text = match response.text().await {
                            Ok(text) => text,
                            Err(e) => {
                                result.response_body_preview = format!("Error reading body: {}", e);
                                result.passed = false;
                                result.error_message =
                                    Some(format!("Failed to read response body: {}", e));
                                "".to_string()
                            }
                        };

                        result.response_body_preview = body_text.chars().take(100).collect();

                        if status.is_success() {
                            let mut app_error_detected = false;
                            // Check if a specific application error code is configured
                            if let Some(code_to_fail) = app_error_code_for_task {
                                let key_to_search = app_error_key_for_task.as_str(); // Use the configured key
                                                                                     // Dynamically construct the search string using both key and code
                                let search_string =
                                    format!(r#""{}":"{}""#, key_to_search, code_to_fail);
                                if body_text.contains(&search_string) {
                                    app_error_detected = true;
                                    // If parsing ApiResponse fails, use the configured key and code in the message
                                    match serde_json::from_str::<ApiResponse>(&body_text) {
                                        Ok(api_response) => {
                                            result.error_message = Some(format!(
                                                "App Error ({}: {}): {}",
                                                key_to_search, code_to_fail, api_response.message
                                            ));
                                        }
                                        Err(_) => {
                                            result.error_message = Some(format!(
                                                "App Error ({}: {}): message parsing failed.",
                                                key_to_search, code_to_fail
                                            ));
                                        }
                                    }
                                }
                            }

                            if app_error_detected {
                                result.passed = false; // Mark as failed due to application error
                            } else {
                                result.passed = true; // Passed if HTTP 2xx and no configured app error
                            }
                        } else {
                            result.passed = false; // Failed if HTTP status is not 2xx
                            result.error_message = Some(format!("HTTP Status Error: {}", status));
                        }
                    }
                    Err(e) => {
                        result.error_message = Some(e.to_string());
                        result.passed = false;
                    }
                }
                result.duration_secs = start_time.elapsed().as_secs_f64();
                result
            });
            handles.push(handle);
        }

        println!(
            "Waiting for {} responses from '{}'...",
            total_paths_for_env, env_name
        );
        for handle in handles {
            let result = handle.await?;
            all_results.push(result);
        }
    }

    let total_test_end_time = Instant::now();
    let total_duration = total_test_end_time.duration_since(total_test_start_time);

    // --- Separate and print tables for passing and then failing tests ---
    let mut failing_results: Vec<UrlTestResult> = Vec::new();
    let mut passing_results: Vec<UrlTestResult> = Vec::new();

    for res in all_results {
        // Consume all_results here by moving elements
        if res.passed {
            passing_results.push(res);
        } else {
            failing_results.push(res);
        }
    }

    // Sort passing results
    passing_results.sort_by(|a, b| {
        a.environment_name
            .cmp(&b.environment_name)
            .then_with(|| a.state_param.cmp(&b.state_param))
    });
    // Sort failing results
    failing_results.sort_by(|a, b| {
        a.environment_name
            .cmp(&b.environment_name)
            .then_with(|| a.state_param.cmp(&b.state_param))
    });

    println!("\nTotal Test Duration: {:.2?}", total_duration);

    // Print Passing Tests Table FIRST
    if !passing_results.is_empty() {
        println!("\n--- Passing Tests Report ({}) ---", passing_results.len());
        print_report_header();
        for res in &passing_results {
            print_test_result_row(res);
        }
        println!("\n--- Passing Tests Report End ---");
    } else {
        println!("\n--- No Passing Tests Detected ---");
    }

    // Print Failing Tests Table SECOND
    if !failing_results.is_empty() {
        println!("\n--- Failing Tests Report ({}) ---", failing_results.len());
        print_report_header();
        for res in &failing_results {
            print_test_result_row(res);
        }
        println!("\n--- Failing Tests Report End ---");
    } else {
        // This case will not be hit if there are passing tests but no failing ones,
        // as the "No Passing Tests Detected" message implies total absence.
        // It serves for the scenario where *all* tests failed or none ran.
    }
    // --- END REPORTING SECTION ---

    if let Some(output_path) = args.output {
        println!("\nSaving report to CSV: {}", output_path);
        let file = fs::File::create(&output_path)?;
        let mut wtr = csv::Writer::from_writer(file);

        // Reconstruct all_results for CSV output (preserving order for CSV might be less critical,
        // but ensuring all are written is).
        let mut all_results_for_csv: Vec<UrlTestResult> = Vec::new();
        all_results_for_csv.extend(passing_results); // Add passing first
        all_results_for_csv.extend(failing_results); // Then add failing

        for res in all_results_for_csv {
            wtr.serialize(res)?;
        }
        wtr.flush()?;
        println!("CSV report saved successfully.");
    }

    Ok(())
}

// Helper function to print a single test result row
fn print_test_result_row(res: &UrlTestResult) {
    let env_display = truncate_string(&res.environment_name, 8);
    let status_str = res.status_code.map_or("N/A".to_string(), |s| s.to_string());

    let passed_str_raw = if res.passed { "PASS" } else { "FAIL" };
    let colored_passed_str = if res.passed {
        format!("{}{}{}", COLOR_GREEN, passed_str_raw, COLOR_RESET)
    } else {
        format!("{}{}{}", COLOR_RED, passed_str_raw, COLOR_RESET)
    };
    let passed_padding_needed = 7;
    let current_visible_length = passed_str_raw.len();
    let formatted_passed_str = format!(
        "{: <width$}",
        colored_passed_str,
        width = passed_padding_needed + (colored_passed_str.len() - current_visible_length)
    );

    let duration_str = format!("{:.2}s", res.duration_secs);

    let error_display_message = res.error_message.as_deref().unwrap_or("None").to_string();

    let state_display = res.state_param.as_deref().unwrap_or("N/A");

    println!(
        "{: <10} | {: <20} | {: <10} | {} | {: <10} | {: <60}",
        env_display,
        truncate_string(state_display, 18),
        status_str,
        formatted_passed_str,
        duration_str,
        truncate_string(&error_display_message, 58)
    );
}

// Helper function to print the table header
fn print_report_header() {
    println!(
        "{: <10} | {: <20} | {: <10} | {: <7} | {: <10} | {: <60}",
        "Env", "State", "Status", "Passed", "Duration", "Error Message"
    );
    println!("{}", "-".repeat(128));
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() > max_len && max_len > 3 {
        format!("{}...", &s[..max_len - 3])
    } else if max_len > 0 && s.len() > max_len {
        s[..max_len].to_string()
    } else {
        s.to_string()
    }
}
