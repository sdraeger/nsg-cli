# NSG CLI

A command-line interface for the [Neuroscience Gateway (NSG)](https://www.nsgportal.org/) BRAIN Initiative API.

## Features

- **Secure credential storage** - Store NSG credentials in `~/.nsg/credentials.json`
- **Job management** - Submit, monitor, and download results from NSG HPC jobs
- **Beautiful CLI** - Colored output with progress indicators
- **XML API support** - Full support for NSG's REST API (XML-based)
- **Multiple commands** - Login, list, status, submit, and download operations

## Installation

### From `crates.io`

```bash
cargo install nsg-cli
```

### Build from source

```bash
cd nsg-cli
cargo build --release
```

The binary will be at `target/release/nsg`.

### Install globally

```bash
cargo install --path .
```

This installs the `nsg` binary to `~/.cargo/bin/`.

## Quick Start

### 1. Login

First, authenticate with your NSG credentials:

```bash
nsg login
```

You'll be prompted for:

- NSG Username
- NSG Password (hidden input)
- NSG Application Key

Your credentials are stored in `~/.nsg/credentials.json` with secure permissions (0600 on Unix).

**Get NSG credentials at:** https://www.nsgportal.org/

### 2. List Jobs

View all your NSG jobs:

```bash
nsg list
```

For detailed status of each job (slower):

```bash
nsg list --detailed
```

### 3. Check Job Status

Monitor a specific job:

```bash
nsg status <JOB_ID>
```

Examples:

```bash
nsg status NGBW-JOB-PY_EXPANSE-xxxxx
nsg status https://nsgr.sdsc.edu:8443/cipresrest/v1/job/username/NGBW-JOB-PY_EXPANSE-xxxxx
```

### 4. Submit a Job

Submit a new job to NSG:

```bash
nsg submit <ZIP_FILE> --tool <TOOL_NAME>
```

Example:

```bash
nsg submit my_analysis.zip --tool PY_EXPANSE
```

Available tools:

- `PY_EXPANSE` - Python on EXPANSE (default)
- `GPU_PY_EXPANSE` - GPU-accelerated Python
- Other NSG tools as supported

### 5. Download Results

Download results from a completed job:

```bash
nsg download <JOB_ID>
```

Specify output directory:

```bash
nsg download <JOB_ID> --output ./my_results
```

## Commands

### `nsg login`

Authenticate and save credentials.

**Options:**

- `-u, --username <USERNAME>` - NSG username (or prompt)
- `-p, --password <PASSWORD>` - NSG password (or prompt securely)
- `-a, --app-key <APP_KEY>` - NSG application key (or prompt)
- `--no-verify` - Skip connection test

**Example:**

```bash
nsg login --username myuser --app-key MY_APP_KEY
```

### `nsg list`

List all jobs for the authenticated user.

**Options:**

- `--detailed` - Fetch detailed status for each job (slower)

**Example:**

```bash
nsg list --detailed
```

### `nsg status <JOB>`

Check status of a specific job.

**Arguments:**

- `<JOB>` - Job URL or Job ID

**Example:**

```bash
nsg status NGBW-JOB-PY_EXPANSE-xxxxx
```

### `nsg submit <ZIP_FILE>`

Submit a new job to NSG.

**Arguments:**

- `<ZIP_FILE>` - Path to ZIP file containing job data

**Options:**

- `-t, --tool <TOOL>` - NSG tool to use (default: PY_EXPANSE)
- `--no-wait` - Don't wait for job submission confirmation

**Example:**

```bash
nsg submit job_data.zip --tool PY_EXPANSE
```

### `nsg download <JOB>`

Download results from a completed job.

**Arguments:**

- `<JOB>` - Job URL or Job ID

**Options:**

- `-o, --output <DIR>` - Output directory (default: ./nsg_results)

**Example:**

```bash
nsg download NGBW-JOB-PY_EXPANSE-xxxxx --output ./results
```

## NSG Job Package Structure

When submitting jobs, NSG expects a specific ZIP structure. For Python jobs:

```
job.zip
└── modeldir/
    ├── input.py          # Main Python script (required)
    ├── data.edf          # Input data files
    ├── params.json       # Configuration
    └── ...               # Other files
```

The main script should be named `input.py` for PY_EXPANSE tool.

## Configuration

Credentials are stored in: `~/.nsg/credentials.json`

**Format:**

```json
{
  "username": "your_username",
  "password": "your_password",
  "app_key": "your_app_key"
}
```

**Security:**

- On Unix systems, the file permissions are set to `0600` (read/write for owner only)
- Never commit this file to version control
- Keep your credentials secure

## API Documentation

This CLI interfaces with the NSG REST API:

- **Base URL:** `https://nsgr.sdsc.edu:8443/cipresrest/v1`
- **Authentication:** HTTP Basic Auth + `cipres-appkey` header
- **Response format:** XML

## Development

### Project Structure

```
nsg-cli/
├── Cargo.toml
├── src/
│   ├── main.rs           # CLI entry point
│   ├── lib.rs            # Library exports
│   ├── client.rs         # NSG API client
│   ├── config.rs         # Credential management
│   ├── models.rs         # Data structures & XML parsing
│   └── commands/         # CLI commands
│       ├── mod.rs
│       ├── login.rs
│       ├── list.rs
│       ├── status.rs
│       ├── submit.rs
│       └── download.rs
└── README.md
```

### Dependencies

- **clap** - CLI argument parsing
- **reqwest** - HTTP client
- **quick-xml** - XML parsing
- **serde** - Serialization
- **colored** - Terminal colors
- **indicatif** - Progress bars
- **rpassword** - Secure password input

### Building

```bash
cargo build          # Debug build
cargo build --release  # Release build (optimized)
cargo test           # Run tests
cargo check          # Type checking only (fast)
```

## Troubleshooting

### Authentication Failed

If login fails:

1. Verify credentials at https://www.nsgportal.org/
2. Check that your NSG account is active
3. Ensure application key is correct
4. Try using `--no-verify` to skip connection test and save credentials anyway

### Job Not Found

If `status` or `download` can't find a job:

1. Verify job ID is correct
2. Use `nsg list` to see all your jobs
3. Try using the full job URL instead of just the ID

### Download Failed

If results download fails:

1. Check job is in COMPLETED stage with `nsg status`
2. Verify job has results available
3. Check output directory permissions

## License

MIT License

## Related Projects

- [DDALAB](https://github.com/sdraeger/DDALAB) - Delay Differential Analysis Laboratory
- [NSG Portal](https://www.nsgportal.org/) - Neuroscience Gateway web interface

## Contributing

This tool was created as part of the DDALAB project. For issues or contributions, please use the DDALAB repository.

## Contact

For NSG-related issues: nsghelp@sdsc.edu
For DDALAB issues: https://github.com/sdraeger/DDALAB/issues
