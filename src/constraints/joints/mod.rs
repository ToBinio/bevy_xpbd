//! General joint logic and different types of built-in joints.

mod fixed;
mod prismatic;
mod revolute;
mod spherical;

pub use fixed::*;
pub use prismatic::*;
pub use revolute::*;
pub use spherical::*;

use crate::prelude::*;
use bevy::prelude::*;

/// Joints are constraints that attach pairs of bodies and restrict their relative positional and rotational degrees of freedom.
pub trait Joint: Component + PositionConstraint + AngularConstraint {
    /// Creates a new joint with a given compliance (inverse of stiffness).
    fn new_with_compliance(entity1: Entity, entity2: Entity, compliance: Scalar) -> Self;

    /// Sets the attachment point on the first body.
    fn with_local_anchor_1(self, anchor: Vector) -> Self;

    /// Sets the attachment point on the second body.
    fn with_local_anchor_2(self, anchor: Vector) -> Self;

    /// Sets the linear velocity damping caused by the joint.
    fn with_lin_vel_damping(self, damping: Scalar) -> Self;

    /// Sets the angular velocity damping caused by the joint.
    fn with_ang_vel_damping(self, damping: Scalar) -> Self;

    /// Returns the two entities constrained by the joint.
    fn entities(&self) -> [Entity; 2];

    /// Returns the linear velocity damping of the joint.
    fn damping_lin(&self) -> Scalar;

    /// Returns the angular velocity damping of the joint.
    fn damping_ang(&self) -> Scalar;

    /// Applies the positional and angular corrections caused by the joint.
    fn constrain(
        &mut self,
        body1: &mut RigidBodyQueryItem,
        body2: &mut RigidBodyQueryItem,
        sub_dt: Scalar,
    );

    /// Returns the positional correction required to limit the distance between two bodies to be between `min` and `max`.
    #[allow(clippy::too_many_arguments)]
    fn limit_distance(
        &mut self,
        min: Scalar,
        max: Scalar,
        r1: Vector,
        r2: Vector,
        pos1: &Pos,
        pos2: &Pos,
    ) -> Vector {
        let pos_offset = (pos2.0 + r2) - (pos1.0 + r1);
        let distance = pos_offset.length();

        if distance <= Scalar::EPSILON {
            return Vector::ZERO;
        }

        // Equation 25
        if distance < min {
            // Separation distance lower limit
            -pos_offset / distance * (distance - min)
        } else if distance > max {
            // Separation distance upper limit
            -pos_offset / distance * (distance - max)
        } else {
            Vector::ZERO
        }
    }

    /// Returns the positional correction required to limit the distance between two bodies to be between `min` and `max` along a given `axis`.
    #[allow(clippy::too_many_arguments)]
    fn limit_distance_along_axis(
        &mut self,
        min: Scalar,
        max: Scalar,
        axis: Vector,
        r1: Vector,
        r2: Vector,
        pos1: &Pos,
        pos2: &Pos,
    ) -> Vector {
        let pos_offset = (pos2.0 + r2) - (pos1.0 + r1);
        let a = pos_offset.dot(axis);

        // Equation 25
        if a < min {
            // Separation distance lower limit
            -axis * (a - min)
        } else if a > max {
            // Separation distance upper limit
            -axis * (a - max)
        } else {
            Vector::ZERO
        }
    }

    /// Returns the angular correction required to limit thw angle between the axes `n1` and `n2` to be in the interval between `alpha` and `beta` using the common rotation axis `n`.
    fn limit_angle(
        n: Vector3,
        n1: Vector3,
        n2: Vector3,
        alpha: Scalar,
        beta: Scalar,
        max_correction: Scalar,
    ) -> Option<Vector3> {
        let mut phi = n1.cross(n2).dot(n).asin();

        if n1.dot(n2) < 0.0 {
            phi = PI - phi;
        }

        if phi > PI {
            phi -= 2.0 * PI;
        }

        if phi < -PI {
            phi += 2.0 * PI;
        }

        if phi < alpha || phi > beta {
            phi = phi.clamp(alpha, beta);

            let rot = Quaternion::from_axis_angle(n, phi);
            let mut omega = rot.mul_vec3(n1).cross(n2);

            phi = omega.length();

            if phi > max_correction {
                omega *= max_correction / phi;
            }

            return Some(omega);
        }

        None
    }
}

/// A joint limit between `min` and `max`. This can represent things like distance limits or angle limits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JointLimit {
    pub min: Scalar,
    pub max: Scalar,
}

impl JointLimit {
    /// Creates a new `JointLimit`.
    pub fn new(min: Scalar, max: Scalar) -> Self {
        Self { min, max }
    }
}
