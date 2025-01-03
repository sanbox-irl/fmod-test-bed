use glam::Vec2;
use u64_id::U64Id;

#[cfg(target_arch = "wasm32")]
pub mod wasmfmod;

// This is the trick to change between libfmod and wasmfmod just with flags
pub mod fmod {
    #[cfg(target_arch = "wasm32")]
    pub use crate::wasmfmod::*;
    #[cfg(not(target_arch = "wasm32"))]
    pub use libfmod::*;
}

type AnyResult<T = ()> = color_eyre::Result<T>;

#[derive(Debug)]
pub struct AudioEngine {
    handle: fmod::Studio,
    event_names: Vec<String>,
    asset_id: Option<U64Id>,
    listener_position: Vec2,
    listener_velocity: Vec2,
}

impl AudioEngine {
    /// Creates a new AudioEngine, initializing FMOD.
    pub fn new(live_update: bool) -> AnyResult<Self> {
        let studio = fmod::Studio::create()?;

        let studio_flags = if live_update {
            fmod::StudioInit::NORMAL | fmod::StudioInit::LIVEUPDATE
        } else {
            fmod::StudioInit::NORMAL
        };

        studio
            .initialize(1024, studio_flags, fmod::Init::RIGHTHANDED_3D, None)
            .expect("Failed to initialize FMOD studio");

        Ok(Self {
            handle: studio,
            event_names: vec![],
            asset_id: None,
            listener_position: Vec2::ZERO,
            listener_velocity: Vec2::ZERO,
        })
    }

    /// Loads bank files from memory directly. To get names our correctly in the event list,
    /// make sure to load the .strings file first.
    pub fn load_bank_files_from_memory(&mut self, asset_id: U64Id, buffers: &[&[u8]]) -> AnyResult {
        for buffer in buffers {
            let bank = self
                .handle
                .load_bank_memory(buffer, fmod::LoadBank::NORMAL)?;

            for maybe_name in bank
                .get_event_list(bank.get_event_count()?)?
                .into_iter()
                .flat_map(|v| v.get_path())
            {
                self.event_names.push(maybe_name);
            }
        }

        self.asset_id = Some(asset_id);

        Ok(())
    }

    /// Unloads the banks from memory, if there are any.
    pub fn unload_banks(&mut self) {
        self.handle.unload_all().expect("failed to unload all");
    }

    /// Gets all the events loaded in the banks.
    pub fn event_names(&self) -> &[String] {
        &self.event_names
    }

    /// Returns the asset id that we loaded all our audio from.
    pub fn asset_id(&self) -> Option<U64Id> {
        self.asset_id
    }

    /// Creates a given event instance.
    ///
    /// Note that this will *not* actually play the given EventInstance at all.
    /// You'll need to run [`EventInstance::start`](fmod::EventInstance::start),
    /// and should almost certainly also run [`EventInstance::release`](fmod::EventInstance::release).
    ///
    /// You can provide an `&str`, but you are *highly* encouraged to make your own Enum which uses `AsRef` to convert
    /// between the types required.
    pub fn create_event_instance(
        &self,
        event_name: &(impl AsRef<str> + ?Sized),
    ) -> AnyResult<EventInstance> {
        let event_name = self.event_name_as_ref(event_name);
        let event_descriptor = self.handle.get_event(event_name)?;

        Ok(EventInstance(event_descriptor.create_instance()?))
    }

    /// Plays a given event by name. If that event does not exist, an error will be returned.
    ///
    /// ## Starting and Releasing
    ///
    /// This runs [`EventInstance::start`] and [`EventInstance::mark_for_release`] immediately.
    /// If you want to avoid running those, use [`AudioEngine::create_event_instance`].
    ///
    /// ## Event Names
    ///
    /// You can provide an `&str`, but you are *highly* encouraged to make your own Enum which uses `AsRef` to convert
    /// between the types required.
    ///
    /// Event names in FMOD always begin with `event:/`. In `debug`, we will check and panic if any event name provided
    /// does not begin with this header.
    pub fn play_event(&self, event_name: &(impl AsRef<str> + ?Sized)) -> AnyResult<EventInstance> {
        let event = self.create_event_instance(event_name)?;

        event.start()?;
        event.mark_for_release()?;

        Ok(event)
    }

    /// Plays a given event by name with position data. If that event does not exist, an error will be returned.
    ///
    /// ## Starting and Releasing
    ///
    /// This runs [`EventInstance::start`] and [`EventInstance::mark_for_release`] immediately.
    /// If you want to avoid running those, use [`AudioEngine::create_event_instance`].
    ///
    /// ## Event Names
    ///
    /// You can provide an `&str`, but you are *highly* encouraged to make your own Enum which uses `AsRef` to convert
    /// between the types required.
    ///
    /// Event names in FMOD always begin with `event:/`. In `debug`, we will check and panic if any event name provided
    /// does not begin with this header.
    pub fn play_event_with_position(
        &self,
        event_name: &(impl AsRef<str> + ?Sized),
        position: Vec2,
    ) -> AnyResult<EventInstance> {
        self.play_event_with_position_velocity(event_name, position, Vec2::ZERO)
    }

    /// Plays a given event by name with position and velocity data. If that event does not exist, an error will be returned.
    ///
    /// ## Starting and Releasing
    ///
    /// This runs [`EventInstance::start`] and [`EventInstance::mark_for_release`] immediately.
    /// If you want to avoid running those, use [`AudioEngine::create_event_instance`].
    ///
    /// ## Event Names
    ///
    /// You can provide an `&str`, but you are *highly* encouraged to make your own Enum which uses `AsRef` to convert
    /// between the types required.
    ///
    /// Event names in FMOD always begin with `event:/`. In `debug`, we will check and panic if any event name provided
    /// does not begin with this header.
    pub fn play_event_with_position_velocity(
        &self,
        event_name: &(impl AsRef<str> + ?Sized),
        position: Vec2,
        velocity: Vec2,
    ) -> AnyResult<EventInstance> {
        let event = self.create_event_instance(event_name)?;

        event.set_position_velocity(position, velocity)?;
        event.start()?;
        event.mark_for_release()?;

        Ok(event)
    }

    /// Sets the master bus to mute. All buses eventually route through the master bus,
    /// so this will mute the enter game.
    pub fn set_global_mute(&self, mute: bool) {
        self.handle
            .get_bus("bus:/")
            .unwrap()
            .set_mute(mute)
            .unwrap();
    }

    /// Sets a global parameter. Most parameters are instanced, and for those, you'll need
    /// to set them *per instance* in [`EventInstance::set_parameter_by_name`]
    pub fn set_global_parameter(&self, parameter_name: &str, value: f32) -> AnyResult {
        self.handle
            .set_parameter_by_name(parameter_name, value, true)?;

        Ok(())
    }

    /// Checks if any event with the given name is playing at all.
    ///
    /// You can provide an `&str`, but you are *highly* encouraged to make your own Enum which uses `AsRef` to convert
    /// between the types required.
    pub fn is_event_playing(&self, event_name: &(impl AsRef<str> + ?Sized)) -> AnyResult<bool> {
        Ok(self.event_instance_count(event_name)? > 0)
    }

    /// Checks how many times a given event is playing.
    ///
    /// You can provide an `&str`, but you are *highly* encouraged to make your own Enum which uses `AsRef` to convert
    /// between the types required.
    pub fn event_instance_count(&self, event_name: &(impl AsRef<str> + ?Sized)) -> AnyResult<u32> {
        let event_descriptor = self.handle.get_event(self.event_name_as_ref(event_name))?;

        Ok(event_descriptor.get_instance_count()? as u32)
    }

    /// Sets the position of the listener in the spatializer.
    ///
    /// See [`AudioEngine::set_listener_velocity`] and [`AudioEngine::set_listener_position_velocity`]
    /// to set the velocity of the listener if that matters for your application. We will continue to use the velocity
    /// last assigned to this function.
    pub fn set_listener_position(&mut self, position: Vec2) -> AnyResult {
        self.set_listener_position_velocity(position, self.listener_velocity)
    }

    /// Sets the velocity of the listener in the spatializer. Some spatializers have
    /// the doppler effect enabled, so you'll want to set the velocity here too.
    ///
    /// We will use the last let listener position (set either with [`AudioEngine::set_listener_position`]
    /// or with [`AudioEngine::set_listener_position_velocity`]) as the listener position.
    pub fn set_listener_velocity(&mut self, velocity: Vec2) -> AnyResult {
        self.set_listener_position_velocity(self.listener_position, velocity)
    }

    /// Sets the position and velocity of the listener in the spatializer. This is provided to reduce
    /// the number of FFI calls we do at once, if you're going to set both all the time.
    ///
    /// Note: the internally tracked `position` and `velocity` will only be updated when this function
    /// returns `Ok`.
    pub fn set_listener_position_velocity(&mut self, position: Vec2, velocity: Vec2) -> AnyResult {
        self.handle.set_listener_attributes(
            0,
            fmod::Attributes3d {
                position: fmod::Vector::new(position.x, position.y, 0.0),
                velocity: fmod::Vector::new(velocity.x, velocity.y, 0.0),
                forward: fmod::Vector::new(0.0, 1.0, 0.0),
                up: fmod::Vector::new(0.0, 0.0, 1.0),
            },
            None,
        )?;

        // update our internals
        self.listener_position = position;
        self.listener_velocity = velocity;

        Ok(())
    }

    /// Gets the internally held listener position. This is the value that was last set using
    /// [`AudioEngine::set_listener_position`] or [`AudioEngine::set_listener_position_velocity`].
    ///
    /// Defaults to [`Vec2::ZERO`].
    pub fn listener_position(&self) -> Vec2 {
        self.listener_position
    }

    /// Gets the internally held listener velocity. This is the value that was last set using
    /// [`AudioEngine::set_listener_velocity`] or [`AudioEngine::set_listener_position_velocity`].
    ///
    /// Defaults to [`Vec2::ZERO`].
    pub fn listener_velocity(&self) -> Vec2 {
        self.listener_velocity
    }

    /// This must be called once per frame. At this point, all commands are *actually* submitted and
    /// callbacks occur. Basically, the good stuff happens here.
    ///
    /// This gets called in [mwe::main_loop] automatically.
    pub fn update(&self) -> AnyResult {
        if self.asset_id.is_none() {
            return Ok(());
        }

        self.handle.update()?;

        Ok(())
    }

    /// Converts the given event name back into a string, doing our debug check
    fn event_name_as_ref<'a>(&self, event_name: &'a (impl AsRef<str> + ?Sized)) -> &'a str {
        let event_name = event_name.as_ref();

        debug_assert!(
            event_name.starts_with("event:/"),
            "all fmod events begin with `event:/`, this event is only {}",
            event_name
        );

        event_name
    }
}

/// An EventInstance is a *particular* event being fired, which can be configured
/// with various effects and parameters.
///
/// We have not bound everything that FMod offers, so to get to the underlying functions,
/// you can run [`EventInstance::inner`].
#[derive(Debug)]
pub struct EventInstance(fmod::EventInstance);

impl EventInstance {
    /// Gives access to the inner [`fmod::EventInstance`].
    /// This is a kind of get-out-of-jail-free card, since we haven't fully
    /// bound the entire FMOD API ourselves yet, so you might need something here.
    pub fn inner(&self) -> &fmod::EventInstance {
        &self.0
    }

    /// Actually starts playing the audio. If the instance was already playing, this will restart playback.
    pub fn start(&self) -> AnyResult {
        Ok(self.0.start()?)
    }

    /// Marks the event instance for release.
    ///
    /// Event instances marked for release are destroyed when they are in the stopped
    /// state [`PlaybackState::Stopped]`.
    pub fn mark_for_release(&self) -> AnyResult {
        Ok(self.0.release()?)
    }

    /// Sets the pitch of the audio.
    /// The pitch multiplier is used to modulate the event instance's pitch.
    /// The pitch multiplier can be set to any value greater than or equal to zero but
    /// the final combined pitch is clamped to the range [0, 100] before being applied.
    ///
    /// The default pitch is `1.0`.
    ///
    /// ## Panics
    ///
    /// In `debug`, we panic if `pitch < 0.0`.
    pub fn set_pitch(&self, pitch: f32) -> AnyResult {
        debug_assert!(pitch >= 0.0);

        self.0.set_pitch(pitch)?;

        Ok(())
    }

    /// Retrieves the pitch multiplier. See [`EventInstance::final_pitch`] to get the final
    /// pitch after any modulation or changes.
    pub fn pitch(&self) -> AnyResult<f32> {
        Ok(self.0.get_pitch()?.0)
    }

    /// Retrieves the final pitch multiplier. The final combined value returned combines the pitch set
    /// using [`EventInstance::set_pitch`] with the result of any automation or modulation.
    /// The final combined pitch is calculated asynchronously once a frame.
    ///
    /// See [`EventInstance::pitch`] to get the pitch on this event alone.
    pub fn final_pitch(&self) -> AnyResult<f32> {
        Ok(self.0.get_pitch()?.1)
    }

    /// Sets the value of a built-in property.
    pub fn set_property(&self, property: EventProperty, value: f32) -> AnyResult {
        self.0.set_property(property.into(), value)?;
        Ok(())
    }
    /// Gets the value of a built-in property.
    pub fn property(&self, property: EventProperty) -> AnyResult<f32> {
        Ok(self.0.get_property(property.into())?)
    }

    /// Sets the timeline cursor position.
    ///
    /// The units are in *milliseconds* and has the maximum size of `i32::MAX` (not `u32`).
    pub fn set_timeline_position(&self, timeline_position: u32) -> AnyResult {
        self.0.set_timeline_position(timeline_position as i32)?;

        Ok(())
    }

    /// Gets the timeline cursor position.
    pub fn timeline_position(&self) -> AnyResult<u32> {
        Ok(self.0.get_timeline_position()? as u32)
    }

    /// Sets the volume level.
    /// This volume is applied as a scaling factor for the event volume.
    /// It does not override the volume level set in FMOD Studio, nor any internal volume automation or modulation.
    pub fn set_volume(&self, volume: f32) -> AnyResult {
        self.0.set_volume(volume)?;

        Ok(())
    }

    /// Retrieves the volume level. See [`EventInstance::final_volume`] to get the final
    /// volume after any modulation or changes.
    pub fn volume(&self) -> AnyResult<f32> {
        Ok(self.0.get_volume()?.0)
    }

    /// Retrieves the final volume multiplier. The final combined value returned combines the volume set
    /// using [`EventInstance::set_volume`] with the result of any automation or modulation.
    /// The final combined volume is calculated asynchronously once a frame.
    ///
    /// See [`EventInstance::volume`] to get the volume on this event alone.
    pub fn final_volume(&self) -> AnyResult<f32> {
        Ok(self.0.get_volume()?.1)
    }

    /// Retrieves the virtualization state.
    ///
    /// This function checks whether an event instance has been virtualized due to the polyphony limit
    /// being exceeded.
    pub fn is_virtual(&self) -> AnyResult<bool> {
        Ok(self.0.is_virtual()?)
    }

    /// Sets the position and velocity on this event instance.
    pub fn set_position_velocity(&self, position: Vec2, velocity: Vec2) -> AnyResult {
        self.0.set_3d_attributes(fmod::Attributes3d {
            position: fmod::Vector::new(position.x, position.y, 0.0),
            velocity: fmod::Vector::new(velocity.x, velocity.y, 0.0),
            forward: fmod::Vector::new(0.0, 1.0, 0.0),
            up: fmod::Vector::new(0.0, 0.0, 1.0),
        })?;

        Ok(())
    }

    /// Gets the position and velocity on this event instance.
    pub fn get_position_velocity(&self) -> AnyResult<AudioPositionVelocity> {
        let atty = self.0.get_3d_attributes()?;

        Ok(AudioPositionVelocity {
            position: Vec2::new(atty.position.x, atty.position.y),
            velocity: Vec2::new(atty.velocity.x, atty.velocity.y),
        })
    }

    /// Sets a given parameter by case-insensitive name.
    ///
    /// `ignore_seek_speed` specifies whether to ignore the parameter's seek speed and set the value immediately
    /// The `value` will be set instantly regardless of `ignore_seek_speed` when the Event playback is
    /// [`PlaybackState::Stopped`].
    ///
    /// If the specified parameter is an automatic parameter then an error is returned. If the event has no parameter
    /// matching name then an error is returned.
    pub fn set_parameter_by_name(
        &self,
        parameter: &str,
        value: f32,
        ignore_seek_speed: bool,
    ) -> AnyResult {
        self.0
            .set_parameter_by_name(parameter, value, ignore_seek_speed)?;

        Ok(())
    }

    /// Retrieves a parameter value by case-insensitive name. See [`EventInstance::get_final_parameter_by_name`] as well.
    ///
    /// Automatic parameters always return value as 0 since they can never have their value set from the public API.
    pub fn get_parameter_by_name(&self, parameter: &str) -> AnyResult<f32> {
        Ok(self.0.get_parameter_by_name(parameter)?.0)
    }

    /// Retrieves a parameter's final value by case-insensitive name. This is the final value of the parameter after
    /// applying adjustments due to automation, modulation, seek speed, and parameter velocity to value. This is
    /// calculated asynchronously once a frame.
    ///
    /// Automatic parameters always return value as 0 since they can never have their value set from the public API.
    ///
    /// See [`EventInstance::get_parameter_by_name`] for the value without other adjustments.
    pub fn get_final_parameter_by_name(&self, parameter: &str) -> AnyResult<f32> {
        Ok(self.0.get_parameter_by_name(parameter)?.1)
    }

    /// Stops playback with a fadeout, allowing AHDSR modulators to complete their release, and DSP effect tails to play out.
    /// This is the preferred way to stop audio.
    ///
    /// If you need to stop immediately, use [`EventInstance::stop_immediately`].
    pub fn stop(&self) -> AnyResult {
        self.0.stop(fmod::StopMode::AllowFadeout)?;

        Ok(())
    }

    /// Stops playback immediately. If you need to stop with a fadeout, use [`EventInstance::stop`].
    pub fn stop_immediately(&self) -> AnyResult {
        self.0.stop(fmod::StopMode::Immediate)?;

        Ok(())
    }

    /// Pauses the given event. If the event is already paused, this doesn't do anything.
    pub fn pause(&self) -> AnyResult {
        self.0.set_paused(true)?;
        Ok(())
    }

    /// Unpauses the given event. If the event isn't paused, this doesn't do anything.
    pub fn unpause(&self) -> AnyResult {
        self.0.set_paused(false)?;
        Ok(())
    }

    /// Returns the pause state of the event. Note that this is different from [`EventInstance::playback_state`].
    pub fn is_paused(&self) -> AnyResult<bool> {
        Ok(self.0.get_paused()?)
    }

    /// You can poll this function to track the playback state of an event instance.
    ///
    /// If the instance is invalid, then the state will be set to [`PlaybackState::Stopped`].
    ///
    /// Note that the playback state can be [`PlaybackState::Playing`] while [`EventInstance::is_paused`] also
    /// return true! In a sense, this is mostly "lifetime" state, rather than just a playback state.
    pub fn playback_state(&self) -> AnyResult<PlaybackState> {
        let state = self.0.get_playback_state()?;

        Ok(state.into())
    }
}

/// Playback state of various objects.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum PlaybackState {
    /// Currently playing, though may be paused.
    Playing,
    /// The timeline cursor is paused on a sustain point.
    Sustaining,
    /// Stopped.
    Stopped,
    /// Preparing to Start.
    Starting,
    /// Preparing to Stop.
    Stopping,
}

impl From<fmod::PlaybackState> for PlaybackState {
    fn from(value: fmod::PlaybackState) -> Self {
        match value {
            fmod::PlaybackState::Playing => PlaybackState::Playing,
            fmod::PlaybackState::Sustaining => PlaybackState::Sustaining,
            fmod::PlaybackState::Stopped => PlaybackState::Stopped,
            fmod::PlaybackState::Starting => PlaybackState::Starting,
            fmod::PlaybackState::Stopping => PlaybackState::Stopping,
        }
    }
}

/// The name of an event property in FMOD.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventProperty {
    /// Priority to set on Core channels created by this event instance, or -1 for default.
    /// Range: [-1, 256], default: -1
    ChannelPriority,
    /// Schedule delay in DSP clocks, or -1 for default.
    /// Range: [-1, inf], default: -1
    ScheduleDelay,
    /// Schedule look-ahead on the timeline in DSP clocks, or -1 for default.
    /// Range: [-1, inf], default: -1
    ScheduleLookahead,
    /// Override the event's 3D minimum distance, or -1 for default.
    /// Range: [-1, inf], default: -1
    MinimumDistance,
    /// Override the event's 3D maximum distance, or -1 for default.
    /// Range: [-1, inf], default: -1
    MaximumDistance,
    /// Override the event's cooldown, or -1 for default.
    /// Range: [-1, inf], default: -1
    Cooldown,
}

impl From<EventProperty> for fmod::EventProperty {
    fn from(value: EventProperty) -> Self {
        match value {
            EventProperty::ChannelPriority => fmod::EventProperty::ChannelPriority,
            EventProperty::ScheduleDelay => fmod::EventProperty::ScheduleDelay,
            EventProperty::ScheduleLookahead => fmod::EventProperty::ScheduleLookahead,
            EventProperty::MinimumDistance => fmod::EventProperty::MinimumDistance,
            EventProperty::MaximumDistance => fmod::EventProperty::MaximumDistance,
            EventProperty::Cooldown => fmod::EventProperty::Cooldown,
        }
    }
}

/// The position and velocity set on various FMOD objects.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct AudioPositionVelocity {
    pub position: Vec2,
    pub velocity: Vec2,
}
