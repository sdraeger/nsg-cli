use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct JobSummary {
    pub job_id: String,
    pub url: String,
    pub tool: Option<String>,
    pub job_stage: Option<String>,
    pub failed: bool,
    pub date_submitted: Option<String>,
    pub date_completed: Option<String>,
}

#[derive(Debug, Clone)]
pub struct JobStatus {
    pub job_id: String,
    pub job_stage: String,
    pub failed: bool,
    pub date_submitted: Option<String>,
    pub date_completed: Option<String>,
    pub tool_id: Option<String>,
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

    // Current job being parsed
    let mut current_url = None;
    let mut current_title = None;
    let mut current_tool = None;
    let mut current_stage = None;
    let mut current_failed = false;
    let mut current_date_submitted = None;
    let mut current_date_completed = None;

    let mut in_self_uri = false;
    let mut in_job = false;
    let mut current_tag = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                current_tag = tag.clone();

                match tag.as_str() {
                    "jobstatus" => {
                        in_job = true;
                        // Reset all fields for new job
                        current_url = None;
                        current_title = None;
                        current_tool = None;
                        current_stage = None;
                        current_failed = false;
                        current_date_submitted = None;
                        current_date_completed = None;
                    }
                    "selfUri" => in_self_uri = true,
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();

                match tag.as_str() {
                    "selfUri" => in_self_uri = false,
                    "jobstatus" => {
                        in_job = false;
                        if let (Some(url), Some(title)) = (current_url.take(), current_title.take())
                        {
                            jobs.push(JobSummary {
                                job_id: title,
                                url,
                                tool: current_tool.take(),
                                job_stage: current_stage.take(),
                                failed: current_failed,
                                date_submitted: current_date_submitted.take(),
                                date_completed: current_date_completed.take(),
                            });
                        }
                        current_failed = false;
                    }
                    _ => {}
                }
                current_tag.clear();
            }
            Ok(Event::Text(e)) => {
                let text = reader
                    .decoder()
                    .decode(e.as_ref())
                    .ok()
                    .map(|s| s.to_string());

                if let Some(text) = text {
                    match current_tag.as_str() {
                        "url" if in_self_uri => current_url = Some(text),
                        "title" if in_self_uri => current_title = Some(text),
                        "toolId" if in_job => current_tool = Some(text),
                        "jobStage" if in_job => current_stage = Some(text),
                        "failed" if in_job => current_failed = text == "true",
                        "dateSubmitted" if in_job => current_date_submitted = Some(text),
                        "dateTerminated" if in_job => current_date_completed = Some(text),
                        _ => {}
                    }
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
    let mut terminal_stage = false;
    let mut date_submitted: Option<String> = None;
    let mut date_completed: Option<String> = None;
    let mut self_uri = String::new();
    let mut results_uri: Option<String> = None;
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
                    "terminalStage" => terminal_stage = text == "true",
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

    // Extract tool from job ID (e.g., "NGBW-JOB-PY_EXPANSE-..." -> "PY_EXPANSE")
    let extracted_tool = extract_tool_from_job_id(&job_id);

    // Use last message timestamp as completion date if job is in terminal stage
    if terminal_stage && date_completed.is_none() && !messages.is_empty() {
        date_completed = messages.last().and_then(|m| m.timestamp.clone());
    }

    Ok(JobStatus {
        job_id,
        job_stage,
        failed,
        date_submitted,
        date_completed,
        tool_id: extracted_tool,
        self_uri,
        results_uri,
        messages,
    })
}

fn extract_tool_from_job_id(job_id: &str) -> Option<String> {
    // Job IDs follow pattern: NGBW-JOB-{TOOL}-{HASH}
    // where HASH is a 32-character hex string
    // TOOL may contain dashes (e.g., TOOL-NAME)
    let parts: Vec<&str> = job_id.split('-').collect();
    if parts.len() >= 4 && parts[0] == "NGBW" && parts[1] == "JOB" {
        let last_part = parts[parts.len() - 1];
        // Check if last part is a 32-char hex hash
        if last_part.len() == 32 && last_part.chars().all(|c| c.is_ascii_hexdigit()) {
            // Tool is everything between "JOB" and the hash
            let tool_parts = &parts[2..parts.len() - 1];
            Some(tool_parts.join("-"))
        } else {
            // Fallback: assume tool is just the third part
            Some(parts[2].to_string())
        }
    } else {
        None
    }
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
