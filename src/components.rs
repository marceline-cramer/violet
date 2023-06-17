use flax::{component, Debuggable, Entity};
use glam::{vec2, Vec2};

use crate::{layout::Layout, shapes::Shape, Constraints};

component! {
    /// Ordered list of children for an entity
    pub children: Vec<Entity> => [ Debuggable ],
    // pub child_of(parent): Entity => [ Debuggable ],

    /// The shape of a widget when drawn
    pub shape: Shape => [ Debuggable ],

    /// Defines the outer bounds of a widget relative to its position
    pub rect: Rect => [ Debuggable ],

    /// Position relative to parent
    pub local_position: Vec2 => [ Debuggable ],

    /// Specifies in screen space where the widget rect upper left corner is
    pub screen_position: Vec2 => [ Debuggable ],

    /// Linear constraints for widget positioning and size
    pub constraints: Constraints => [ Debuggable ],

    /// Manages the layout of the children
    pub layout: Layout => [ Debuggable ],

    /// Spacing between a outer and inner bounds
    pub padding: Edges => [ Debuggable ],
    pub margin: Edges => [ Debuggable ],

}

/// Spacing between a outer and inner bounds
#[derive(Clone, Copy, Debug, Default)]
pub struct Edges {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Edges {
    pub fn new(left: f32, right: f32, top: f32, bottom: f32) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    pub fn even(distance: f32) -> Self {
        Self {
            left: distance,
            right: distance,
            top: distance,
            bottom: distance,
        }
    }

    pub(crate) fn size(&self) -> Vec2 {
        vec2(self.left + self.right, self.top + self.bottom)
    }
}

#[derive(Clone, Copy, Debug, Default)]
/// Defines the penultimate bounds of a widget
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    pub fn from_size_pos(size: Vec2, pos: Vec2) -> Self {
        Self {
            min: pos,
            max: pos + size,
        }
    }

    #[inline]
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    #[inline]
    pub fn pos(&self) -> Vec2 {
        self.min
    }

    /// Makes the rect smaller by the given padding
    pub fn inset(&self, padding: &Edges) -> Rect {
        Self {
            min: self.min + vec2(padding.left, padding.top),
            max: self.max - vec2(padding.right, padding.bottom),
        }
    }

    /// Makes the rect larger by the given padding
    pub fn pad(&self, padding: &Edges) -> Rect {
        Self {
            min: self.min - vec2(padding.left, padding.top),
            max: self.max + vec2(padding.right, padding.bottom),
        }
    }

    pub(crate) fn translate(&self, v: Vec2) -> Self {
        Self {
            min: self.min + v,
            max: self.max + v,
        }
    }
}
