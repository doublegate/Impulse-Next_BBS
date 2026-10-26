//! JAM message base format implementation

mod header;
mod kludge;
mod write;

pub use header::*;
pub use kludge::*;
pub use write::*;

use crate::error::{MessageError, Result};
use crate::sanitize::MessageSanitizer;
use crate::traits::MessageBase;
use crate::types::{
    FullMessage, MessageBaseStats, MessageHeader, MessageThread, NewMessage, SearchCriteria,
};
use crate::validation::MessageValidator;
use async_trait::async_trait;
use binrw::BinRead;
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::RwLock;

/// JAM message base implementation
pub struct JamMessageBase {
    /// Base path (without extension)
    base_path: PathBuf,
    /// Base header (cached)
    base_header: Arc<RwLock<Option<JamBaseHeader>>>,
    /// Message header cache (msg_num -> header) - reserved for future use
    #[allow(dead_code)]
    header_cache: Arc<RwLock<HashMap<u32, JamMessageHeader>>>,
    /// Subfield cache (msg_num -> subfields) - reserved for future use
    #[allow(dead_code)]
    subfield_cache: Arc<RwLock<HashMap<u32, Vec<JamSubfield>>>>,
}

impl JamMessageBase {
    /// Create a new JAM message base
    ///
    /// # Arguments
    /// * `base_path` - Path to the base without extension (e.g., "/msg/general")
    ///   The implementation will look for general.jhr, general.jdt, general.jdx
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            base_header: Arc::new(RwLock::new(None)),
            header_cache: Arc::new(RwLock::new(HashMap::new())),
            subfield_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the path to the header file
    fn jhr_path(&self) -> PathBuf {
        self.base_path.with_extension("jhr")
    }

    /// Get the path to the data file
    fn jdt_path(&self) -> PathBuf {
        self.base_path.with_extension("jdt")
    }

    /// Get the path to the index file
    #[allow(dead_code)]
    fn jdx_path(&self) -> PathBuf {
        self.base_path.with_extension("jdx")
    }

    /// Load the base header
    async fn load_base_header(&self) -> Result<JamBaseHeader> {
        // Check cache
        {
            let cache = self.base_header.read().await;
            if let Some(header) = cache.as_ref() {
                return Ok(header.clone());
            }
        }

        // Load from file
        let mut file = File::open(self.jhr_path()).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        let mut cursor = std::io::Cursor::new(buffer);
        let header = JamBaseHeader::read(&mut cursor)
            .map_err(|e| MessageError::InvalidHeader(e.to_string()))?;

        if !header.is_valid() {
            return Err(MessageError::InvalidFormat(
                "Invalid JAM signature".to_string(),
            ));
        }

        // Cache it
        {
            let mut cache = self.base_header.write().await;
            *cache = Some(header.clone());
        }

        Ok(header)
    }

    /// Load a message header by position in .jhr file
    async fn load_message_header(
        &self,
        position: u64,
    ) -> Result<(JamMessageHeader, Vec<JamSubfield>)> {
        let mut file = File::open(self.jhr_path()).await?;
        file.seek(std::io::SeekFrom::Start(position)).await?;

        let mut buffer = vec![0u8; 68];
        file.read_exact(&mut buffer).await?;

        let mut cursor = std::io::Cursor::new(buffer);
        let header = JamMessageHeader::read(&mut cursor)
            .map_err(|e| MessageError::InvalidHeader(e.to_string()))?;

        if !header.is_valid() {
            return Err(MessageError::CorruptMessage(
                "Invalid message header signature".to_string(),
            ));
        }

        // Read subfields
        let mut subfield_buffer = vec![0u8; header.subfield_len as usize];
        file.read_exact(&mut subfield_buffer).await?;

        let mut cursor = std::io::Cursor::new(subfield_buffer);
        let mut subfields = Vec::new();

        while cursor.position() < header.subfield_len as u64 {
            match JamSubfield::read(&mut cursor) {
                Ok(subfield) => subfields.push(subfield),
                Err(_) => break,
            }
        }

        Ok((header, subfields))
    }

    /// Load message text from .jdt file
    async fn load_message_text(&self, offset: u32, length: u32) -> Result<String> {
        let mut file = File::open(self.jdt_path()).await?;
        file.seek(std::io::SeekFrom::Start(offset as u64)).await?;

        let mut buffer = vec![0u8; length as usize];
        file.read_exact(&mut buffer).await?;

        String::from_utf8(buffer).map_err(|e| e.into())
    }

    /// Get subfield value by type
    fn get_subfield_value(subfields: &[JamSubfield], field_type: SubfieldType) -> Option<String> {
        subfields
            .iter()
            .find(|s| s.subfield_type() == field_type)
            .map(|s| s.as_string())
    }

    /// Build thread information for a message
    async fn build_thread(&self, msg_num: u32, header: &JamMessageHeader) -> Result<MessageThread> {
        let mut thread = MessageThread::new(msg_num);
        thread.parent_id = if header.reply_to != 0 {
            Some(header.reply_to)
        } else {
            None
        };

        // Build children list
        let current_reply = header.reply_1st;
        if current_reply != 0 {
            thread.add_child(current_reply);
            // We'd need to load that message to get the next reply
            // This is a simplified implementation
        }

        // Calculate depth by walking up parent chain
        let mut depth = 0;
        let current_parent = header.reply_to;
        let mut path = vec![msg_num];

        if current_parent != 0 {
            // Safety limit check
            if depth < 100 {
                path.insert(0, current_parent);
                depth += 1;
                // We'd need to load the parent to continue walking
                // This is a simplified implementation
            }
        }

        thread.depth = depth;
        thread.path = path;

        Ok(thread)
    }

    /// Resolve the byte position of `msg_num`'s header in the `.jhr` file
    async fn resolve_header_position(&self, msg_num: u32) -> Result<u64> {
        let base_header = self.load_base_header().await?;

        if msg_num < base_header.base_msg_num
            || msg_num >= base_header.base_msg_num + base_header.active
        {
            return Err(MessageError::MessageNotFound(msg_num));
        }

        let buffer = tokio::fs::read(self.jhr_path()).await?;
        let mut position = 24usize;
        let steps = (msg_num - base_header.base_msg_num) as usize;

        for _ in 0..steps {
            if position + 68 > buffer.len() {
                return Err(MessageError::MessageNotFound(msg_num));
            }
            let subfield_len =
                u32::from_le_bytes(buffer[position + 8..position + 12].try_into().map_err(
                    |_| MessageError::CorruptMessage("Truncated JAM message header".to_string()),
                )?) as usize;
            position = position
                .checked_add(68)
                .and_then(|p| p.checked_add(subfield_len))
                .ok_or_else(|| {
                    MessageError::CorruptMessage("JAM header offset overflow".to_string())
                })?;
            if position > buffer.len() {
                return Err(MessageError::MessageNotFound(msg_num));
            }
        }

        Ok(position as u64)
    }
}

#[async_trait]
impl MessageBase for JamMessageBase {
    async fn read_message(&self, msg_num: u32) -> Result<FullMessage> {
        let position = self.resolve_header_position(msg_num).await?;
        let (header, subfields) = self.load_message_header(position).await?;

        // Extract fields from subfields
        let from = Self::get_subfield_value(&subfields, SubfieldType::SendName)
            .unwrap_or_else(|| "Unknown".to_string());
        let to = Self::get_subfield_value(&subfields, SubfieldType::RecvName)
            .unwrap_or_else(|| "All".to_string());
        let subject = Self::get_subfield_value(&subfields, SubfieldType::Subject)
            .unwrap_or_else(|| "(No subject)".to_string());

        // Load message text
        let text = self
            .load_message_text(header.offset, header.text_len)
            .await?;
        let (kludges, body) = parse_kludges(&text);

        let msg_header = MessageHeader {
            msg_num,
            from,
            to,
            subject,
            date: header.written_date().unwrap_or_else(Utc::now),
            is_read: header.attributes().is_read(),
            is_private: header.attributes().is_private(),
            reply_to: if header.reply_to != 0 {
                Some(header.reply_to)
            } else {
                None
            },
            reply_count: if header.reply_1st != 0 { 1 } else { 0 },
        };

        Ok(FullMessage {
            header: msg_header,
            body,
            kludges,
        })
    }

    async fn message_count(&self) -> Result<u32> {
        let base_header = self.load_base_header().await?;
        Ok(base_header.active)
    }

    async fn search(&self, criteria: &SearchCriteria) -> Result<Vec<u32>> {
        let count = self.message_count().await?;
        let mut results = Vec::new();

        for msg_num in 1..=count {
            match self.read_message(msg_num).await {
                Ok(msg) => {
                    let mut matches = true;

                    if let Some(ref subject) = criteria.subject
                        && !msg
                            .header
                            .subject
                            .to_lowercase()
                            .contains(&subject.to_lowercase())
                    {
                        matches = false;
                    }

                    if let Some(ref from) = criteria.from
                        && !msg
                            .header
                            .from
                            .to_lowercase()
                            .contains(&from.to_lowercase())
                    {
                        matches = false;
                    }

                    if let Some(ref to) = criteria.to
                        && !msg.header.to.to_lowercase().contains(&to.to_lowercase())
                    {
                        matches = false;
                    }

                    if let Some(ref body_search) = criteria.body
                        && !msg
                            .body
                            .to_lowercase()
                            .contains(&body_search.to_lowercase())
                    {
                        matches = false;
                    }

                    if criteria.unread_only && msg.header.is_read {
                        matches = false;
                    }

                    if criteria.private_only && !msg.header.is_private {
                        matches = false;
                    }

                    if matches {
                        results.push(msg_num);
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(results)
    }

    async fn get_thread(&self, msg_num: u32) -> Result<MessageThread> {
        let position = self.resolve_header_position(msg_num).await?;
        let (header, _) = self.load_message_header(position).await?;
        self.build_thread(msg_num, &header).await
    }

    async fn list_messages(&self, start: u32, count: u32) -> Result<Vec<MessageHeader>> {
        let total = self.message_count().await?;
        let end = std::cmp::min(start + count - 1, total);
        let mut headers = Vec::new();

        for msg_num in start..=end {
            match self.read_message(msg_num).await {
                Ok(msg) => headers.push(msg.header),
                Err(_) => continue,
            }
        }

        Ok(headers)
    }

    async fn get_stats(&self) -> Result<MessageBaseStats> {
        let base_header = self.load_base_header().await?;

        Ok(MessageBaseStats {
            total_messages: base_header.active,
            unread_messages: 0, // Would need to scan all messages
            oldest_message: base_header.created_date(),
            newest_message: base_header.modified_date(),
            total_size: 0, // Would need to check file sizes
        })
    }

    async fn mark_read(&mut self, _msg_num: u32) -> Result<()> {
        // Would need to update the message header in .jhr file
        // This requires write access, which we'll implement later
        Ok(())
    }

    async fn message_exists(&self, msg_num: u32) -> Result<bool> {
        let total = self.message_count().await?;
        Ok(msg_num >= 1 && msg_num <= total)
    }

    async fn get_message_range(&self) -> Result<(u32, u32)> {
        let base_header = self.load_base_header().await?;
        let first = base_header.base_msg_num;
        let last = first + base_header.active - 1;
        Ok((first, last))
    }

    async fn post_message(&mut self, message: NewMessage) -> Result<u32> {
        // Validate message
        let validator = MessageValidator::new();
        validator.validate(&message)?;

        // Sanitize message
        let sanitizer = MessageSanitizer::new();
        let sanitized = sanitizer.sanitize(&message);

        // Get current message count
        let base_header = self.load_base_header().await?;
        let next_msg_num = base_header.base_msg_num + base_header.active;

        // Create writer
        let writer = JamWriter::new(&self.base_path);

        // Get current .JDT offset
        let current_offset = writer.get_jdt_size().await?;

        // Write message
        let (jhr_data, jdt_data, _next_offset) = writer
            .write_message(&sanitized, next_msg_num, current_offset)
            .await?;

        // Append to files
        writer.append_message(&jhr_data, &jdt_data).await?;

        // Update base header
        writer.update_base_header(base_header.active + 1).await?;

        // Invalidate cache
        {
            let mut cache = self.base_header.write().await;
            *cache = None;
        }

        Ok(next_msg_num)
    }

    async fn reply_to_message(
        &mut self,
        parent_msg_num: u32,
        mut message: NewMessage,
    ) -> Result<u32> {
        // Verify parent exists
        if !self.message_exists(parent_msg_num).await? {
            return Err(MessageError::MessageNotFound(parent_msg_num));
        }

        // Set reply_to field
        message.reply_to = Some(parent_msg_num);

        // Ensure subject starts with "Re: "
        if !message.subject.starts_with("Re: ") {
            message.subject = format!("Re: {}", message.subject);
        }

        // Post as normal message
        self.post_message(message).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jam_paths() {
        let base = JamMessageBase::new("/msg/general");
        assert_eq!(base.jhr_path(), PathBuf::from("/msg/general.jhr"));
        assert_eq!(base.jdt_path(), PathBuf::from("/msg/general.jdt"));
        assert_eq!(base.jdx_path(), PathBuf::from("/msg/general.jdx"));
    }

    #[tokio::test]
    async fn test_jam_multi_message_read_search_and_thread() {
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path().join("general");

        let writer = JamWriter::new(&base_path);
        writer.initialize_base().await.unwrap();

        let mut base = JamMessageBase::new(&base_path);

        let msg1 = NewMessage::new("Alice", "Bob", "First Topic").with_body("Hello from Alice");
        let num1 = base.post_message(msg1).await.unwrap();
        assert_eq!(num1, 1);

        let msg2 = NewMessage::new("Bob", "Alice", "Second Topic").with_body("Hello from Bob");
        let num2 = base.post_message(msg2).await.unwrap();
        assert_eq!(num2, 2);

        let reply = NewMessage::new("Carol", "Alice", "First Topic").with_body("Reply from Carol");
        let num3 = base.reply_to_message(1, reply).await.unwrap();
        assert_eq!(num3, 3);

        // Read each message by number and verify distinct headers and bodies
        let read1 = base.read_message(1).await.unwrap();
        assert_eq!(read1.header.from, "Alice");
        assert_eq!(read1.body, "Hello from Alice");

        let read2 = base.read_message(2).await.unwrap();
        assert_eq!(read2.header.from, "Bob");
        assert_eq!(read2.body, "Hello from Bob");

        let read3 = base.read_message(3).await.unwrap();
        assert_eq!(read3.header.from, "Carol");
        assert_eq!(read3.header.reply_to, Some(1));

        // Out-of-bounds message numbers return MessageNotFound
        assert!(matches!(
            base.read_message(0).await,
            Err(MessageError::MessageNotFound(0))
        ));
        assert!(matches!(
            base.read_message(99).await,
            Err(MessageError::MessageNotFound(99))
        ));

        // Search by sender
        let bob_matches = base
            .search(&SearchCriteria::new().with_from("Bob"))
            .await
            .unwrap();
        assert_eq!(bob_matches, vec![2]);

        // Thread lookup for a reply message (msg_num > 1)
        let thread3 = base.get_thread(3).await.unwrap();
        assert_eq!(thread3.root_msg, 3);
        assert_eq!(thread3.parent_id, Some(1));
        assert_eq!(thread3.depth, 1);
    }
}
