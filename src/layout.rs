use flax::{EntityRef, World};
use glam::{vec2, Vec2};
use itertools::Itertools;

use crate::{
    components::{self, children, layout, margin, padding, Edges, Rect},
    unit::Unit,
};

#[derive(Debug, Clone)]
struct MarginCursor {
    pending_margin: f32,
    start: Vec2,
    cursor: Vec2,
    line_height: f32,
    axis: Vec2,
    cross_axis: Vec2,
}

impl MarginCursor {
    fn new(start: Vec2, axis: Vec2, cross_axis: Vec2) -> Self {
        Self {
            pending_margin: 0.0,
            start,
            cursor: start,
            line_height: 0.0,
            axis,
            cross_axis,
        }
    }

    fn put(&mut self, block: &Block) -> Vec2 {
        let (front_margin, back_margin) = block.margin.in_axis(self.axis);

        let advance = (self.pending_margin.max(0.0).max(back_margin.max(0.0))
            + self.pending_margin.min(0.0)
            + back_margin.min(0.0))
        .max(0.0);

        self.pending_margin = front_margin;

        self.cursor += advance * self.axis + block.rect.support(-self.axis) * self.axis;

        let (start_margin, end_margin) = block.margin.in_axis(self.cross_axis);
        let pos = self.cursor + start_margin * self.cross_axis;

        let extent = block.rect.support(self.axis);

        self.cursor += extent * self.axis;

        self.line_height = self
            .line_height
            .max(block.rect.size().dot(self.cross_axis) + start_margin + end_margin);

        pos
    }

    fn finish(&mut self) -> Rect {
        self.cursor += self.line_height * self.cross_axis;
        self.cursor += self.pending_margin * self.axis;

        self.pending_margin = 0.0;

        let line = Rect::from_two_points(self.start, self.cursor);
        self.start = self.start * self.axis + self.cursor + self.cross_axis;

        line
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Direction {
    #[default]
    Horizontal,
    Vertical,
    HorizontalReverse,
    VerticalReverse,
}

impl Direction {
    fn axis(&self) -> (Vec2, Vec2) {
        match self {
            Direction::Horizontal => (Vec2::X, Vec2::Y),
            Direction::Vertical => (Vec2::Y, Vec2::X),
            Direction::HorizontalReverse => (-Vec2::X, Vec2::Y),
            Direction::VerticalReverse => (-Vec2::Y, Vec2::X),
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum CrossAlign {
    #[default]
    /// Align items to the start of the cross axis
    Start,
    /// Align items to the center of the cross axis
    Center,
    /// Align items to the end of the cross axis
    End,

    /// Fill the cross axis
    Stretch,
}

impl CrossAlign {
    fn align_offset(&self, total_size: f32, size: f32) -> f32 {
        match self {
            CrossAlign::Start => 0.0,
            CrossAlign::Center => (total_size - size) / 2.0,
            CrossAlign::End => total_size - size,
            CrossAlign::Stretch => 0.0,
        }
    }
}

#[derive(Default, Debug)]
pub struct Layout {
    pub cross_align: CrossAlign,
    pub direction: Direction,
}

impl Layout {
    /// Position and size the children of the given entity using all the provided available space
    ///
    /// Returns the inner rect
    fn apply(
        &self,
        world: &World,
        entity: &EntityRef,
        content_area: Rect,
        constraints: LayoutLimits,
    ) -> Rect {
        let (axis, cross_axis) = self.direction.axis();

        let (_, total_preferred_size, blocks) = self.query_size(world, entity, content_area);

        // Size remaining if everything got at least its preferred size
        let total_preferred_size = total_preferred_size.size().dot(axis);
        // let preferred_remaining =
        //     (constraints.max.dot(axis) - preferred_size.size().dot(axis)).max(0.0);
        //
        // // Size remaining if everything got at least its min size
        // let min_remaining =
        // (constraints.max.dot(axis) - min_size.size().dot(axis) - preferred_remaining).max(0.0);

        // tracing::debug!(total_preferred_size, "remaining sizes");

        let available_size = constraints.max;

        // Start at the corner of the inner rect
        //
        // The inner rect is position relative to the layouts parent
        let inner_rect = content_area;

        let mut cursor = MarginCursor::new(Vec2::ZERO, axis, cross_axis);

        // Reset to local
        let content_area = Rect {
            min: Vec2::ZERO,
            max: inner_rect.size(),
        };

        let blocks = blocks
            .into_iter()
            .map(|(entity, block)| {
                // The size required to go from min to preferred size
                let min_size = block.min.size().dot(axis);
                let preferred_size = block.preferred.size().dot(axis);

                let to_preferred = preferred_size - min_size;
                let axis_sizing = (min_size
                    + (constraints.max.dot(axis) * (to_preferred / total_preferred_size)))
                    * axis;

                // let axis_sizing = block.preferred.rect.size() * axis;

                let child_constraints = if let CrossAlign::Stretch = self.cross_align {
                    let margin = entity.get_copy(margin()).unwrap_or_default();

                    let size = inner_rect.size().min(constraints.max) - margin.size();
                    LayoutLimits {
                        min: size * cross_axis,
                        max: size * cross_axis + axis_sizing,
                    }
                } else {
                    LayoutLimits {
                        min: Vec2::ZERO,
                        max: available_size * cross_axis + axis_sizing,
                    }
                };

                // let local_rect = widget_outer_bounds(world, &child, size);
                let block = update_subtree(
                    world,
                    &entity,
                    // Supply our whole inner content area
                    content_area,
                    child_constraints,
                );

                cursor.put(&block);

                (entity, block)
            })
            .collect_vec();

        let line = cursor.finish();

        let line_size = line.size();

        let start = match self.direction {
            Direction::Horizontal => inner_rect.min,
            Direction::Vertical => inner_rect.min,
            Direction::HorizontalReverse => vec2(inner_rect.max.x, inner_rect.min.y),
            Direction::VerticalReverse => vec2(inner_rect.min.x, inner_rect.max.y),
        };

        let mut cursor = MarginCursor::new(start, axis, cross_axis);

        for (entity, block) in blocks {
            // And move it all by the cursor position
            let height = (block.rect.size() + block.margin.size()).dot(cross_axis);

            let pos = cursor.put(&block)
                + self
                    .cross_align
                    .align_offset(line_size.dot(cross_axis), height)
                    * cross_axis;

            entity.update_dedup(components::rect(), block.rect);
            entity.update_dedup(components::local_position(), pos);
        }

        cursor.finish()
    }

    pub(crate) fn query_size<'a>(
        &self,
        world: &'a World,
        entity: &EntityRef,
        inner_rect: Rect,
    ) -> (Rect, Rect, Vec<(EntityRef<'a>, SizeQuery)>) {
        let children = entity.get(children()).ok();
        let children = children.as_ref().map(|v| v.as_slice()).unwrap_or_default();

        // let available_size = inner_rect.size();

        // Start at the corner of the inner rect
        //
        // The inner rect is position relative to the layouts parent

        let (axis, cross_axis) = self.direction.axis();

        let mut min_cursor = MarginCursor::new(Vec2::ZERO, axis, cross_axis);
        let mut preferred_cursor = MarginCursor::new(Vec2::ZERO, axis, cross_axis);

        // Reset to local
        let content_area = Rect {
            min: Vec2::ZERO,
            max: inner_rect.size(),
        };

        let blocks = children
            .iter()
            .map(|&child| {
                let entity = world.entity(child).expect("Invalid child");

                // let local_rect = widget_outer_bounds(world, &child, size);
                let query = query_size(world, &entity, content_area);

                min_cursor.put(&Block::new(query.min, query.margin));
                preferred_cursor.put(&Block::new(query.preferred, query.margin));
                (entity, query)
            })
            .collect_vec();

        (min_cursor.finish(), preferred_cursor.finish(), blocks)
    }
}

pub struct SizeQuery {
    min: Rect,
    preferred: Rect,
    margin: Edges,
}

pub fn query_size(world: &World, entity: &EntityRef, content_area: Rect) -> SizeQuery {
    let margin = entity
        .get(components::margin())
        .ok()
        .as_deref()
        .copied()
        .unwrap_or_default();

    let padding = entity
        .get(padding())
        .ok()
        .as_deref()
        .copied()
        .unwrap_or_default();

    // Flow
    if let Ok(layout) = entity.get(layout()) {
        // For a given layout use the largest size that fits within the constraints and then
        // potentially shrink it down.

        let (min, preferred, _) = layout.query_size(world, entity, content_area.inset(&padding));

        SizeQuery {
            min: min.pad(&padding),
            preferred: preferred.pad(&padding),
            margin,
        }
    }
    // Stack
    else if let Ok(children) = entity.get(children()) {
        todo!()
    } else {
        let (min_size, preferred_size) = resolve_size(entity, content_area);

        let min_offset = resolve_pos(entity, content_area, min_size);
        let preferred_offset = resolve_pos(entity, content_area, preferred_size);

        // Leaf

        SizeQuery {
            min: Rect::from_size_pos(min_size, min_offset),
            preferred: Rect::from_size_pos(preferred_size, preferred_offset),
            margin,
        }
    }
}

/// Constraints for a child widget passed down from the parent.
///
/// Allows for the parent to control the size of the children, such as stretching
#[derive(Debug, Clone, Copy)]
pub(crate) struct LayoutLimits {
    pub min: Vec2,
    pub max: Vec2,
}

/// A block is a rectangle and surrounding support such as margin
#[derive(Debug, Clone, Copy)]
pub(crate) struct Block {
    pub(crate) rect: Rect,
    pub(crate) margin: Edges,
}

impl Block {
    pub(crate) fn new(rect: Rect, margin: Edges) -> Self {
        Self { rect, margin }
    }
}

/// Updates the layout of the given subtree given the passes constraints.
///
/// Returns the outer bounds of the subtree.
#[must_use = "This function does not mutate the entity"]
pub(crate) fn update_subtree(
    world: &World,
    entity: &EntityRef,
    // The area in which children can be placed without clipping
    content_area: Rect,
    limits: LayoutLimits,
) -> Block {
    // let _span = tracing::info_span!( "Updating subtree", %entity, ?constraints).entered();
    let margin = entity
        .get(components::margin())
        .ok()
        .as_deref()
        .copied()
        .unwrap_or_default();

    let padding = entity
        .get(padding())
        .ok()
        .as_deref()
        .copied()
        .unwrap_or_default();

    // Flow
    if let Ok(layout) = entity.get(layout()) {
        // For a given layout use the largest size that fits within the constraints and then
        // potentially shrink it down.

        let rect = layout
            .apply(
                world,
                entity,
                content_area.inset(&padding),
                LayoutLimits {
                    min: limits.min,
                    max: limits.max - padding.size(),
                },
            )
            .pad(&padding)
            .clamp(limits.min, limits.max);

        Block { rect, margin }
    }
    // Stack
    else if let Ok(children) = entity.get(children()) {
        let total_bounds = Rect {
            min: Vec2::ZERO,
            max: Vec2::ONE,
        };

        for &child in &*children {
            let entity = world.entity(child).unwrap();

            // let local_rect = widget_outer_bounds(world, &entity, inner_rect.size());

            assert_eq!(content_area.size(), limits.max);
            let constraints = LayoutLimits {
                min: Vec2::ZERO,
                max: limits.max - padding.size(),
            };

            // We ask ourselves the question:
            //
            // Relative to ourselves, where can our children be placed without clipping.
            //
            // The answer is a origin bound rect of the same size as our content area, inset by the
            // imposed padding.
            let content_area = Rect {
                min: Vec2::ZERO,
                max: content_area.size(),
            }
            .inset(&padding);
            assert_eq!(content_area.size(), constraints.max);

            let res = update_subtree(world, &entity, content_area, constraints);

            entity.update_dedup(components::rect(), res.rect);
        }
        Block {
            rect: total_bounds,
            margin,
        }
    } else {
        let size = resolve_size(entity, content_area)
            .1
            .clamp(limits.min, limits.max);

        let pos = resolve_pos(entity, content_area, size);

        Block {
            rect: Rect::from_size_pos(size, pos),
            margin,
        }
    }
}

fn resolve_size(entity: &EntityRef, content_area: Rect) -> (Vec2, Vec2) {
    let parent_size = content_area.size();
    let min_size = entity
        .get(components::min_size())
        .as_deref()
        .unwrap_or(&Unit::ZERO)
        .resolve(parent_size);

    let size = entity
        .get(components::size())
        .as_deref()
        .unwrap_or(&Unit::ZERO)
        .resolve(parent_size)
        .max(min_size);

    (min_size, size)
}

fn resolve_pos(entity: &EntityRef, content_area: Rect, self_size: Vec2) -> Vec2 {
    let offset = entity.get(components::offset());
    let anchor = entity.get(components::anchor());

    let offset = offset
        .as_deref()
        .unwrap_or(&Unit::ZERO)
        .resolve(content_area.size());

    let pos =
        content_area.pos() + offset - anchor.as_deref().unwrap_or(&Unit::ZERO).resolve(self_size);
    pos
}
