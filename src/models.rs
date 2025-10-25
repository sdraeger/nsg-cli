use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct JobSummary {
    pub job_id: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct JobStatus {
    pub job_id: String,
    pub job_stage: String,
    pub failed: bool,
    pub date_submitted: Option<String>,
    pub self_uri: String,
    pub results_uri: Option<String>,
    pub messages: Vec<JobMessage>,
}

#[derive(Debug, Clone)]
pub struct JobMessage {
    pub stage: String,
    pub text: String,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OutputFile {
    pub filename: String,
    pub download_uri: String,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct DownloadedFile {
    pub filename: String,
    pub path: PathBuf,
    pub size: u64,
}

pub fn parse_job_list(xml: &str) -> Result<Vec<JobSummary>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut jobs = Vec::new();
    let mut buf = Vec::new();
    let mut current_url = None;
    let mut current_title = None;
    let mut in_self_uri = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.name().as_ref() == b"selfUri" => {
                in_self_uri = true;
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"selfUri" => {
                in_self_uri = false;
                if let (Some(url), Some(title)) = (current_url.take(), current_title.take()) {
                    jobs.push(JobSummary { job_id: title, url });
                }
            }
            Ok(Event::Start(e)) if in_self_uri && e.name().as_ref() == b"url" => {
                if let Ok(Event::Text(t)) = reader.read_event_into(&mut buf) {
                    current_url = reader
                        .decoder()
                        .decode(t.as_ref())
                        .ok()
                        .map(|s| s.to_string());
                }
            }
            Ok(Event::Start(e)) if in_self_uri && e.name().as_ref() == b"title" => {
                if let Ok(Event::Text(t)) = reader.read_event_into(&mut buf) {
                    current_title = reader
                        .decoder()
                        .decode(t.as_ref())
                        .ok()
                        .map(|s| s.to_string());
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "XML parse error at position {}: {}",
                    reader.buffer_position(),
                    e
                ))
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(jobs)
}

pub fn parse_job_status(xml: &str) -> Result<JobStatus> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut job_id = String::new();
    let mut job_stage = String::new();
    let mut failed = false;
    let mut date_submitted = None;
    let mut self_uri = String::new();
    let mut results_uri = None;
    let mut messages = Vec::new();

    let mut current_tag = String::new();
    let mut in_results_uri = false;
    let mut in_message = false;
    let mut current_message_stage = String::new();
    let mut current_message_text = String::new();
    let mut current_message_timestamp = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                current_tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match current_tag.as_str() {
                    "resultsUri" => in_results_uri = true,
                    "message" => {
                        in_message = true;
                        current_message_stage.clear();
                        current_message_text.clear();
                        current_message_timestamp = None;
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "resultsUri" => in_results_uri = false,
                    "message" => {
                        if in_message {
                            messages.push(JobMessage {
                                stage: current_message_stage.clone(),
                                text: current_message_text.clone(),
                                timestamp: current_message_timestamp.clone(),
                            });
                            in_message = false;
                        }
                    }
                    _ => {}
                }
                current_tag.clear();
            }
            Ok(Event::Text(e)) => {
                let text = reader
                    .decoder()
                    .decode(e.as_ref())
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                match current_tag.as_str() {
                    "jobHandle" => job_id = text,
                    "jobStage" => job_stage = text,
                    "failed" => failed = text == "true",
                    "dateSubmitted" => date_submitted = Some(text),
                    "url" if in_results_uri => results_uri = Some(text),
                    "url" if !in_results_uri && self_uri.is_empty() => self_uri = text,
                    "stage" if in_message => current_message_stage = text,
                    "text" if in_message => current_message_text = text,
                    "timestamp" if in_message => current_message_timestamp = Some(text),
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow::anyhow!("XML parse error: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    if job_id.is_empty() {
        anyhow::bail!("Failed to parse job status: missing job ID");
    }

    Ok(JobStatus {
        job_id,
        job_stage,
        failed,
        date_submitted,
        self_uri,
        results_uri,
        messages,
    })
}

pub fn parse_output_files(xml: &str) -> Result<Vec<OutputFile>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut files = Vec::new();
    let mut buf = Vec::new();

    let mut in_jobfile = false;
    let mut in_download_uri = false;
    let mut current_filename = None;
    let mut current_download_uri = None;
    let mut current_size = None;
    let mut current_tag = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "jobfile" => in_jobfile = true,
                    "downloadUri" => in_download_uri = true,
                    _ => current_tag = tag,
                }
            }
            Ok(Event::End(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "jobfile" => {
                        if let (Some(filename), Some(download_uri), Some(size)) = (
                            current_filename.take(),
                            current_download_uri.take(),
                            current_size.take(),
                        ) {
                            files.push(OutputFile {
                                filename,
                                download_uri,
                                size,
                            });
                        }
                        in_jobfile = false;
                    }
                    "downloadUri" => in_download_uri = false,
                    _ => {}
                }
                current_tag.clear();
            }
            Ok(Event::Text(e)) => {
                let text = reader
                    .decoder()
                    .decode(e.as_ref())
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                if in_jobfile {
                    match current_tag.as_str() {
                        "filename" => current_filename = Some(text),
                        "length" => current_size = text.parse().ok(),
                        "url" if in_download_uri => current_download_uri = Some(text),
                        _ => {}
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow::anyhow!("XML parse error: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(files)
}
