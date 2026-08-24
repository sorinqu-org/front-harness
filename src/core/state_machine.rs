use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelinePhase {
    Idle,
    Auditing,
    Researching,
    Designing,
    Implementing,
    Verifying,
    Completed,
    Failed,
}

impl std::fmt::Display for PipelinePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "Idle"),
            Self::Auditing => write!(f, "Auditing"),
            Self::Researching => write!(f, "Researching"),
            Self::Designing => write!(f, "Designing"),
            Self::Implementing => write!(f, "Implementing"),
            Self::Verifying => write!(f, "Verifying"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed => write!(f, "Failed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSnapshot {
    pub step_index: usize,
    pub phase: PipelinePhase,
    pub timestamp: DateTime<Utc>,
    pub state_data: serde_json::Value,
}

pub struct StateMachine {
    current_phase: PipelinePhase,
    history: Vec<PipelineSnapshot>,
    step_counter: usize,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            current_phase: PipelinePhase::Idle,
            history: Vec::new(),
            step_counter: 0,
        }
    }

    pub fn current_phase(&self) -> &PipelinePhase {
        &self.current_phase
    }

    pub fn transition_to(&mut self, next: PipelinePhase, data: serde_json::Value) -> Result<()> {
        let snapshot = PipelineSnapshot {
            step_index: self.step_counter,
            phase: self.current_phase.clone(),
            timestamp: Utc::now(),
            state_data: data,
        };
        self.history.push(snapshot);
        self.step_counter += 1;
        self.current_phase = next;
        Ok(())
    }

    pub fn rollback(&mut self) -> Result<PipelineSnapshot> {
        if let Some(prev) = self.history.pop() {
            self.current_phase = prev.phase.clone();
            self.step_counter = prev.step_index;
            Ok(prev)
        } else {
            bail!("No previous snapshot to rollback to");
        }
    }

    pub fn history(&self) -> &[PipelineSnapshot] {
        &self.history
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}
