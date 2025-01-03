//! This is libfmod replacement for WASM with limited functionality, where it
//! is almost 1:1. Anything that includes opaque JsValue is not Copy, so that's
//! one difference between libfmod and this. JsValue is actually a 32bit number
//! , so this can be solved by transmuting I read somewhere. But, I kept it as
//! like this for now. If lack of Copy is too much of an issue, it is a quick
//! change, at least feels like one.

use std::{
    ffi::{c_void, IntoStringError, NulError},
    fmt::{Display, Formatter},
};

use wasm_bindgen::prelude::*;

use bitflags::bitflags;

macro_rules! err_fmod {
    ($ function : expr , $ code : expr) => {
        Error::Fmod {
            function: $function.to_string(),
            code: $code as i32,
            message: "".to_string(),
        }
    };
}

// Studio wrapper and binding
#[derive(Debug, Clone)]
pub struct Studio {
    opaque: JsValue,
}

impl Studio {
    pub fn create() -> Result<Self, Error> {
        let result = Studio_System_Create();
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(Self { opaque: result.1 }),
            err => Err(err_fmod!("Studio_System_Create", err)),
        }
    }
    pub fn initialize(
        &self,
        max_channels: i32,
        studio_flags: StudioInit,
        flags: Init,
        // I don't know whether extra_driver_data works or not and I don't
        // know how to test it. But my gut feeling says this probably won't
        // work. This one probably needs to be created on Emscripten side.
        extra_driver_data: Option<*mut c_void>,
    ) -> Result<(), Error> {
        let result = Studio_System_Initialize(
            &self.opaque,
            max_channels,
            studio_flags.bits(),
            flags.bits(),
            extra_driver_data,
        );
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(()),
            err => Err(err_fmod!("Studio_System_Initialize", err)),
        }
    }

    pub fn load_bank_memory(&self, buffer: &[u8], flags: LoadBank) -> Result<Bank, Error> {
        let result = Studio_System_LoadBankMemory(&self.opaque, buffer, flags.bits());
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(Bank { opaque: result.1 }),
            err => Err(err_fmod!("Studio_System_LoadBankMemory", err)),
        }
    }
    pub fn unload_all(&self) -> Result<(), Error> {
        let result = Studio_System_UnloadAll(&self.opaque);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(()),
            err => Err(err_fmod!("Studio_System_UnloadAll", err)),
        }
    }
    pub fn get_event(&self, path_or_id: &str) -> Result<EventDescription, Error> {
        let result = Studio_System_GetEvent(&self.opaque, path_or_id);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(EventDescription { opaque: result.1 }),
            err => Err(err_fmod!("Studio_System_GetEvent", err)),
        }
    }
    pub fn get_bus(&self, path_or_id: &str) -> Result<Bus, Error> {
        let result = Studio_System_GetBus(&self.opaque, path_or_id);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(Bus { opaque: result.1 }),
            err => Err(err_fmod!("Studio_System_GetBus", err)),
        }
    }
    pub fn set_parameter_by_name(
        &self,
        name: &str,
        value: f32,
        ignore_seek_speed: bool,
    ) -> Result<(), Error> {
        let result = Studio_System_SetParameterByName(&self.opaque, name, value, ignore_seek_speed);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(()),
            err => Err(err_fmod!("Studio_System_SetParameterByName", err)),
        }
    }
    pub fn set_listener_attributes(
        &self,
        index: i32,
        attributes: Attributes3d,
        attenuation_position: Option<Vector>,
    ) -> Result<(), Error> {
        let result = Studio_System_SetListenerAttributes(
            &self.opaque,
            index,
            Attributes3d::from(attributes),
            attenuation_position.map(Vector::from),
        );
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(()),
            err => Err(err_fmod!("Studio_System_Update", err)),
        }
    }
    pub fn update(&self) -> Result<(), Error> {
        let result = Studio_System_Update(&self.opaque);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(()),
            err => Err(err_fmod!("Studio_System_Update", err)),
        }
    }
}
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    fn Studio_System_Create() -> JsValueJSResult;
    #[wasm_bindgen]
    fn Studio_System_Initialize(
        studio: &JsValue,
        max_channels: i32,
        studio_flags: u32,
        flags: u32,
        extra_driver_data: Option<*mut c_void>,
    ) -> JSResult;
    #[wasm_bindgen]
    fn Studio_System_LoadBankMemory(studio: &JsValue, buffer: &[u8], flags: u32)
        -> JsValueJSResult;
    #[wasm_bindgen]
    fn Studio_System_UnloadAll(studio: &JsValue) -> JSResult;
    #[wasm_bindgen]
    fn Studio_System_GetEvent(studio: &JsValue, path: &str) -> JsValueJSResult;
    #[wasm_bindgen]
    fn Studio_System_GetBus(studio: &JsValue, path: &str) -> JsValueJSResult;
    #[wasm_bindgen]
    fn Studio_System_SetParameterByName(
        studio: &JsValue,
        name: &str,
        value: f32,
        ignore_seek_speed: bool,
    ) -> JSResult;
    #[wasm_bindgen]
    fn Studio_System_SetListenerAttributes(
        studio: &JsValue,
        index: i32,
        attributes: Attributes3d,
        attenuation_position: Option<Vector>,
    ) -> JSResult;
    #[wasm_bindgen]
    fn Studio_System_Update(studio: &JsValue) -> JSResult;
}

// Bank wrapper and binding
#[derive(Debug, Clone)]
pub struct Bank {
    opaque: JsValue,
}
impl Bank {
    pub fn get_event_list(&self, capacity: i32) -> Result<Vec<EventDescription>, Error> {
        let result = Studio_Bank_GetEventList(&self.opaque, capacity);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(result
                .1
                .into_iter()
                .map(|opaque| EventDescription { opaque })
                .collect()),
            err => Err(err_fmod!("Studio_Bank_GetEventList", err)),
        }
    }
    pub fn get_event_count(&self) -> Result<i32, Error> {
        let result = Studio_Bank_GetEventCount(&self.opaque);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(result.1),
            err => Err(err_fmod!("Studio_Bank_GetEventCount", err)),
        }
    }
}
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    fn Studio_Bank_GetEventList(bank: &JsValue, capacity: i32) -> JsValueVecJSResult;
    #[wasm_bindgen]
    fn Studio_Bank_GetEventCount(bank: &JsValue) -> I32JSResult;
}

// EventDescription wrapper and binding
#[derive(Debug, Clone)]
pub struct EventDescription {
    opaque: JsValue,
}
impl EventDescription {
    pub fn get_path(&self) -> Result<String, Error> {
        let result = Studio_EventDescription_GetPath(&self.opaque);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(result.1),
            err => Err(err_fmod!("Studio_EventDescription_GetPath", err)),
        }
    }
    pub fn create_instance(&self) -> Result<EventInstance, Error> {
        let result = Studio_EventDescription_CreateInstance(&self.opaque);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(EventInstance { opaque: result.1 }),
            err => Err(err_fmod!("Studio_EventDescription_CreateInstance", err)),
        }
    }
    pub fn get_instance_count(&self) -> Result<i32, Error> {
        let result = Studio_EventDescription_GetInstanceCount(&self.opaque);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(result.1),
            err => Err(err_fmod!("Studio_EventDescription_GetInstanceCount", err)),
        }
    }
}
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    fn Studio_EventDescription_GetPath(description: &JsValue) -> StringJSResult;
    #[wasm_bindgen]
    fn Studio_EventDescription_CreateInstance(description: &JsValue) -> JsValueJSResult;
    #[wasm_bindgen]
    fn Studio_EventDescription_GetInstanceCount(description: &JsValue) -> I32JSResult;
}

// EventInstance wrapper and binding
#[derive(Debug, Clone)]
pub struct EventInstance {
    opaque: JsValue,
}
impl EventInstance {
    pub fn start(&self) -> Result<(), Error> {
        let result = Studio_EventInstance_Start(&self.opaque);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(()),
            err => Err(err_fmod!("Studio_EventInstance_Start", err)),
        }
    }
    pub fn release(&self) -> Result<(), Error> {
        let result = Studio_EventInstance_Release(&self.opaque);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(()),
            err => Err(err_fmod!("Studio_EventInstance_Release", err)),
        }
    }
    pub fn get_3d_attributes(&self) -> Result<Attributes3d, Error> {
        let result = Studio_EventInstance_Get3DAttributes(&self.opaque);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(Attributes3d::from(result.1)),
            err => Err(err_fmod!("Studio_EventInstance_Get3DAttributes", err)),
        }
    }
    pub fn set_3d_attributes(&self, attributes: Attributes3d) -> Result<(), Error> {
        let result =
            Studio_EventInstance_Set3DAttributes(&self.opaque, Attributes3d::from(attributes));
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(()),
            err => Err(err_fmod!("Studio_EventInstance_Set3DAttributes", err)),
        }
    }
    pub fn get_pitch(&self) -> Result<(f32, f32), Error> {
        let result = Studio_EventInstance_GetPitch(&self.opaque);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok((result.1, result.2)),
            err => Err(err_fmod!("Studio_EventInstance_GetPitch", err)),
        }
    }
    pub fn set_pitch(&self, pitch: f32) -> Result<(), Error> {
        let result = Studio_EventInstance_SetPitch(&self.opaque, pitch);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(()),
            err => Err(err_fmod!("Studio_EventInstance_SetPitch", err)),
        }
    }
    pub fn get_property(&self, index: EventProperty) -> Result<f32, Error> {
        let result = Studio_EventInstance_GetProperty(&self.opaque, EventProperty::from(index));
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(result.1),
            err => Err(err_fmod!("Studio_EventInstance_GetProperty", err)),
        }
    }
    pub fn set_property(&self, index: EventProperty, value: f32) -> Result<(), Error> {
        let result =
            Studio_EventInstance_SetProperty(&self.opaque, EventProperty::from(index), value);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(()),
            err => Err(err_fmod!("Studio_EventInstance_SetProperty", err)),
        }
    }
    pub fn get_timeline_position(&self) -> Result<i32, Error> {
        let result = Studio_EventInstance_GetTimelinePosition(&self.opaque);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(result.1),
            err => Err(err_fmod!("Studio_EventInstance_GetTimelinePosition", err)),
        }
    }
    pub fn set_timeline_position(&self, position: i32) -> Result<(), Error> {
        let result = Studio_EventInstance_SetTimelinePosition(&self.opaque, position);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(()),
            err => Err(err_fmod!("Studio_EventInstance_SetTimelinePosition", err)),
        }
    }
    pub fn get_volume(&self) -> Result<(f32, f32), Error> {
        let result = Studio_EventInstance_GetVolume(&self.opaque);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok((result.1, result.2)),
            err => Err(err_fmod!("Studio_EventInstance_GetVolume", err)),
        }
    }
    pub fn set_volume(&self, volume: f32) -> Result<(), Error> {
        let result = Studio_EventInstance_SetVolume(&self.opaque, volume);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(()),
            err => Err(err_fmod!("Studio_EventInstance_SetVolume", err)),
        }
    }
    pub fn is_virtual(&self) -> Result<bool, Error> {
        let result = Studio_EventInstance_IsVirtual(&self.opaque);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(result.1),
            err => Err(err_fmod!("Studio_EventInstance_IsVirtual", err)),
        }
    }
    pub fn get_parameter_by_name(&self, name: &str) -> Result<(f32, f32), Error> {
        let result = Studio_EventInstance_GetParameterByName(&self.opaque, name);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok((result.1, result.2)),
            err => Err(err_fmod!("Studio_EventInstance_GetParameterByName", err)),
        }
    }
    pub fn set_parameter_by_name(
        &self,
        name: &str,
        value: f32,
        ignore_seek_speed: bool,
    ) -> Result<(), Error> {
        let result =
            Studio_EventInstance_SetParameterByName(&self.opaque, name, value, ignore_seek_speed);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(()),
            err => Err(err_fmod!("Studio_EventInstance_SetParameterByName", err)),
        }
    }
    pub fn stop(&self, mode: StopMode) -> Result<(), Error> {
        let result = Studio_EventInstance_Stop(&self.opaque, StopMode::from(mode));
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(()),
            err => Err(err_fmod!("Studio_EventInstance_Stop", err)),
        }
    }
    pub fn get_paused(&self) -> Result<bool, Error> {
        let result = Studio_EventInstance_GetPaused(&self.opaque);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(result.1),
            err => Err(err_fmod!("Studio_EventInstance_GetPaused", err)),
        }
    }
    pub fn set_paused(&self, paused: bool) -> Result<(), Error> {
        let result = Studio_EventInstance_SetPaused(&self.opaque, paused);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(()),
            err => Err(err_fmod!("Studio_EventInstance_SetPaused", err)),
        }
    }
    pub fn get_playback_state(&self) -> Result<PlaybackState, Error> {
        let result = Studio_EventInstance_GetPlaybackState(&self.opaque);
        match FMODResult::from(result.0) {
            FMODResult::Ok => Ok(match result.1 {
                PlaybackState::Playing => PlaybackState::Playing,
                PlaybackState::Sustaining => PlaybackState::Sustaining,
                PlaybackState::Stopped => PlaybackState::Stopped,
                PlaybackState::Starting => PlaybackState::Starting,
                PlaybackState::Stopping => PlaybackState::Stopping,
            }),
            err => Err(err_fmod!("Studio_EventInstance_GetPitch", err)),
        }
    }
}
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    fn Studio_EventInstance_Start(instance: &JsValue) -> JSResult;
    #[wasm_bindgen]
    fn Studio_EventInstance_Release(instance: &JsValue) -> JSResult;
    #[wasm_bindgen]
    fn Studio_EventInstance_Get3DAttributes(instance: &JsValue) -> Attributes3dJSResult;
    #[wasm_bindgen]
    fn Studio_EventInstance_Set3DAttributes(
        instance: &JsValue,
        attributes: Attributes3d,
    ) -> JSResult;
    #[wasm_bindgen]
    fn Studio_EventInstance_GetPitch(instance: &JsValue) -> F32F32JSResult;
    #[wasm_bindgen]
    fn Studio_EventInstance_SetPitch(instance: &JsValue, pitch: f32) -> JSResult;
    #[wasm_bindgen]
    fn Studio_EventInstance_GetProperty(instance: &JsValue, index: EventProperty) -> F32JSResult;
    #[wasm_bindgen]
    fn Studio_EventInstance_SetProperty(
        instance: &JsValue,
        index: EventProperty,
        value: f32,
    ) -> JSResult;
    #[wasm_bindgen]
    fn Studio_EventInstance_GetTimelinePosition(instance: &JsValue) -> I32JSResult;
    #[wasm_bindgen]
    fn Studio_EventInstance_SetTimelinePosition(instance: &JsValue, position: i32) -> JSResult;
    #[wasm_bindgen]
    fn Studio_EventInstance_GetVolume(instance: &JsValue) -> F32F32JSResult;
    #[wasm_bindgen]
    fn Studio_EventInstance_SetVolume(instance: &JsValue, volume: f32) -> JSResult;
    #[wasm_bindgen]
    fn Studio_EventInstance_IsVirtual(instance: &JsValue) -> BoolJSResult;
    #[wasm_bindgen]
    fn Studio_EventInstance_GetParameterByName(instance: &JsValue, name: &str) -> F32F32JSResult;
    #[wasm_bindgen]
    fn Studio_EventInstance_SetParameterByName(
        instance: &JsValue,
        name: &str,
        value: f32,
        ignore_seek_speed: bool,
    ) -> JSResult;
    #[wasm_bindgen]
    fn Studio_EventInstance_Stop(instance: &JsValue, stop_mode: StopMode) -> JSResult;
    #[wasm_bindgen]
    fn Studio_EventInstance_SetPaused(instance: &JsValue, paused: bool) -> JSResult;
    #[wasm_bindgen]
    fn Studio_EventInstance_GetPaused(instance: &JsValue) -> BoolJSResult;
    #[wasm_bindgen]
    fn Studio_EventInstance_GetPlaybackState(instance: &JsValue) -> PlaybackStateJSResult;
}

// Bus wrapper and binding
#[derive(Debug, Clone)]
pub struct Bus {
    opaque: JsValue,
}
impl Bus {
    pub fn set_mute(&self, mute: bool) -> Result<(), ()> {
        Studio_Bus_SetMute(&self.opaque, mute);
        Ok(())
    }
}
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    fn Studio_Bus_SetMute(bus: &JsValue, mute: bool);
}

// Structs, bitflags and enums for libfmod parity
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
#[wasm_bindgen]
impl Vector {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}
impl From<[f32; 3]> for Vector {
    fn from(value: [f32; 3]) -> Vector {
        Vector {
            x: value[0],
            y: value[1],
            z: value[2],
        }
    }
}
impl From<Vector> for [f32; 3] {
    fn from(value: Vector) -> [f32; 3] {
        [value.x, value.y, value.z]
    }
}
impl From<(f32, f32, f32)> for Vector {
    fn from(value: (f32, f32, f32)) -> Vector {
        Vector {
            x: value.0,
            y: value.1,
            z: value.2,
        }
    }
}
impl From<Vector> for (f32, f32, f32) {
    fn from(value: Vector) -> (f32, f32, f32) {
        (value.x, value.y, value.z)
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Attributes3d {
    pub position: Vector,
    pub velocity: Vector,
    pub forward: Vector,
    pub up: Vector,
}

#[wasm_bindgen]
impl Attributes3d {
    #[wasm_bindgen(constructor)]
    pub fn new(position: Vector, velocity: Vector, forward: Vector, up: Vector) -> Self {
        Self {
            position,
            velocity,
            forward,
            up,
        }
    }
}

// Enums below are repr(i32) and explicitly annotated with numbers as source of
// truth for those are not us.

#[wasm_bindgen]
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlaybackState {
    Playing = 0,
    Sustaining = 1,
    Stopped = 2,
    Starting = 3,
    Stopping = 4,
}

#[wasm_bindgen]
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EventProperty {
    ChannelPriority = 0,
    ScheduleDelay = 1,
    ScheduleLookahead = 2,
    MinimumDistance = 3,
    MaximumDistance = 4,
    Cooldown = 5,
    Max = 6,
}

#[wasm_bindgen]
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StopMode {
    AllowFadeout = 0,
    Immediate = 1,
}

// Copy of libfmod's Error
#[derive(Debug)]
pub enum Error {
    Fmod {
        function: String,
        code: i32,
        message: String,
    },
    EnumBindgen {
        enumeration: String,
        value: String,
    },
    String(IntoStringError),
    StringNul(NulError),
    NotDspFft,
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Fmod {
                function,
                code,
                message,
            } => {
                write!(f, "{}: {} ({})", function, message, code)
            }
            Error::EnumBindgen { enumeration, value } => {
                write!(
                    f,
                    "FMOD returns unexpected value {} for {} enum",
                    value, enumeration
                )
            }
            Error::String(_) => {
                write!(f, "invalid UTF-8 when converting C string")
            }
            Error::StringNul(_) => {
                write!(
                    f,
                    "nul byte was found in the middle, C strings can't contain it"
                )
            }
            Error::NotDspFft => {
                write!(f, "trying get FFT from DSP which not FFT")
            }
        }
    }
}
impl std::error::Error for Error {}
impl From<NulError> for Error {
    fn from(error: NulError) -> Self {
        Error::StringNul(error)
    }
}

// Copy of bitflags that libfmod provides. Unfortunately, if FMOD changes these
// these will be updated though. But I guess for that exact reason, FMOD
// wouldn't make a breaking change on these. This was the best I could come up
// without making libfmod a dependency.
bitflags! {
    pub struct Init: u32 {
        const NORMAL = 0x00000000;
        const STREAM_FROM_UPDATE = 0x00000001;
        const MIX_FROM_UPDATE = 0x00000002;
        const RIGHTHANDED_3D = 0x00000004;
        const CLIP_OUTPUT = 0x00000008;
        const CHANNEL_LOWPASS = 0x00000100;
        const CHANNEL_DISTANCEFILTER = 0x00000200;
        const PROFILE_ENABLE = 0x00010000;
        const VOL0_BECOMES_VIRTUAL = 0x00020000;
        const GEOMETRY_USECLOSEST = 0x00040000;
        const PREFER_DOLBY_DOWNMIX = 0x00080000;
        const THREAD_UNSAFE = 0x00100000;
        const PROFILE_METER_ALL = 0x00200000;
        const MEMORY_TRACKING = 0x00400000;
    }
    pub struct LoadBank: u32 {
        const NORMAL = 0x00000000;
        const NONBLOCKING = 0x00000001;
        const DECOMPRESS_SAMPLES = 0x00000002;
        const UNENCRYPTED = 0x00000004;
    }

    pub struct StudioInit: u32 {
        const NORMAL = 0x00000000;
        const LIVEUPDATE = 0x00000001;
        const ALLOW_MISSING_PLUGINS = 0x00000002;
        const SYNCHRONOUS_UPDATE = 0x00000004;
        const DEFERRED_CALLBACKS = 0x00000008;
        const LOAD_FROM_UPDATE = 0x00000010;
        const MEMORY_TRACKING = 0x00000020;
    }
}

// Same as above, but this is for FMOD's Result enum that functions return
// when they are called. This is different from libfmod, as I had the lib-
// erty of creating an actual enum. Maybe I could've gone with wasm-bindgen'ing
// this too. Or maybe I should use i32 and TryFrom for the above enums too?
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i32)]
enum FMODResult {
    Ok = 0,
    ErrBadCommand = 1,
    ErrChannelAlloc = 2,
    ErrChannelStolen = 3,
    ErrDMA = 4,
    ErrDSPConnection = 5,
    ErrDSPDontProcess = 6,
    ErrDSPFormat = 7,
    ErrDSPInUse = 8,
    ErrDSPNotFound = 9,
    ErrDSPPReserved = 10,
    ErrDSPSilence = 11,
    ErrDSPTtype = 12,
    ErrFileBad = 13,
    ErrFileCouldNotSeek = 14,
    ErrFileDiskEjected = 15,
    ErrFileEOF = 16,
    ErrFileEndOfData = 17,
    ErrFileNotFound = 18,
    ErrFormat = 19,
    ErrHeaderMismatch = 20,
    ErrHTTP = 21,
    ErrHTTPAccess = 22,
    ErrHTTPProxyAuth = 23,
    ErrHTTPServerError = 24,
    ErrHTTPTimeout = 25,
    ErrInitialization = 26,
    ErrInitialized = 27,
    ErrInternal = 28,
    ErrInvalidFloat = 29,
    ErrInvalidHandle = 30,
    ErrInvalidParam = 31,
    ErrInvalidPosition = 32,
    ErrInvalidSpeaker = 33,
    ErrInvalidSyncPOINT = 34,
    ErrInvalidThread = 35,
    ErrInvalidVector = 36,
    ErrMaxAudible = 37,
    ErrMemory = 38,
    ErrMemoryCantPoint = 39,
    ErrNeeds3D = 40,
    ErrNeedsHardware = 41,
    ErrNetConnect = 42,
    ErrNetSocketError = 43,
    ErrNetURL = 44,
    ErrNetWouldBlock = 45,
    ErrNotReady = 46,
    ErrOutputAllocated = 47,
    ErrOutputCreateBuffer = 48,
    ErrOutputDriverCall = 49,
    ErrOutputFormat = 50,
    ErrOutputInit = 51,
    ErrOutputNoDrivers = 52,
    ErrPlugin = 53,
    ErrPluginMissing = 54,
    ErrPluginResource = 55,
    ErrPluginVersion = 56,
    ErrRecord = 57,
    ErrReverbChannelGroup = 58,
    ErrReverbInstance = 59,
    ErrSubsounds = 60,
    ErrSubsoundAllocated = 61,
    ErrSubsoundCantMove = 62,
    ErrTagNotFound = 63,
    ErrTooManyChannels = 64,
    ErrTruncated = 65,
    ErrUnimplemented = 66,
    ErrUnitialized = 67,
    ErrUnsupported = 68,
    ErrVersion = 69,
    ErrEventAlreadyLoaded = 70,
    ErrEventLiveUpdateBusy = 71,
    ErrEventLiveUpdateMismatch = 72,
    ErrEventLiveUpdateTimeout = 73,
    ErrEventNotFound = 74,
    ErrStudioUnitialized = 75,
    ErrStudioNotLoaded = 76,
    ErrInvalidString = 77,
    ErrAlreadyLocked = 78,
    ErrNotLocked = 79,
    ErrRecordDisconnected = 80,
    ErrTooManySamples = 81,
    /// This is an error code we made up, for the purposes of in case there is
    /// a mismatch in future.
    ErrUnknown = 82,
}

// Rust not giving me choice... Either boilerplate or use some proc macro that
// generates it. I just copied enum above and edited it with multicursor.
impl From<i32> for FMODResult {
    fn from(value: i32) -> Self {
        match value {
            0 => FMODResult::Ok,
            1 => FMODResult::ErrBadCommand,
            2 => FMODResult::ErrChannelAlloc,
            3 => FMODResult::ErrChannelStolen,
            4 => FMODResult::ErrDMA,
            5 => FMODResult::ErrDSPConnection,
            6 => FMODResult::ErrDSPDontProcess,
            7 => FMODResult::ErrDSPFormat,
            8 => FMODResult::ErrDSPInUse,
            9 => FMODResult::ErrDSPNotFound,
            10 => FMODResult::ErrDSPPReserved,
            11 => FMODResult::ErrDSPSilence,
            12 => FMODResult::ErrDSPTtype,
            13 => FMODResult::ErrFileBad,
            14 => FMODResult::ErrFileCouldNotSeek,
            15 => FMODResult::ErrFileDiskEjected,
            16 => FMODResult::ErrFileEOF,
            17 => FMODResult::ErrFileEndOfData,
            18 => FMODResult::ErrFileNotFound,
            19 => FMODResult::ErrFormat,
            20 => FMODResult::ErrHeaderMismatch,
            21 => FMODResult::ErrHTTP,
            22 => FMODResult::ErrHTTPAccess,
            23 => FMODResult::ErrHTTPProxyAuth,
            24 => FMODResult::ErrHTTPServerError,
            25 => FMODResult::ErrHTTPTimeout,
            26 => FMODResult::ErrInitialization,
            27 => FMODResult::ErrInitialized,
            28 => FMODResult::ErrInternal,
            29 => FMODResult::ErrInvalidFloat,
            30 => FMODResult::ErrInvalidHandle,
            31 => FMODResult::ErrInvalidParam,
            32 => FMODResult::ErrInvalidPosition,
            33 => FMODResult::ErrInvalidSpeaker,
            34 => FMODResult::ErrInvalidSyncPOINT,
            35 => FMODResult::ErrInvalidThread,
            36 => FMODResult::ErrInvalidVector,
            37 => FMODResult::ErrMaxAudible,
            38 => FMODResult::ErrMemory,
            39 => FMODResult::ErrMemoryCantPoint,
            40 => FMODResult::ErrNeeds3D,
            41 => FMODResult::ErrNeedsHardware,
            42 => FMODResult::ErrNetConnect,
            43 => FMODResult::ErrNetSocketError,
            44 => FMODResult::ErrNetURL,
            45 => FMODResult::ErrNetWouldBlock,
            46 => FMODResult::ErrNotReady,
            47 => FMODResult::ErrOutputAllocated,
            48 => FMODResult::ErrOutputCreateBuffer,
            49 => FMODResult::ErrOutputDriverCall,
            50 => FMODResult::ErrOutputFormat,
            51 => FMODResult::ErrOutputInit,
            52 => FMODResult::ErrOutputNoDrivers,
            53 => FMODResult::ErrPlugin,
            54 => FMODResult::ErrPluginMissing,
            55 => FMODResult::ErrPluginResource,
            56 => FMODResult::ErrPluginVersion,
            57 => FMODResult::ErrRecord,
            58 => FMODResult::ErrReverbChannelGroup,
            59 => FMODResult::ErrReverbInstance,
            60 => FMODResult::ErrSubsounds,
            61 => FMODResult::ErrSubsoundAllocated,
            62 => FMODResult::ErrSubsoundCantMove,
            63 => FMODResult::ErrTagNotFound,
            64 => FMODResult::ErrTooManyChannels,
            65 => FMODResult::ErrTruncated,
            66 => FMODResult::ErrUnimplemented,
            67 => FMODResult::ErrUnitialized,
            68 => FMODResult::ErrUnsupported,
            69 => FMODResult::ErrVersion,
            70 => FMODResult::ErrEventAlreadyLoaded,
            71 => FMODResult::ErrEventLiveUpdateBusy,
            72 => FMODResult::ErrEventLiveUpdateMismatch,
            73 => FMODResult::ErrEventLiveUpdateTimeout,
            74 => FMODResult::ErrEventNotFound,
            75 => FMODResult::ErrStudioUnitialized,
            76 => FMODResult::ErrStudioNotLoaded,
            77 => FMODResult::ErrInvalidString,
            78 => FMODResult::ErrAlreadyLocked,
            79 => FMODResult::ErrNotLocked,
            80 => FMODResult::ErrRecordDisconnected,
            81 => FMODResult::ErrTooManySamples,
            _ => FMODResult::ErrUnknown,
        }
    }
}

// Used to create our unique JS classes for each distinct return type
macro_rules! create_js_result {
    ($type:ident, $value_0:ty) => {
        #[wasm_bindgen]
        #[derive(Clone, Debug)]
        struct $type(i32, $value_0);

        #[wasm_bindgen]
        impl $type {
            #[wasm_bindgen(constructor)]
            pub fn new(fmod_result: i32, value_0: $value_0) -> Self {
                Self(fmod_result, value_0)
            }
        }
    };
    ($type:ident, $value_0:ty, $value_1:ty) => {
        #[wasm_bindgen]
        #[derive(Clone, Debug)]
        struct $type(i32, $value_0, $value_1);

        #[wasm_bindgen]
        impl $type {
            #[wasm_bindgen(constructor)]
            pub fn new(fmod_result: i32, value_0: $value_0, value_1: $value_1) -> Self {
                Self(fmod_result, value_0, value_1)
            }
        }
    };
}

// No type, just result
#[wasm_bindgen]
struct JSResult(i32);

#[wasm_bindgen]
impl JSResult {
    #[wasm_bindgen(constructor)]
    pub fn new(fmod_result: i32) -> Self {
        Self(fmod_result)
    }
}

// Generic ones
create_js_result!(JsValueJSResult, JsValue);
create_js_result!(JsValueVecJSResult, Vec<JsValue>);

// Our custom stuff
create_js_result!(Attributes3dJSResult, Attributes3d);
create_js_result!(PlaybackStateJSResult, PlaybackState);

// Primitives
create_js_result!(I32JSResult, i32);
create_js_result!(F32JSResult, f32);
create_js_result!(BoolJSResult, bool);
create_js_result!(StringJSResult, String);

// Multiple primitive
create_js_result!(F32F32JSResult, f32, f32);
