use crate::prelude::*;

/// Angular constraints apply an angular correction of a given rotation angle around a given axis.
///
/// The constraint functions are based on equations 11-16 in the paper [Detailed Rigid Body Simulation with Extended Position Based Dynamics](https://matthias-research.github.io/pages/publications/PBDBodies.pdf).
pub trait AngularConstraint: XpbdConstraint<2> {
    /// Applies angular constraints for interactions between two bodies.
    ///
    /// Here in 2D, `axis` is a unit vector with the Z coordinate set to 1 or -1. It controls if the body should rotate counterclockwise or clockwise.
    ///
    /// Returns the angular impulse that is applied proportional to the inverse masses of the bodies.
    #[cfg(feature = "2d")]
    fn apply_angular_correction(
        &self,
        body1: &mut RigidBodyQueryItem,
        body2: &mut RigidBodyQueryItem,
        delta_lagrange: Scalar,
        axis: Vector3,
    ) -> Scalar {
        if delta_lagrange >= -Scalar::EPSILON {
            return 0.0;
        }

        // Compute angular impulse
        // `axis.z` is 1 or -1 and it controls if the body should rotate counterclockwise or clockwise
        let p = -delta_lagrange * axis.z;

        let rot1 = *body1.rot;
        let rot2 = *body2.rot;

        let inv_inertia1 = body1.world_inv_inertia().0;
        let inv_inertia2 = body2.world_inv_inertia().0;

        // Apply rotational updates
        if body1.rb.is_dynamic() {
            *body1.rot += Self::get_delta_rot(rot1, inv_inertia1, p);
        }
        if body2.rb.is_dynamic() {
            *body2.rot -= Self::get_delta_rot(rot2, inv_inertia2, p);
        }

        p
    }

    /// Applies angular constraints for interactions between two bodies.
    ///
    /// Returns the angular impulse that is applied proportional to the inverse masses of the bodies.
    #[cfg(feature = "3d")]
    fn apply_angular_correction(
        &self,
        body1: &mut RigidBodyQueryItem,
        body2: &mut RigidBodyQueryItem,
        delta_lagrange: Scalar,
        axis: Vector,
    ) -> Vector {
        if delta_lagrange >= -Scalar::EPSILON {
            return Vector::ZERO;
        }

        // Compute angular impulse
        let p = -delta_lagrange * axis;

        let rot1 = *body1.rot;
        let rot2 = *body2.rot;

        let inv_inertia1 = body1.world_inv_inertia().0;
        let inv_inertia2 = body2.world_inv_inertia().0;

        // Apply rotational updates
        if body1.rb.is_dynamic() {
            *body1.rot += Self::get_delta_rot(rot1, inv_inertia1, p);
        }
        if body2.rb.is_dynamic() {
            *body2.rot -= Self::get_delta_rot(rot2, inv_inertia2, p);
        }

        p
    }

    #[cfg(feature = "2d")]
    fn compute_generalized_inverse_mass(&self, body: &RigidBodyQueryItem, axis: Vector3) -> Scalar {
        if body.rb.is_dynamic() {
            axis.dot(body.inv_inertia.0 * axis)
        } else {
            // Static and kinematic bodies are a special case, where 0.0 can be thought of as infinite mass.
            0.0
        }
    }

    #[cfg(feature = "3d")]
    fn compute_generalized_inverse_mass(&self, body: &RigidBodyQueryItem, axis: Vector) -> Scalar {
        if body.rb.is_dynamic() {
            axis.dot(body.world_inv_inertia().0 * axis)
        } else {
            // Static and kinematic bodies are a special case, where 0.0 can be thought of as infinite mass.
            0.0
        }
    }

    #[cfg(feature = "2d")]
    fn get_delta_rot(_rot: Rot, inv_inertia: Scalar, p: Scalar) -> Rot {
        // Equation 8/9 but in 2D
        Rot::from_radians(inv_inertia * p)
    }

    #[cfg(feature = "3d")]
    fn get_delta_rot(rot: Rot, inv_inertia: Matrix3, p: Vector) -> Rot {
        // Equation 8/9
        Rot(Quaternion::from_vec4(0.5 * (inv_inertia * p).extend(0.0)) * rot.0)
    }

    /// Computes the torque acting along the constraint using the equation tau = lambda * n / h^2
    fn compute_torque(&self, lagrange: Scalar, axis: Vector3, dt: Scalar) -> Torque {
        // Eq (17)
        #[cfg(feature = "2d")]
        {
            lagrange * axis.z / dt.powi(2)
        }
        #[cfg(feature = "3d")]
        {
            lagrange * axis / dt.powi(2)
        }
    }
}
