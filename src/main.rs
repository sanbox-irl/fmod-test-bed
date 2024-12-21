use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

use fmod_test_bed::{AudioEngine, EventInstance};
use u64_id::U64Id;

#[cfg(target_arch = "wasm32")]
macro_rules! agnostic_print {
    ($($t:tt)*) => (web_sys::console::log_1(&format_args!($($t)*).to_string().into()))
}

#[cfg(target_arch = "x86_64")]
macro_rules! agnostic_print {
    ($($t:tt)*) => (println!("{}", format_args!($($t)*)))
}

// Wasm "loop", it uses requestAnimationFrame from browser window to run in a
// good refresh rate. Changing tabs makes it slower, so audio starts to cut
// for that reason.
#[cfg(target_arch = "wasm32")]
pub fn main() -> Result<(), wasm_bindgen::JsValue> {
    use std::panic;
    use wasm_bindgen::prelude::*;

    // In case some panic occurs at Rust side, this allows it to log into
    // console instead of some vague error.
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let engine = Rc::new(RefCell::new(setup()));

    // :(
    let closure: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let self_closure = closure.clone();

    *closure.borrow_mut() = Some(Closure::new(move || {
        if !tick(engine.borrow_mut()) {
            return;
        }
        web_sys::window()
            .unwrap()
            .request_animation_frame(
                self_closure
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref(),
            )
            .unwrap();
    }));

    web_sys::window()
        .unwrap()
        .request_animation_frame(closure.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();

    Ok(())
}

struct Game {
    tick_count: u32,
    engine: AudioEngine,
    current: Option<EventInstance>,
}

// Native loop. Uses Rc<RefCell>> because tick has to, because of the closure
// above in wasm main.
#[cfg(target_arch = "x86_64")]
pub fn main() {
    let engine = Rc::new(RefCell::new(setup()));

    while tick(engine.borrow_mut()) {
        std::thread::sleep(std::time::Duration::from_secs_f64(1. / 144.));
    }
}

// Shared setup code
fn setup() -> Game {
    agnostic_print!("- AudioEngine::new()");
    let mut engine = AudioEngine::new(true).unwrap();

    let master_strings_bank = include_bytes!("../resources/Master.strings.bank");
    let master_bank = include_bytes!("../resources/Master.bank");
    let music_bank = include_bytes!("../resources/Music.bank");

    agnostic_print!("- AudioEngine::load_bank_files_from_memory()");
    engine
        .load_bank_files_from_memory(
            U64Id::new(),
            &[master_strings_bank, master_bank, music_bank],
        )
        .unwrap();

    Game {
        tick_count: 0,
        engine,
        current: None,
    }
}
// Shared tick code
fn tick(mut game: RefMut<Game>) -> bool {
    let mut check_tick = 0;
    let mut next_check = || {
        check_tick += 144;
        check_tick
    };

    if game.tick_count == next_check() {
        agnostic_print!(
            "- AudioEngine::event_names() -> {:?}",
            game.engine.event_names()
        );
    }
    if game.tick_count == next_check() {
        agnostic_print!("- AudioEngine::play_event(\"event:/Music/Level 02\")");
        game.current = game.engine.play_event("event:/Music/Level 02").ok();
    }
    if game.tick_count == next_check() {
        agnostic_print!("- AudioEngine::set_global_mute(true)");
        game.engine.set_global_mute(true);
    }
    if game.tick_count == next_check() {
        agnostic_print!("- AudioEngine::set_global_mute(false)");
        game.engine.set_global_mute(false);
    }
    if game.tick_count == next_check() {
        agnostic_print!(
            "- AudioEngine::is_event_playing(\"event:/Music/Level 02\") -> {:?}",
            game.engine.is_event_playing("event:/Music/Level 02"),
        );
    }
    if game.tick_count == next_check() {
        agnostic_print!(
            "- AudioEngine::event_instance_count(\"event:/Music/Level 02\") -> {:?}",
            game.engine.event_instance_count("event:/Music/Level 02"),
        );
    }
    if game.tick_count == next_check() {
        agnostic_print!("- AudioEngine::set_global_parameter(\"Area\", 70.0) !! This one doesn't work with example because
            I don't know how to use FMOD :(");
        game.engine.set_global_parameter("Area", 70.0).ok();
    }
    if game.tick_count == next_check() {
        agnostic_print!(
            "- AudioEngine::set_listener_position_velocity((15.0, 15.0).into(), (5.0, 5.0).into())"
        );
        game.engine
            .set_listener_position_velocity((15.0, 15.0).into(), (5.0, 5.0).into())
            .ok();
    }
    if game.tick_count == next_check() {
        agnostic_print!("---");
        agnostic_print!(
            "- AudioEngine::listener_position() -> {:?}",
            game.engine.listener_position()
        );
        agnostic_print!(
            "- AudioEngine::listener_position() -> {:?}",
            game.engine.listener_velocity()
        );
    }
    if game.tick_count == next_check() {
        agnostic_print!("- EventInstance::set_pitch(1.5)");
        game.current.as_ref().unwrap().set_pitch(1.5).unwrap();
    }
    if game.tick_count == next_check() {
        agnostic_print!("---");
        agnostic_print!(
            "- EventInstance::pitch() -> {:?}",
            game.current.as_ref().unwrap().pitch(),
        );
        agnostic_print!(
            "- EventInstance::final_pitch() -> {:?}",
            game.current.as_ref().unwrap().final_pitch()
        );
    }
    if game.tick_count == next_check() {
        agnostic_print!("- EventInstance::set_pitch(1.0)");
        game.current.as_ref().unwrap().set_pitch(1.0).unwrap();
    }
    if game.tick_count == next_check() {
        agnostic_print!("- EventInstance::set_volume(0.25)");
        game.current.as_ref().unwrap().set_volume(0.25).unwrap();
    }
    if game.tick_count == next_check() {
        agnostic_print!("---");
        agnostic_print!(
            "- EventInstance::volume() -> {:?}",
            game.current.as_ref().unwrap().volume(),
        );
        agnostic_print!(
            "- EventInstance::final_volume() -> {:?}",
            game.current.as_ref().unwrap().final_volume(),
        );
    }
    if game.tick_count == next_check() {
        agnostic_print!("- EventInstance::set_volume(1.0)");
        game.current.as_ref().unwrap().set_volume(1.0).unwrap();
    }
    if game.tick_count == next_check() {
        agnostic_print!("- EventInstance::pause()");
        game.current.as_ref().unwrap().pause().unwrap();
    }
    if game.tick_count == next_check() {
        agnostic_print!(
            "- EventInstance::is_paused() -> {:?}",
            game.current.as_ref().unwrap().is_paused(),
        );
    }
    if game.tick_count == next_check() {
        agnostic_print!("- EventInstance::unpause()");
        game.current.as_ref().unwrap().unpause().unwrap();
    }
    if game.tick_count == next_check() {
        agnostic_print!("- EventInstance::set_timeline_position(5000)");
        game.current
            .as_ref()
            .unwrap()
            .set_timeline_position(5000)
            .unwrap();
    }
    if game.tick_count == next_check() {
        agnostic_print!(
            "- EventInstance::timeline_position() -> {:?}",
            game.current.as_ref().unwrap().timeline_position(),
        );
    }
    if game.tick_count == next_check() {
        agnostic_print!(
            "- EventInstance::is_virtual() -> {:?}",
            game.current.as_ref().unwrap().is_virtual(),
        );
    }
    if game.tick_count == next_check() {
        agnostic_print!("- EventInstance::stop()");
        game.current.as_ref().unwrap().stop().unwrap();
    }
    if game.tick_count == next_check() {
        agnostic_print!(
            "- EventInstance::playback_state() -> {:?}",
            game.current.as_ref().unwrap().playback_state(),
        );
    }
    if game.tick_count == next_check() {
        agnostic_print!("- EventInstance::start()");
        game.current.as_ref().unwrap().start().unwrap();
    }
    if game.tick_count == next_check() {
        agnostic_print!("- EventInstance::set_property(ScheduleDelay, 1.0)");
        game.current
            .as_ref()
            .unwrap()
            .set_property(fmod_test_bed::EventProperty::ScheduleDelay, 1.0)
            .unwrap();
    }
    if game.tick_count == next_check() {
        agnostic_print!(
            "- EventInstance::property(ScheduleDelay) -> {:?}",
            game.current
                .as_ref()
                .unwrap()
                .property(fmod_test_bed::EventProperty::ScheduleDelay),
        );
    }
    if game.tick_count == next_check() {
        agnostic_print!("- EventInstance::set_parameter_by_name(\"Area\", 70.0, false)");
        game.current
            .as_ref()
            .unwrap()
            .set_parameter_by_name("Area", 70.0, false)
            .unwrap();
    }
    if game.tick_count == next_check() {
        agnostic_print!("---");
        agnostic_print!(
            "- EventInstance::get_parameter_by_name(\"Area\") -> {:?}",
            game.current.as_ref().unwrap().get_parameter_by_name("Area"),
        );
        agnostic_print!(
            "- EventInstance::get_final_parameter_by_name(\"Area\") -> {:?}",
            game.current
                .as_ref()
                .unwrap()
                .get_final_parameter_by_name("Area"),
        );
    }
    if game.tick_count == next_check() {
        agnostic_print!(
            "- EventInstance::set_position_velocity((2.0, 2.0).into(), (4.0, 4.0).into())"
        );
        game.current
            .as_ref()
            .unwrap()
            .set_position_velocity((2.0, 2.0).into(), (4.0, 4.0).into())
            .unwrap();
    }
    if game.tick_count == next_check() {
        agnostic_print!(
            "- EventInstance::get_position_velocity() -> {:?}",
            game.current.as_ref().unwrap().get_position_velocity()
        );
    }
    if game.tick_count == next_check() {
        agnostic_print!("- EventInstance::mark_for_release()");
        game.current.as_ref().unwrap().mark_for_release().unwrap();
    }
    if game.tick_count == next_check() {
        agnostic_print!("- EventInstance::stop_immediately()");
        game.current.as_ref().unwrap().stop_immediately().unwrap();
    }
    if game.tick_count == next_check() {
        agnostic_print!("- EventInstance::unload_banks()");
        game.engine.unload_banks();

        return false;
    }

    game.engine.update().unwrap();
    game.tick_count += 1;

    true
}
