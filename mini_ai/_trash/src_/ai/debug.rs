/*
pub fn action_insert_default_debug<T: Component + Default>(
    trigger: Trigger<ActionInit, T>,
    mut commands: Commands,
    components: &Components,
) {
    let name = components.get_name(trigger.event().0).unwrap_or_default();
    debug!("insert default {name}");
    commands.entity(trigger.target()).insert(T::default());
}

pub fn action_remove_debug<T: Component>(
    trigger: Trigger<ActionResult, T>,
    mut commands: Commands,
    components: &Components,
) {
    let (marker, id) = match *trigger.event() {
        ActionResult::Successed(id) => ("successed", id),
        ActionResult::Cancelled(id) => ("cancelled", id),
    };
    let entity = trigger.target();
    let name = components.get_name(id).unwrap_or_default();
    debug!("remove {marker} {name} from {entity}");
    commands.entity(entity).remove::<(T, CurrentAction)>();
}
*/
