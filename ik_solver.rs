struct PoleTarget {
    origin: Vec3,
    tangent: Vec3,
    normal: Vec3,
    angle: f32,
}

pub fn inverse_kinematics_system(
    query: Query<(Entity, &IkConstraint)>,
    parents: Query<&ChildOf>,
    mut transforms: Query<(&mut Transform, &mut GlobalTransform)>,
) {
    for (entity, constraint) in query.iter() {
        if !constraint.enabled {
            continue;
        }

        if let Err(e) = constraint.solve(entity, &parents, &mut transforms) {
            bevy::log::warn!("Failed to solve IK constraint: {e}");
        }
    }
}

/// Inverse kintematics constraint to be added to the tail joint of
/// a chain of bones. The solver will attempt to match the global translation of
/// of this entity to that of the target.
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct IkConstraint {
    /// How many bones are included in the IK constraint.
    pub chain_length: usize,
    /// Maximum number of iterations to solve this constraint.
    pub iterations: usize,
    /// Target entity. The target must have a `Transform` and `GlobalTransform`.
    pub target: Entity,
    /// Target entity for pole rotation.
    /// If this would be a person's arm, this is the direction that the elbow will point.
    pub pole_target: Option<Entity>,
    /// Pole target roll offset.
    /// If a pole target is set, the bone will roll toward the pole target.
    /// This angle is the offset to apply to the roll.
    pub pole_angle: f32,
    /// Whether this constraint is enabled. Disabled constraints will be skipped.
    pub enabled: bool,
}

impl IkConstraint {
    pub fn solve(
        &self,
        entity: Entity,
        parents: &Query<&ChildOf>,
        transforms: &mut Query<(&mut Transform, &mut GlobalTransform)>,
    ) -> Result<(), QueryEntityError> {
        if self.chain_length == 0 {
            return Ok(());
        }

        let mut joints = Vec::with_capacity(self.chain_length + 2);
        joints.push(entity);
        for i in 0..self.chain_length + 1 {
            joints.push(parents.get(joints[i])?.parent());
        }

        let target = transforms.get(self.target)?.1.translation();
        let normal = transforms.get(joints[0])?.0.translation;

        let pole_target = if let Some(pole_target) = self.pole_target {
            let origin = joints[self.chain_length];
            let origin = transforms.get(origin).unwrap().1.translation();
            let pole_target = transforms.get(pole_target)?.1.translation();

            let tangent = (target - origin).normalize();
            let axis = (pole_target - origin).cross(tangent);
            let normal = tangent.cross(axis).normalize();

            Some(PoleTarget {
                origin,
                tangent,
                normal,
                angle: self.pole_angle,
            })
        } else {
            None
        };

        for _ in 0..self.iterations {
            let result = Self::solve_recursive(
                &joints[1..],
                normal,
                target,
                pole_target.as_ref(),
                transforms,
            )?;

            if result.transform_point(normal).distance_squared(target) < 0.001 {
                return Ok(());
            }
        }

        Ok(())
    }

    fn solve_recursive(
        chain: &[Entity],
        normal: Vec3,
        target: Vec3,
        pole_target: Option<&PoleTarget>,
        transforms: &mut Query<(&mut Transform, &mut GlobalTransform)>,
    ) -> Result<GlobalTransform, QueryEntityError> {
        let (current, chain) = chain.split_first().unwrap();

        let (&transform, &gtx) = transforms.get(current)?;

        if chain.is_empty() {
            return Ok(gtx);
        }

        // determine absolute rotation and translation for this bone where the tail touches the target.
        let rotation = if let Some(pt) = pole_target {
            let on_pole =
                pt.origin + (gtx.translation() - pt.origin).project_onto_normalized(pt.tangent);
            let distance = on_pole.distance(gtx.translation());
            let from_position = on_pole + pt.normal * distance;

            let base = Quat::from_rotation_arc(normal.normalize(), Vec3::Z);

            let forward = target - from_position;
            let forward = forward.try_normalize().unwrap_or(pt.tangent);
            let up = forward.cross(pt.normal).normalize();
            let right = up.cross(forward);
            let orientation = Mat3::from_cols(right, up, forward) * Mat3::from_rotation_z(pt.angle);

            (Quat::from_mat3(&orientation) * base).normalize()
        } else {
            let axis = (target - gtx.translation()).normalize();
            Quat::from_rotation_arc(normal.normalize(), axis).normalize()
        };
        let translation = target - rotation.mul_vec3(normal);

        // recurse to target the parent towards the current translation
        let parent = Self::solve_recursive(
            chain,
            transform.translation,
            translation,
            pole_target,
            transforms,
        )?;

        // apply constraints on the way back from recursing
        let (mut transform, mut gtx) = transforms.get_mut(current)?;

        transform.rotation = Quat::from_affine3(&parent.affine()).inverse().normalize() * rotation;
        *gtx = parent.mul_transform(*transform);

        Ok(*gtx)
    }
}

fn pole_rot(pt: PoleTarget) -> Quat {
    let on_pole = pt.origin + (gtx.translation() - pt.origin).project_onto_normalized(pt.tangent);
    let distance = on_pole.distance(gtx.translation());
    let from_position = on_pole + pt.normal * distance;

    let base = Quat::from_rotation_arc(normal.normalize(), Vec3::Z);

    let forward = target - from_position;
    let forward = forward.try_normalize().unwrap_or(pt.tangent);
    let up = forward.cross(pt.normal).normalize();
    let right = up.cross(forward);
    let orientation = Mat3::from_cols(right, up, forward) * Mat3::from_rotation_z(pt.angle);

    (Quat::from_mat3(&orientation) * base).normalize()
}
