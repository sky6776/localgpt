//! Undo/redo history — edit operations and their inverses.

use serde::{Deserialize, Serialize};

use crate::entity::{EntityPatch, WorldEntity};
use crate::identity::EntityId;
use crate::world::{CameraDef, EnvironmentDef};

/// A recorded world edit with its inverse for undo support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldEdit {
    /// Monotonically increasing sequence number.
    pub seq: u64,
    /// The operation that was performed.
    pub op: EditOp,
    /// The inverse operation (for undo).
    pub inverse: EditOp,
    /// Timestamp in milliseconds since epoch.
    pub timestamp_ms: u64,
    /// Who performed the edit (e.g., "user", "llm", agent ID).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

/// An atomic edit operation on the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditOp {
    /// Spawn a new entity.
    SpawnEntity { entity: WorldEntity },
    /// Delete an entity by ID.
    DeleteEntity { id: EntityId },
    /// Modify an entity with a patch.
    ModifyEntity { id: EntityId, patch: EntityPatch },
    /// Set environment (background color, ambient light, fog).
    SetEnvironment { env: EnvironmentDef },
    /// Set camera position, look-at target, and FOV.
    SetCamera { camera: CameraDef },
    /// A batch of atomic operations (all-or-nothing).
    Batch { ops: Vec<EditOp> },
}

impl EditOp {
    /// Compute the inverse of this operation.
    ///
    /// For `SpawnEntity`, the inverse is `DeleteEntity`.
    /// For `DeleteEntity`, the caller must provide the entity state to restore.
    /// For `ModifyEntity`, the caller must provide the previous state.
    /// For `Batch`, the inverse is a batch of inverses in reverse order.
    pub fn compute_inverse_spawn(entity: &WorldEntity) -> EditOp {
        EditOp::DeleteEntity { id: entity.id }
    }

    /// Create a spawn operation for an entity.
    pub fn spawn(entity: WorldEntity) -> EditOp {
        EditOp::SpawnEntity { entity }
    }

    /// Create a delete operation.
    pub fn delete(id: EntityId) -> EditOp {
        EditOp::DeleteEntity { id }
    }

    /// Create a modify operation.
    pub fn modify(id: EntityId, patch: EntityPatch) -> EditOp {
        EditOp::ModifyEntity { id, patch }
    }
}

/// Edit history — append-only log of world edits.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditHistory {
    /// All edits in chronological order.
    pub edits: Vec<WorldEdit>,
    /// Current position in the edit log (for undo/redo).
    /// Points to the next edit to be undone.
    pub cursor: usize,
}

impl EditHistory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a new edit. Truncates any redo history.
    pub fn push(&mut self, op: EditOp, inverse: EditOp, author: Option<String>) {
        // Truncate redo history
        self.edits.truncate(self.cursor);

        let seq = self.edits.len() as u64;
        self.edits.push(WorldEdit {
            seq,
            op,
            inverse,
            timestamp_ms: 0, // Caller should set this
            author,
        });
        self.cursor = self.edits.len();
    }

    /// Get the next operation to undo, if any.
    pub fn undo(&mut self) -> Option<&EditOp> {
        if self.cursor == 0 {
            return None;
        }
        self.cursor -= 1;
        Some(&self.edits[self.cursor].inverse)
    }

    /// Get the next operation to redo, if any.
    pub fn redo(&mut self) -> Option<&EditOp> {
        if self.cursor >= self.edits.len() {
            return None;
        }
        let op = &self.edits[self.cursor].op;
        self.cursor += 1;
        Some(op)
    }

    /// Number of operations that can be undone.
    pub fn undo_count(&self) -> usize {
        self.cursor
    }

    /// Number of operations that can be redone.
    pub fn redo_count(&self) -> usize {
        self.edits.len() - self.cursor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::WorldEntity;

    #[test]
    fn undo_redo_basic() {
        let mut history = EditHistory::new();

        let entity = WorldEntity::new(1, "cube");
        let op = EditOp::spawn(entity.clone());
        let inverse = EditOp::delete(entity.id);

        history.push(op, inverse, None);
        assert_eq!(history.undo_count(), 1);
        assert_eq!(history.redo_count(), 0);

        // Undo
        let undo_op = history.undo().unwrap();
        assert!(matches!(undo_op, EditOp::DeleteEntity { .. }));
        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 1);

        // Redo
        let redo_op = history.redo().unwrap();
        assert!(matches!(redo_op, EditOp::SpawnEntity { .. }));
        assert_eq!(history.undo_count(), 1);
        assert_eq!(history.redo_count(), 0);
    }

    #[test]
    fn new_edit_truncates_redo() {
        let mut history = EditHistory::new();

        // Push two edits
        for i in 0..2 {
            let entity = WorldEntity::new(i, format!("e{i}"));
            history.push(
                EditOp::spawn(entity.clone()),
                EditOp::delete(entity.id),
                None,
            );
        }

        // Undo one
        history.undo();
        assert_eq!(history.redo_count(), 1);

        // Push a new edit — should truncate redo
        let entity = WorldEntity::new(99, "new");
        history.push(
            EditOp::spawn(entity.clone()),
            EditOp::delete(entity.id),
            None,
        );
        assert_eq!(history.redo_count(), 0);
        assert_eq!(history.undo_count(), 2); // first + new
    }
}
