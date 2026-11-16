//! Touch contacts retain their original key and commit in release order.
use calculator_core::Action;
use eframe::egui::{Pos2, Rect};
use std::collections::HashMap;
#[derive(Clone, Copy)]
pub struct Target {
    pub rect: Rect,
    pub action: Action,
    pub group: u8,
}
#[derive(Default)]
pub struct TouchController {
    contacts: HashMap<u64, Target>,
}
impl TouchController {
    pub fn is_pressed_at(&self, pos: Pos2) -> bool {
        self.contacts
            .values()
            .any(|target| target.rect.contains(pos))
    }
    pub fn is_active(&self) -> bool {
        !self.contacts.is_empty()
    }
    pub fn start(&mut self, id: u64, pos: Pos2, targets: &[Target]) {
        if let Some(target) = targets.iter().find(|t| t.rect.contains(pos)) {
            self.contacts.insert(id, *target);
        }
    }
    pub fn cancel_contact(&mut self, id: u64) {
        self.contacts.remove(&id);
    }
    pub fn cancel(&mut self) {
        self.contacts.clear();
    }
    pub fn end(&mut self, id: u64, pos: Pos2, targets: &[Target]) -> Option<Action> {
        let target = self.contacts.remove(&id)?;
        let rect = target.rect;
        let row: Vec<_> = targets
            .iter()
            .filter(|t| {
                t.group == target.group && (t.rect.center().y - rect.center().y).abs() < 2.0
            })
            .collect();
        let left = row
            .iter()
            .filter(|t| t.rect.max.x < rect.min.x)
            .max_by(|a, b| a.rect.max.x.total_cmp(&b.rect.max.x));
        let right = row
            .iter()
            .filter(|t| t.rect.min.x > rect.max.x)
            .min_by(|a, b| a.rect.min.x.total_cmp(&b.rect.min.x));
        let left_margin = left
            .map(|t| rect.min.x - t.rect.min.x)
            .or_else(|| right.map(|t| t.rect.max.x - rect.max.x))
            .unwrap_or(rect.width() * 0.5);
        let right_margin = right
            .map(|t| t.rect.max.x - rect.max.x)
            .or_else(|| left.map(|t| rect.min.x - t.rect.min.x))
            .unwrap_or(rect.width() * 0.5);
        let above = targets
            .iter()
            .filter(|t| t.group == target.group && t.rect.max.y < rect.min.y)
            .max_by(|a, b| a.rect.max.y.total_cmp(&b.rect.max.y));
        let below = targets
            .iter()
            .filter(|t| t.group == target.group && t.rect.min.y > rect.max.y)
            .min_by(|a, b| a.rect.min.y.total_cmp(&b.rect.min.y));
        let top_margin = above
            .map(|t| rect.min.y - t.rect.min.y)
            .or_else(|| below.map(|t| t.rect.max.y - rect.max.y))
            .unwrap_or(rect.height() * 0.5);
        let bottom_margin = below
            .map(|t| t.rect.max.y - rect.max.y)
            .or_else(|| above.map(|t| rect.min.y - t.rect.min.y))
            .unwrap_or(rect.height() * 0.5);
        Rect::from_min_max(
            Pos2::new(rect.min.x - left_margin, rect.min.y - top_margin),
            Pos2::new(rect.max.x + right_margin, rect.max.y + bottom_margin),
        )
        .contains(pos)
        .then_some(target.action)
    }
}
