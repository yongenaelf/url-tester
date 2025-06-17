# url_tester

A simple, fast, and configurable CLI tool designed to test API endpoints across various environments. It validates HTTP status codes and can perform application-level error detection based on configurable JSON responses. Get clear console reports, with failing tests grouped for quick identification, and export results to CSV for detailed analysis.

[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

---

## 🚀 Features

* **Configurable Environments:** Define multiple base URLs (e.g., `dev`, `staging`, `prod`) in your `config.toml`.
* **Configurable Paths:** Specify API endpoints to test against each defined base URL.
* **HTTP Status Code Checks:** Automatically identifies non-2xx HTTP responses as failures.
* **Application-Level Error Detection:** Configurable to fail tests based on a specific JSON key-value pair in successful (2xx) API responses (e.g., detecting `{"code": "50000"}`).
* **Detailed Console Report:** Presents test results in a clear, colored table format, with all passing tests displayed first, followed by a separate, dedicated table for all failing tests.
* **CSV Export:** Exports all test results (both passing and failing) to a CSV file for further analysis and record-keeping.
* **Environment Filtering:** Run tests only for a specific environment defined in your configuration.

---

## 📦 Installation

### From Source (Requires Rust)

If you have Rust and Cargo installed, you can build and install `url_tester` directly from its source code:

```bash
git clone [https://github.com/yongenaelf/url_tester.git](https://github.com/yongenaelf/url_tester.git)
cd url_tester
cargo install --path .
```

This command compiles the project and places the `url_tester` executable in your Cargo bin directory (usually `~/.cargo/bin`), which should be in your system's `PATH`.

-----

## ⚙️ Configuration (`config.toml`)

`url_tester` requires a `config.toml` file to define the endpoints to test and any specific application error conditions.

Here's an example `config.toml` demonstrating its structure and available options:

```toml
# config.toml

# These are the API paths that will be tested against each defined baseurl.
paths = [
    "/some/path/to/test",
    "/another"
]

# Configure the JSON key for an application-level error.
# This field is optional. If omitted, it defaults to "code".
# Example: If your API returns `{"errorCode": "AUTH_FAILED"}`, you'd set this to "errorCode".
app_error_key_to_fail = "code"

# Configure the specific value of the application error key to fail the test.
# This field is optional. If omitted, no application-level error check will be performed.
# Example: If your API returns `{"code": "50000"}` for an internal error,
# setting this to "50000" will mark the test as failed, even if the HTTP status is 200 OK.
app_error_code_to_fail = "50000"

# Define your environments here.
# Each key (e.g., "dev", "testnet", "staging") is an environment name.
# The 'baseurl' is the root URL for that environment.
[environments.dev]
baseurl = "https://example.com/api"

[environments.testnet]
baseurl = "https://testnet.example.com/api"

[environments.staging]
baseurl = "https://staging.example.com/api"
```

-----

## 🚀 Usage

Execute `url_tester` from your terminal, providing the path to your configuration file:

```bash
url_tester --config path/to/your/config.toml
```

### Command-line Options

  * `-c, --config <FILE>`: **(Required)** Specifies the path to your `config.toml` file.
  * `-o, --output <FILE>`: **(Optional)** Provides a path to an output CSV file where all test results will be saved.
  * `--env <ENVIRONMENT>`: **(Optional)** Filters tests to run only for a specific environment name defined in your `config.toml` (e.g., `--env dev`).

### Examples

**Run all tests and print results to the console:**

```bash
url_tester --config my_api_tests.toml
```

**Run tests specifically for the "staging" environment and save all results to a CSV file:**

```bash
url_tester --config my_api_tests.toml --env staging --output staging_report.csv
```

-----

## 📊 Report Output

### Console Output

The console output provides a concise summary of the total test duration, followed by two separate, easy-to-read tables:

  * **Passing Tests Report:** Lists all URLs that successfully passed both HTTP status code and any configured application-level error checks.
  * **Failing Tests Report:** Clearly highlights all URLs that failed, providing details on the HTTP status error or the detected application error message. Failing entries are prominently colored red for immediate attention.

### CSV Output

When you use the `--output` option, a CSV file will be generated. This file includes comprehensive details for every test, such as the environment name, the full URL, the HTTP status code, a preview of the response body, the pass/fail status, any associated error messages, the test duration, and the extracted `State` parameter from the URL.

-----

## 📄 License

This project is licensed under the [MIT License](https://www.google.com/search?q=LICENSE).

-----
