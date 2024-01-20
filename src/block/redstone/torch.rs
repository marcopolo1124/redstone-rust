use super::*;

pub fn redstone_torch_delayed_listener(
    mut listeners: &mut EventListener,
    world_map: &mut WorldMap
) {
    ////println!("start of listening");
    let torch_listeners = listeners.redstone_state.clone();
    listeners.redstone_state.clear();

    for ((x, y), (on, signal, signal_type)) in torch_listeners {
        if on {
            set_power(&mut world_map.0, x, y, signal, signal_type, &mut listeners);
        } else {
            set_power_to_0(
                &mut world_map.0,
                x,
                y,
                signal_type,
                signal,
                &mut listeners,
            );
        }
    }
}