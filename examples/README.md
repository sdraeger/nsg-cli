# NSG CLI Examples

This directory contains example job packages for submitting to NSG using the CLI.

## Test Job

The `test-job` directory contains a minimal example that demonstrates the required package structure.

### Structure

```
test-job/
└── modeldir/           # Required: NSG expects this subdirectory
    ├── input.py        # Required: Main entry point for PY_EXPANSE
    └── params.json     # Optional: Job parameters
```

### Creating the ZIP Package

```bash
cd test-job
zip -r ../test-job.zip modeldir/
```

### Submitting the Job

```bash
# From the examples directory
nsg submit test-job.zip --tool PY_EXPANSE
```

### Expected Output

When the job completes, you should see:
- `test_output.json` - Results from the test script
- `stdout.txt` - Standard output
- `stderr.txt` - Standard error (if any)

### Monitoring

```bash
# After submission, note the JOB_ID, then:
nsg status <JOB_ID>

# When completed:
nsg download <JOB_ID> --output ./test-results
```

## DDA Job Example

For DDA (Delay Differential Analysis) jobs, the package structure should include:

```
dda-job/
└── modeldir/
    ├── run_dda_nsg.py     # DDA wrapper script
    ├── recording.edf      # Input EDF file
    └── params.json        # DDA parameters
```

See the `nsg_wrapper` directory in the main DDALAB repository for the actual DDA wrapper implementation.

## Tips

1. **Package Structure**: Always use the `modeldir/` subdirectory - NSG requires it
2. **Entry Point**: For PY_EXPANSE, name your main script `input.py`
3. **File Paths**: Use relative paths in your scripts (files are in the same directory)
4. **Output Files**: Any files you create will be available for download when the job completes
5. **Error Handling**: Check `stderr.txt` if your job fails
6. **Timeouts**: NSG jobs have walltime limits (typically 1-48 hours depending on the cluster)

## Testing Locally

Before submitting to NSG, test your script locally:

```bash
cd test-job/modeldir
python3 input.py
```

This helps catch errors before using HPC resources.
