//! Additional [`GizmoBuffer`] Functions -- Arcs
//!
//! Includes the implementation of [`GizmoBuffer::arc_2d`],
//! and assorted support items.

use crate::{circles::DEFAULT_CIRCLE_RESOLUTION, gizmos::GizmoBuffer, prelude::GizmoConfigGroup};
use bevy_color::Color;
use bevy_math::{Isometry2d, Isometry3d, Quat, Rot2, Vec2, Vec3};
use core::f32::consts::{FRAC_PI_2, TAU};

// === 2D ===

impl<Config, Clear> GizmoBuffer<Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    /// Draw an arc, which is a part of the circumference of a circle, in 2D.
    ///
    /// This should be called for each frame the arc needs to be rendered.
    ///
    /// # Arguments
    /// - `isometry` defines the translation and rotation of the arc.
    ///   - the translation specifies the center of the arc
    ///   - the rotation is counter-clockwise starting from `Vec2::Y`
    /// - `arc_angle` sets the length of this arc, in radians.
    /// - `radius` controls the distance from `position` to this arc, and thus its curvature.
    /// - `color` sets the color to draw the arc.
    ///
    /// # Example
    /// ```
    /// # use bevy_gizmos::prelude::*;
    /// # use bevy_math::prelude::*;
    /// # use std::f32::consts::FRAC_PI_4;
    /// # use bevy_color::palettes::basic::{GREEN, RED};
    /// fn system(mut gizmos: Gizmos) {
    ///     gizmos.arc_2d(Isometry2d::IDENTITY, FRAC_PI_4, 1., GREEN);
    ///
    ///     // Arcs have 32 line-segments by default.
    ///     // You may want to increase this for larger arcs.
    ///     gizmos
    ///         .arc_2d(Isometry2d::IDENTITY, FRAC_PI_4, 5., RED)
    ///         .resolution(64);
    /// }
    /// # bevy_ecs::system::assert_is_system(system);
    /// ```
    #[inline]
    pub fn arc_2d(
        &mut self,
        isometry: impl Into<Isometry2d>,
        arc_angle: f32,
        radius: f32,
        color: impl Into<Color>,
    ) -> Arc2dBuilder<'_, Config, Clear> {
        Arc2dBuilder {
            gizmos: self,
            isometry: isometry.into(),
            arc_angle,
            radius,
            color: color.into(),
            resolution: None,
        }
    }
}

/// A builder returned by [`GizmoBuffer::arc_2d`].
pub struct Arc2dBuilder<'a, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    gizmos: &'a mut GizmoBuffer<Config, Clear>,
    isometry: Isometry2d,
    arc_angle: f32,
    radius: f32,
    color: Color,
    resolution: Option<u32>,
}

impl<Config, Clear> Arc2dBuilder<'_, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    /// Set the number of lines used to approximate the geometry of this arc.
    pub fn resolution(mut self, resolution: u32) -> Self {
        self.resolution.replace(resolution);
        self
    }
}

impl<Config, Clear> Drop for Arc2dBuilder<'_, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    fn drop(&mut self) {
        if !self.gizmos.enabled {
            return;
        }

        let resolution = self
            .resolution
            .unwrap_or_else(|| resolution_from_angle(self.arc_angle));

        let positions =
            arc_2d_inner(self.arc_angle, self.radius, resolution).map(|vec2| self.isometry * vec2);
        self.gizmos.linestrip_2d(positions, self.color);
    }
}

fn arc_2d_inner(arc_angle: f32, radius: f32, resolution: u32) -> impl Iterator<Item = Vec2> {
    (0..=resolution)
        .map(move |n| arc_angle * n as f32 / resolution as f32)
        .map(|angle| angle + FRAC_PI_2)
        .map(Vec2::from_angle)
        .map(move |vec2| vec2 * radius)
}

// === 3D ===

impl<Config, Clear> GizmoBuffer<Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    /// Draw an arc, which is a part of the circumference of a circle, in 3D. For default values
    /// this is drawing a standard arc. A standard arc is defined as
    ///
    /// - an arc with a center at `Vec3::ZERO`
    /// - starting at `Vec3::X`
    /// - embedded in the XZ plane
    /// - rotates counterclockwise
    ///
    /// This should be called for each frame the arc needs to be rendered.
    ///
    /// # Arguments
    /// - `angle`: sets how much of a circle circumference is passed, e.g. PI is half a circle. This
    ///   value should be in the range (-2 * PI..=2 * PI)
    /// - `radius`: distance between the arc and its center point
    /// - `isometry` defines the translation and rotation of the arc.
    ///   - the translation specifies the center of the arc
    ///   - the rotation is counter-clockwise starting from `Vec3::Y`
    /// - `color`: color of the arc
    ///
    /// # Builder methods
    /// The resolution of the arc (i.e. the level of detail) can be adjusted with the
    /// `.resolution(...)` method.
    ///
    /// # Example
    /// ```
    /// # use bevy_gizmos::prelude::*;
    /// # use bevy_math::prelude::*;
    /// # use std::f32::consts::PI;
    /// # use bevy_color::palettes::css::ORANGE;
    /// fn system(mut gizmos: Gizmos) {
    ///     // rotation rotates normal to point in the direction of `Vec3::NEG_ONE`
    ///     let rotation = Quat::from_rotation_arc(Vec3::Y, Vec3::NEG_ONE.normalize());
    ///
    ///     gizmos
    ///        .arc_3d(
    ///          270.0_f32.to_radians(),
    ///          0.25,
    ///          Isometry3d::new(Vec3::ONE, rotation),
    ///          ORANGE
    ///          )
    ///          .resolution(100);
    /// }
    /// # bevy_ecs::system::assert_is_system(system);
    /// ```
    #[inline]
    pub fn arc_3d(
        &mut self,
        angle: f32,
        radius: f32,
        isometry: impl Into<Isometry3d>,
        color: impl Into<Color>,
    ) -> Arc3dBuilder<'_, Config, Clear> {
        Arc3dBuilder {
            gizmos: self,
            start_vertex: Vec3::X,
            isometry: isometry.into(),
            angle,
            radius,
            color: color.into(),
            resolution: None,
        }
    }

    /// Draws the shortest arc between two points (`from` and `to`) relative to a specified `center` point.
    ///
    /// # Arguments
    ///
    /// - `center`: The center point around which the arc is drawn.
    /// - `from`: The starting point of the arc.
    /// - `to`: The ending point of the arc.
    /// - `color`: color of the arc
    ///
    /// # Builder methods
    /// The resolution of the arc (i.e. the level of detail) can be adjusted with the
    /// `.resolution(...)` method.
    ///
    /// # Examples
    /// ```
    /// # use bevy_gizmos::prelude::*;
    /// # use bevy_math::prelude::*;
    /// # use bevy_color::palettes::css::ORANGE;
    /// fn system(mut gizmos: Gizmos) {
    ///     gizmos.short_arc_3d_between(
    ///        Vec3::ONE,
    ///        Vec3::ONE + Vec3::NEG_ONE,
    ///        Vec3::ZERO,
    ///        ORANGE
    ///        )
    ///        .resolution(100);
    /// }
    /// # bevy_ecs::system::assert_is_system(system);
    /// ```
    ///
    /// # Notes
    /// - This method assumes that the points `from` and `to` are distinct from `center`. If one of
    ///   the points is coincident with `center`, nothing is rendered.
    /// - The arc is drawn as a portion of a circle with a radius equal to the distance from the
    ///   `center` to `from`. If the distance from `center` to `to` is not equal to the radius, then
    ///   the results will behave as if this were the case
    #[inline]
    pub fn short_arc_3d_between(
        &mut self,
        center: Vec3,
        from: Vec3,
        to: Vec3,
        color: impl Into<Color>,
    ) -> Arc3dBuilder<'_, Config, Clear> {
        self.arc_from_to(center, from, to, color, |x| x)
    }

    /// Draws the longest arc between two points (`from` and `to`) relative to a specified `center` point.
    ///
    /// # Arguments
    /// - `center`: The center point around which the arc is drawn.
    /// - `from`: The starting point of the arc.
    /// - `to`: The ending point of the arc.
    /// - `color`: color of the arc
    ///
    /// # Builder methods
    /// The resolution of the arc (i.e. the level of detail) can be adjusted with the
    /// `.resolution(...)` method.
    ///
    /// # Examples
    /// ```
    /// # use bevy_gizmos::prelude::*;
    /// # use bevy_math::prelude::*;
    /// # use bevy_color::palettes::css::ORANGE;
    /// fn system(mut gizmos: Gizmos) {
    ///     gizmos.long_arc_3d_between(
    ///        Vec3::ONE,
    ///        Vec3::ONE + Vec3::NEG_ONE,
    ///        Vec3::ZERO,
    ///        ORANGE
    ///        )
    ///        .resolution(100);
    /// }
    /// # bevy_ecs::system::assert_is_system(system);
    /// ```
    ///
    /// # Notes
    /// - This method assumes that the points `from` and `to` are distinct from `center`. If one of
    ///   the points is coincident with `center`, nothing is rendered.
    /// - The arc is drawn as a portion of a circle with a radius equal to the distance from the
    ///   `center` to `from`. If the distance from `center` to `to` is not equal to the radius, then
    ///   the results will behave as if this were the case.
    #[inline]
    pub fn long_arc_3d_between(
        &mut self,
        center: Vec3,
        from: Vec3,
        to: Vec3,
        color: impl Into<Color>,
    ) -> Arc3dBuilder<'_, Config, Clear> {
        self.arc_from_to(center, from, to, color, |angle| {
            if angle > 0.0 {
                TAU - angle
            } else if angle < 0.0 {
                -TAU - angle
            } else {
                0.0
            }
        })
    }

    #[inline]
    fn arc_from_to(
        &mut self,
        center: Vec3,
        from: Vec3,
        to: Vec3,
        color: impl Into<Color>,
        angle_fn: impl Fn(f32) -> f32,
    ) -> Arc3dBuilder<'_, Config, Clear> {
        // `from` and `to` can be the same here since in either case nothing gets rendered and the
        // orientation ambiguity of `up` doesn't matter
        let from_axis = (from - center).normalize_or_zero();
        let to_axis = (to - center).normalize_or_zero();
        let (up, angle) = Quat::from_rotation_arc(from_axis, to_axis).to_axis_angle();

        let angle = angle_fn(angle);
        let radius = center.distance(from);
        let rotation = Quat::from_rotation_arc(Vec3::Y, up);

        let start_vertex = rotation.inverse() * from_axis;

        Arc3dBuilder {
            gizmos: self,
            start_vertex,
            isometry: Isometry3d::new(center, rotation),
            angle,
            radius,
            color: color.into(),
            resolution: None,
        }
    }

    /// Draws the shortest arc between two points (`from` and `to`) relative to a specified `center` point.
    ///
    /// # Arguments
    ///
    /// - `center`: The center point around which the arc is drawn.
    /// - `from`: The starting point of the arc.
    /// - `to`: The ending point of the arc.
    /// - `color`: color of the arc
    ///
    /// # Builder methods
    /// The resolution of the arc (i.e. the level of detail) can be adjusted with the
    /// `.resolution(...)` method.
    ///
    /// # Examples
    /// ```
    /// # use bevy_gizmos::prelude::*;
    /// # use bevy_math::prelude::*;
    /// # use bevy_color::palettes::css::ORANGE;
    /// fn system(mut gizmos: Gizmos) {
    ///     gizmos.short_arc_2d_between(
    ///        Vec2::ZERO,
    ///        Vec2::X,
    ///        Vec2::Y,
    ///        ORANGE
    ///        )
    ///        .resolution(100);
    /// }
    /// # bevy_ecs::system::assert_is_system(system);
    /// ```
    ///
    /// # Notes
    /// - This method assumes that the points `from` and `to` are distinct from `center`. If one of
    ///   the points is coincident with `center`, nothing is rendered.
    /// - The arc is drawn as a portion of a circle with a radius equal to the distance from the
    ///   `center` to `from`. If the distance from `center` to `to` is not equal to the radius, then
    ///   the results will behave as if this were the case
    #[inline]
    pub fn short_arc_2d_between(
        &mut self,
        center: Vec2,
        from: Vec2,
        to: Vec2,
        color: impl Into<Color>,
    ) -> Arc2dBuilder<'_, Config, Clear> {
        self.arc_2d_from_to(center, from, to, color, core::convert::identity)
    }

    /// Draws the longest arc between two points (`from` and `to`) relative to a specified `center` point.
    ///
    /// # Arguments
    /// - `center`: The center point around which the arc is drawn.
    /// - `from`: The starting point of the arc.
    /// - `to`: The ending point of the arc.
    /// - `color`: color of the arc
    ///
    /// # Builder methods
    /// The resolution of the arc (i.e. the level of detail) can be adjusted with the
    /// `.resolution(...)` method.
    ///
    /// # Examples
    /// ```
    /// # use bevy_gizmos::prelude::*;
    /// # use bevy_math::prelude::*;
    /// # use bevy_color::palettes::css::ORANGE;
    /// fn system(mut gizmos: Gizmos) {
    ///     gizmos.long_arc_2d_between(
    ///        Vec2::ZERO,
    ///        Vec2::X,
    ///        Vec2::Y,
    ///        ORANGE
    ///        )
    ///        .resolution(100);
    /// }
    /// # bevy_ecs::system::assert_is_system(system);
    /// ```
    ///
    /// # Notes
    /// - This method assumes that the points `from` and `to` are distinct from `center`. If one of
    ///   the points is coincident with `center`, nothing is rendered.
    /// - The arc is drawn as a portion of a circle with a radius equal to the distance from the
    ///   `center` to `from`. If the distance from `center` to `to` is not equal to the radius, then
    ///   the results will behave as if this were the case.
    #[inline]
    pub fn long_arc_2d_between(
        &mut self,
        center: Vec2,
        from: Vec2,
        to: Vec2,
        color: impl Into<Color>,
    ) -> Arc2dBuilder<'_, Config, Clear> {
        self.arc_2d_from_to(center, from, to, color, |angle| angle - TAU)
    }

    #[inline]
    fn arc_2d_from_to(
        &mut self,
        center: Vec2,
        from: Vec2,
        to: Vec2,
        color: impl Into<Color>,
        angle_fn: impl Fn(f32) -> f32,
    ) -> Arc2dBuilder<'_, Config, Clear> {
        // `from` and `to` can be the same here since in either case nothing gets rendered and the
        // orientation ambiguity of `up` doesn't matter
        let from_axis = (from - center).normalize_or_zero();
        let to_axis = (to - center).normalize_or_zero();
        let rotation = Vec2::Y.angle_to(from_axis);
        let arc_angle_raw = from_axis.angle_to(to_axis);

        let arc_angle = angle_fn(arc_angle_raw);
        let radius = center.distance(from);

        Arc2dBuilder {
            gizmos: self,
            isometry: Isometry2d::new(center, Rot2::radians(rotation)),
            arc_angle,
            radius,
            color: color.into(),
            resolution: None,
        }
    }
}

/// A builder returned by [`GizmoBuffer::arc_2d`].
pub struct Arc3dBuilder<'a, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    gizmos: &'a mut GizmoBuffer<Config, Clear>,
    // this is the vertex the arc starts on in the XZ plane. For the normal arc_3d method this is
    // always starting at Vec3::X. For the short/long arc methods we actually need a way to start
    // at the from position and this is where this internal field comes into play. Some implicit
    // assumptions:
    //
    // 1. This is always in the XZ plane
    // 2. This is always normalized
    //
    // DO NOT expose this field to users as it is easy to mess this up
    start_vertex: Vec3,
    isometry: Isometry3d,
    angle: f32,
    radius: f32,
    color: Color,
    resolution: Option<u32>,
}

impl<Config, Clear> Arc3dBuilder<'_, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    /// Set the number of lines for this arc.
    pub fn resolution(mut self, resolution: u32) -> Self {
        self.resolution.replace(resolution);
        self
    }
}

impl<Config, Clear> Drop for Arc3dBuilder<'_, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    fn drop(&mut self) {
        if !self.gizmos.enabled {
            return;
        }

        let resolution = self
            .resolution
            .unwrap_or_else(|| resolution_from_angle(self.angle));

        let positions = arc_3d_inner(
            self.start_vertex,
            self.isometry,
            self.angle,
            self.radius,
            resolution,
        );
        self.gizmos.linestrip(positions, self.color);
    }
}

fn arc_3d_inner(
    start_vertex: Vec3,
    isometry: Isometry3d,
    angle: f32,
    radius: f32,
    resolution: u32,
) -> impl Iterator<Item = Vec3> {
    // drawing arcs bigger than TAU degrees or smaller than -TAU degrees makes no sense since
    // we won't see the overlap and we would just decrease the level of details since the resolution
    // would be larger
    let angle = angle.clamp(-TAU, TAU);
    (0..=resolution)
        .map(move |frac| frac as f32 / resolution as f32)
        .map(move |percentage| angle * percentage)
        .map(move |frac_angle| Quat::from_axis_angle(Vec3::Y, frac_angle) * start_vertex)
        .map(move |vec3| vec3 * radius)
        .map(move |vec3| isometry * vec3)
}

// helper function for getting a default value for the resolution parameter
fn resolution_from_angle(angle: f32) -> u32 {
    ((angle.abs() / TAU) * DEFAULT_CIRCLE_RESOLUTION as f32).ceil() as u32
}
