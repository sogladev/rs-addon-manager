use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Serialize, Clone, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum OperationType {
    Install,
    Update,
}

#[derive(Debug, Serialize, Clone, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum OperationEvent {
    Started { operation: OperationType },
    Progress { current: usize, total: usize },
    Status(String),
    Warning(String),
    Error(String),
    Completed,
}

#[derive(Debug, Serialize, Clone, TS, PartialEq, Eq, Hash)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct OperationKey {
    pub repo_url: String,
    pub folder_path: String,
}

#[derive(Debug, Serialize, Clone, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct OperationEventPayload {
    pub key: OperationKey,
    pub event: OperationEvent,
}

/// Global operation tracker for managing ongoing install/update operations
pub struct OperationTracker {
    active_operations: Arc<Mutex<HashMap<OperationKey, OperationType>>>,
}

impl OperationTracker {
    pub fn new() -> Self {
        Self {
            active_operations: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn start_operation(&self, key: &OperationKey, operation: OperationType) {
        if let Ok(mut ops) = self.active_operations.lock() {
            ops.insert(key.clone(), operation);
        }
    }

    pub fn finish_operation(&self, key: &OperationKey) {
        if let Ok(mut ops) = self.active_operations.lock() {
            ops.remove(key);
        }
    }

    pub fn is_active(&self, key: &OperationKey) -> bool {
        if let Ok(ops) = self.active_operations.lock() {
            ops.contains_key(key)
        } else {
            false
        }
    }

    pub fn get_operation_type(&self, key: &OperationKey) -> Option<OperationType> {
        if let Ok(ops) = self.active_operations.lock() {
            ops.get(key).cloned()
        } else {
            None
        }
    }
}

impl Default for OperationTracker {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OperationReporter {
    pub key: OperationKey,
    pub event_fn: Box<dyn FnMut(OperationEvent) + Send>,
}

impl OperationReporter {
    pub fn new<F>(key: OperationKey, event_fn: F) -> Self
    where
        F: FnMut(OperationEvent) + Send + 'static,
    {
        Self {
            key,
            event_fn: Box::new(event_fn),
        }
    }

    pub fn progress(&mut self, current: usize, total: usize) {
        (self.event_fn)(OperationEvent::Progress { current, total });
    }

    pub fn status<S: Into<String>>(&mut self, message: S) {
        (self.event_fn)(OperationEvent::Status(message.into()));
    }

    pub fn warning<S: Into<String>>(&mut self, message: S) {
        (self.event_fn)(OperationEvent::Warning(message.into()));
    }

    pub fn error<S: Into<String>>(&mut self, message: S) {
        (self.event_fn)(OperationEvent::Error(message.into()));
    }

    pub fn completed(&mut self) {
        (self.event_fn)(OperationEvent::Completed);
    }
}
