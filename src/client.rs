use crate::config::Credentials;
use crate::models::*;
use anyhow::{Context, Result};
use reqwest::blocking::{multipart, Client};
use std::io::{Read, Write};
use std::path::Path;

const NSG_BASE_URL: &str = "https://nsgr.sdsc.edu:8443/cipresrest/v1";

pub struct NsgClient {
    client: Client,
    credentials: Credentials,
    base_url: String,
}

impl NsgClient {
    pub fn new(credentials: Credentials) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            credentials,
            base_url: NSG_BASE_URL.to_string(),
        })
    }

    pub fn new_with_url(credentials: Credentials, base_url: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            credentials,
            base_url,
        })
    }

    fn build_request(
        &self,
        method: reqwest::Method,
        path: &str,
    ) -> reqwest::blocking::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.client
            .request(method, &url)
            .basic_auth(&self.credentials.username, Some(&self.credentials.password))
            .header("cipres-appkey", &self.credentials.app_key)
    }

    pub fn test_connection(&self) -> Result<()> {
        let path = format!("/job/{}", self.credentials.username);
        let response = self
            .build_request(reqwest::Method::GET, &path)
            .send()
            .context("Failed to connect to NSG API")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Authentication failed: HTTP {} - Check your credentials",
                response.status()
            );
        }

        Ok(())
    }

    pub fn list_jobs(&self) -> Result<Vec<JobSummary>> {
        let path = format!("/job/{}", self.credentials.username);
        let response = self
            .build_request(reqwest::Method::GET, &path)
            .send()
            .context("Failed to fetch job list")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to list jobs: HTTP {}", response.status());
        }

        let body = response.text()?;
        parse_job_list(&body)
    }

    pub fn get_job_status(&self, job_url_or_id: &str) -> Result<JobStatus> {
        let path = if job_url_or_id.starts_with("http") {
            job_url_or_id
                .strip_prefix(&self.base_url)
                .context("Invalid job URL")?
                .to_string()
        } else if job_url_or_id.starts_with("/job/") {
            job_url_or_id.to_string()
        } else {
            format!("/job/{}/{}", self.credentials.username, job_url_or_id)
        };

        let response = self
            .build_request(reqwest::Method::GET, &path)
            .send()
            .context("Failed to fetch job status")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to get job status: HTTP {}\nJob: {}",
                response.status(),
                job_url_or_id
            );
        }

        let body = response.text()?;
        parse_job_status(&body)
    }

    pub fn submit_job(&self, zip_path: &Path, tool: &str) -> Result<JobStatus> {
        let path = format!("/job/{}", self.credentials.username);

        let file_part = multipart::Part::file(zip_path)
            .context("Failed to read ZIP file")?
            .file_name(
                zip_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("job.zip")
                    .to_string(),
            );

        let form = multipart::Form::new()
            .text("tool", tool.to_string())
            .part("input.infile_", file_part)
            .text("metadata.statusEmail", "true");

        let response = self
            .build_request(reqwest::Method::POST, &path)
            .multipart(form)
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .context("Failed to submit job")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            anyhow::bail!("Failed to submit job: HTTP {}\nResponse: {}", status, body);
        }

        let body = response.text()?;
        parse_job_status(&body)
    }

    pub fn download_results<F>(
        &self,
        job_url_or_id: &str,
        output_dir: &Path,
        mut progress_callback: F,
    ) -> Result<Vec<DownloadedFile>>
    where
        F: FnMut(&str, u64, u64), // (filename, bytes_downloaded, total_bytes)
    {
        let job_status = self.get_job_status(job_url_or_id)?;

        let results_url = job_status
            .results_uri
            .context("Job has no results URL - may not be completed yet")?;

        let results_path = results_url
            .strip_prefix(&self.base_url)
            .context("Invalid results URL")?;

        let response = self
            .build_request(reqwest::Method::GET, results_path)
            .send()
            .context("Failed to fetch results list")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get results: HTTP {}", response.status());
        }

        let body = response.text()?;
        let output_files = parse_output_files(&body)?;

        std::fs::create_dir_all(output_dir).context("Failed to create output directory")?;

        let mut downloaded = Vec::new();

        for file in output_files {
            let download_path = file
                .download_uri
                .strip_prefix(&self.base_url)
                .context("Invalid download URL")?;

            let output_path = output_dir.join(&file.filename);

            let mut response = self
                .build_request(reqwest::Method::GET, download_path)
                .send()
                .with_context(|| format!("Failed to download {}", file.filename))?;

            if !response.status().is_success() {
                anyhow::bail!(
                    "Failed to download {}: HTTP {}",
                    file.filename,
                    response.status()
                );
            }

            let mut dest = std::fs::File::create(&output_path)
                .with_context(|| format!("Failed to create {}", output_path.display()))?;

            // Download with progress tracking
            let total_size = file.size;
            let mut downloaded_bytes = 0u64;
            let mut buffer = [0u8; 8192];

            loop {
                let bytes_read = response
                    .read(&mut buffer)
                    .with_context(|| format!("Failed to read from {}", file.filename))?;

                if bytes_read == 0 {
                    break;
                }

                dest.write_all(&buffer[..bytes_read])
                    .with_context(|| format!("Failed to write to {}", file.filename))?;

                downloaded_bytes += bytes_read as u64;
                progress_callback(&file.filename, downloaded_bytes, total_size);
            }

            downloaded.push(DownloadedFile {
                filename: file.filename,
                path: output_path,
                size: file.size,
            });
        }

        Ok(downloaded)
    }
}
