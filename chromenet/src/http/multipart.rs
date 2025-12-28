//! Multipart form data support.
//!
//! Provides RFC 2046 multipart/form-data encoding for file uploads.
//! Inspired by wreq's multipart implementation.
//!
//! # Example
//! ```ignore
//! use chromenet::http::multipart::{Form, Part};
//!
//! let form = Form::new()
//!     .text("username", "user123")
//!     .part("file", Part::bytes(b"file content").file_name("doc.txt"));
//!
//! // Use form.into_body() to get the request body
//! ```

use bytes::Bytes;
use std::borrow::Cow;

/// A multipart form for file uploads.
#[derive(Debug)]
pub struct Form {
    boundary: String,
    fields: Vec<(Cow<'static, str>, Part)>,
}

impl Default for Form {
    fn default() -> Self {
        Self::new()
    }
}

impl Form {
    /// Create a new empty form.
    pub fn new() -> Self {
        Self {
            boundary: generate_boundary(),
            fields: Vec::new(),
        }
    }

    /// Get the boundary string.
    pub fn boundary(&self) -> &str {
        &self.boundary
    }

    /// Add a text field.
    pub fn text<N, V>(self, name: N, value: V) -> Self
    where
        N: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        self.part(name, Part::text(value))
    }

    /// Add a custom part.
    pub fn part<N>(mut self, name: N, part: Part) -> Self
    where
        N: Into<Cow<'static, str>>,
    {
        self.fields.push((name.into(), part));
        self
    }

    /// Get the Content-Type header value.
    pub fn content_type(&self) -> String {
        format!("multipart/form-data; boundary={}", self.boundary)
    }

    /// Compute the total content length if possible.
    ///
    /// Returns None if any part has unknown length.
    pub fn content_length(&self) -> Option<usize> {
        if self.fields.is_empty() {
            return Some(0);
        }

        let mut length = 0usize;

        for (name, part) in &self.fields {
            // --boundary\r\n
            length += 2 + self.boundary.len() + 2;

            // Content-Disposition header
            let header = part.format_headers(name);
            length += header.len();

            // \r\n\r\n
            length += 4;

            // Body
            length += part.data.len();

            // \r\n
            length += 2;
        }

        // Final boundary: --boundary--\r\n
        length += 2 + self.boundary.len() + 4;

        Some(length)
    }

    /// Convert the form into a body bytes.
    pub fn into_body(self) -> Bytes {
        if self.fields.is_empty() {
            return Bytes::new();
        }

        let mut output = Vec::new();

        for (name, part) in self.fields {
            // --boundary\r\n
            output.extend_from_slice(b"--");
            output.extend_from_slice(self.boundary.as_bytes());
            output.extend_from_slice(b"\r\n");

            // Headers
            output.extend_from_slice(part.format_headers(&name).as_bytes());
            output.extend_from_slice(b"\r\n\r\n");

            // Body
            output.extend_from_slice(&part.data);
            output.extend_from_slice(b"\r\n");
        }

        // Final boundary
        output.extend_from_slice(b"--");
        output.extend_from_slice(self.boundary.as_bytes());
        output.extend_from_slice(b"--\r\n");

        Bytes::from(output)
    }
}

/// A part of a multipart form.
#[derive(Debug, Clone)]
pub struct Part {
    data: Bytes,
    content_type: Option<String>,
    file_name: Option<Cow<'static, str>>,
}

impl Part {
    /// Create a text part.
    pub fn text<V>(value: V) -> Self
    where
        V: Into<Cow<'static, str>>,
    {
        let s = value.into();
        Self {
            data: Bytes::from(s.into_owned()),
            content_type: Some("text/plain; charset=utf-8".to_string()),
            file_name: None,
        }
    }

    /// Create a part from bytes.
    pub fn bytes<B>(data: B) -> Self
    where
        B: Into<Bytes>,
    {
        Self {
            data: data.into(),
            content_type: None,
            file_name: None,
        }
    }

    /// Set the content type.
    pub fn content_type<S: Into<String>>(mut self, mime: S) -> Self {
        self.content_type = Some(mime.into());
        self
    }

    /// Set the file name.
    pub fn file_name<S>(mut self, name: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.file_name = Some(name.into());
        self
    }

    /// Format the part headers.
    fn format_headers(&self, name: &str) -> String {
        let mut header = format!(
            "Content-Disposition: form-data; name=\"{}\"",
            escape_quotes(name)
        );

        if let Some(ref filename) = self.file_name {
            header.push_str(&format!("; filename=\"{}\"", escape_quotes(filename)));
        }

        if let Some(ref mime) = self.content_type {
            header.push_str(&format!("\r\nContent-Type: {}", mime));
        }

        header
    }

    /// Get the data length.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if part is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Escape quotes and backslashes in a string.
fn escape_quotes(s: &str) -> Cow<'_, str> {
    if s.contains('"') || s.contains('\\') || s.contains('\r') || s.contains('\n') {
        Cow::Owned(
            s.replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\r', "\\r")
                .replace('\n', "\\n"),
        )
    } else {
        Cow::Borrowed(s)
    }
}

/// Generate a random boundary string.
fn generate_boundary() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let nanos = now.as_nanos();

    // Use timestamp + process id for uniqueness
    format!(
        "----chromenet-boundary-{:016x}{:08x}",
        nanos,
        std::process::id()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_form() {
        let form = Form::new();
        assert!(form.into_body().is_empty());
    }

    #[test]
    fn test_text_field() {
        let form = Form::new().text("name", "value");
        let body = form.into_body();

        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("name=\"name\""));
        assert!(body_str.contains("value"));
    }

    #[test]
    fn test_file_part() {
        let part = Part::bytes(b"file data".as_slice())
            .file_name("test.txt")
            .content_type("text/plain");

        let form = Form::new().part("upload", part);
        let body = form.into_body();

        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("filename=\"test.txt\""));
        assert!(body_str.contains("Content-Type: text/plain"));
        assert!(body_str.contains("file data"));
    }

    #[test]
    fn test_boundary() {
        let form = Form::new();
        assert!(form.boundary().starts_with("----chromenet-boundary-"));
    }

    #[test]
    fn test_content_type() {
        let form = Form::new();
        let ct = form.content_type();
        assert!(ct.starts_with("multipart/form-data; boundary="));
    }

    #[test]
    fn test_content_length() {
        let form = Form::new().text("key", "value");

        let length = form.content_length().unwrap();
        let body = form.into_body();
        assert_eq!(length, body.len());
    }

    #[test]
    fn test_escape_quotes() {
        assert_eq!(escape_quotes("normal"), "normal");
        assert_eq!(escape_quotes("with\"quote"), "with\\\"quote");
        assert_eq!(escape_quotes("with\\slash"), "with\\\\slash");
    }

    #[test]
    fn test_multiple_parts() {
        let form = Form::new()
            .text("field1", "value1")
            .text("field2", "value2")
            .part(
                "file",
                Part::bytes(b"binary".as_slice()).file_name("data.bin"),
            );

        let body = form.into_body();
        let body_str = String::from_utf8_lossy(&body);

        assert!(body_str.contains("field1"));
        assert!(body_str.contains("value1"));
        assert!(body_str.contains("field2"));
        assert!(body_str.contains("value2"));
        assert!(body_str.contains("data.bin"));
        assert!(body_str.ends_with("--\r\n"));
    }
}
