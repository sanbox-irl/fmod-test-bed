///
/// # Explanation of the glue code and rationale behind the style.
/// 
/// Glue code exists, because out arguments set a field named `val` for the
/// given object. For the actual returns, they return a result enum. A
/// JavaScript class with methods could've been used here, but I didn't use it 
/// to make it parallel to C api, which also is used by libfmod. So, any
/// function name here is same in C and in the inner calls of libfmod FFI
/// bindings.
///
/// Functions can have three types of return, just the result, or a class of 2
/// elements, where second element is the value, or class of 3 elements, where
/// last two elements are values.
///
/// Then, the Rust side of this code checks whether result is OK or not and
/// converts it into Result<T, E>. I made this choice because as far as I can
/// understand currently sending Result<T, E> back from JS requires
/// throw/catch. While I think maybe I could've written a code similar to
/// generated code, it wouldn't be concise as this one. Plus, this style also
/// matches the way libfmod handles it in the Rust side.
///
/// Initially, these result types were going to be objects, but unfortunately,
/// only way to read those from rust side was to use serde with wasm-bindgen.
/// As I didn't know the implications of this, I decided against it and instead
/// created classes. Unfortunately, this caused multiple classes for different
/// primitive & struct types. For opaque types, using JsValue was easiest and
/// least cumbersome option.
///
/// Where it was needed to send buffers with length and *u8, &[u8] was used,
/// as that is translated to JS typed arrays by the bindgen. While it differs
/// from the C API, I think letting wasm-bindgen to handle it was better choice
/// here, as input for FMOD Emscripten was already going to be a typed JS array.
///
/// Another change was made for functions that return size so one can allocate,
/// it upfront. Doing this in the rust-side would require two calls and either,
/// copying all that data and slicing it in the rust side, or doing the slicing
/// via js_sys on JS Array and then iterate over it. So, path of least resist-
/// ance was neither of those, so I decided to stray away from the API compat
/// for that reason. This is only the case for JS side though, rust side of API
/// matches libfmod 1:1.
///
/// Single line statements with results are set to a variable named result, then
/// returned for the readibility purposes, due to the lack of type information.
///
/// All this is based on insights gathered from these sources:
/// - https://www.fmod.com/docs/2.02/api/platforms-html5.html
/// - https://www.fmod.com/docs/2.02/api/studio-api.html
/// - https://rustwasm.github.io/docs/wasm-bindgen/introduction.html
///
///
/// In theory, all this can be generated. Another thing is, this file currently
/// only works for wasm-bindgen's no_modules target. I was initially using web
/// target, but that creates modules and I didn't want to do the hassle of imp-
/// orting FMOD and this file into created JS module by the wasm-bindgen. So,
/// this file can be made so it works for both of those, by making wasm_bindgen
/// imports optional and importing FMODModule. 
/// 

// FMODModule function that FMOD provides populates this object.
const FMOD = {};

// Called when Emscripten runtime has initialized
FMOD.onRuntimeInitialized = () => {
};
// Called before FMOD runs, but after Emscripten runtime has initialized
FMOD.preRun = () => {
};

// Populates FMOD
FMODModule(FMOD);

// Imports generated classes from wasm_bindgen global
const {
  // Structs
  Vector,
  Attributes3d,
  // Typeless results
  JSResult,
  JsValueJSResult,
  JsValueVecJSResult,
  
  // Typed results
  Attributes3dJSResult,
  PlaybackStateJSResult,
  
  // Typed primitive results
  I32JSResult,
  F32JSResult,
  BoolJSResult,
  StringJSResult,
  
  // Typed tuple primitive results
  F32F32JSResult,
} = wasm_bindgen;


// Below are the bindings that wasm-bindgen calls.

// Studio

function Studio_System_Create() {
  const studio = {};
  const result = FMOD.Studio_System_Create(studio);
  return new JsValueJSResult(result, studio.val);
}
function Studio_System_Initialize(
  studio,
  maxChannels,
  studioFlags,
  flags,
  extraDriverData,
) {
  const result = studio.initialize(
    maxChannels,
    studioFlags,
    flags,
    extraDriverData,
  );
  return new JSResult(result);
}
function Studio_System_LoadBankMemory(studio, buffer, mode, flags) {
  const bank = {};
  const result = studio.loadBankMemory(
    buffer,
    buffer.length,
    mode,
    flags,
    bank,
  );
  return new JsValueJSResult(result, bank.val);
}
function Studio_System_UnloadAll(studio) {
  const result = studio.unloadAll();
  return new JSResult(result);
}
function Studio_System_GetEvent(studio, path) {
  const event = {};
  const result = studio.getEvent(path, event);
  return new JsValueJSResult(result, event.val);
}
function Studio_System_GetBus(studio, path) {
  const bus = {};
  const result = studio.getBus(path, bus);
  return new JsValueJSResult(result, bus.val);
}
function Studio_System_SetParameterByName(
  studio,
  name,
  value,
  ignoreSeekSpeed
) {
  const result = studio.setParameterByName(name, value, ignoreSeekSpeed);
  return new JSResult(result);
}
function Studio_System_SetListenerAttributes(
  studio,
  listener,
  attributes,
  attenuationPosition,
) {
  const result = studio.setListenerAttributes(
    listener,
    attributes,
    attenuationPosition,
  );
  return new JSResult(result);
}
function Studio_System_Update(studio) {
  const result = studio.update();
  return new JSResult(result);
}

// Bank

function Studio_Bank_GetEventList(bank, capacity) {
  const array = {};
  const count = {};
  const result = bank.getEventList(array, capacity, count);
  return new JsValueVecJSResult(result, array.val.slice(0, count.val));
}
function Studio_Bank_GetEventCount(bank) {
  const count = {};
  const result = bank.getEventCount(count);
  return new I32JSResult(result, count.val);
}

// EventDescription

function Studio_EventDescription_GetPath(eventDescription) {
  const retrieved = {};
  let result = eventDescription.getPath(null, 0, retrieved);
  // 0 is OK
  if (result !== 0) {
    return new StringJSResult(result, null);
  }
  const path = {};
  result = eventDescription.getPath(path, retrieved.val, retrieved);
  return new StringJSResult(result, path.val);
}
function Studio_EventDescription_CreateInstance(eventDescription) {
  const instance = {};
  const result = eventDescription.createInstance(instance);
  return new JsValueJSResult(result, instance.val);
}
function Studio_EventDescription_GetInstanceCount(eventDescription) {
  const count = {};
  const result = eventDescription.getInstanceCount(count);
  return new I32JSResult(result, count.val);
}

// EventInstance

function Studio_EventInstance_Start(eventInstance) {
  const result = eventInstance.start();
  return new JSResult(result);
}
function Studio_EventInstance_Release(eventInstance) {
  const result = eventInstance.release();
  return new JSResult(result);
}
function Studio_EventInstance_Get3DAttributes(eventInstance) {
  const attributes = {};
  const result = eventInstance.get3DAttributes(attributes);
  // This one is a bit annoying
  return new Attributes3dJSResult(result, new Attributes3d(
    new Vector(attributes["position.x"], attributes["position.y"], attributes["position.z"]),
    new Vector(attributes["velocity.x"], attributes["velocity.y"], attributes["velocity.z"]),
    new Vector(attributes["forward.x"], attributes["forward.y"], attributes["forward.z"]),
    new Vector(attributes["up.x"], attributes["up.y"], attributes["up.z"]),
  ));
}
function Studio_EventInstance_Set3DAttributes(eventInstance, attributes) {
  const result = eventInstance.set3DAttributes(attributes);
  return new JSResult(result);
}
function Studio_EventInstance_SetPitch(eventInstance, pitch) {
  const result = eventInstance.setPitch(pitch);
  return new JSResult(result);
}
function Studio_EventInstance_GetPitch(eventInstance) {
  const pitch = {};
  const finalPitch = {};
  const result = eventInstance.getPitch(pitch, finalPitch);
  return new F32F32JSResult(result, pitch.val, finalPitch.val);
}
function Studio_EventInstance_SetProperty(eventInstance, index, value) {
  const result = eventInstance.setProperty(index, value);
  return new JSResult(result);
}
function Studio_EventInstance_GetProperty(eventInstance, index) {
  // FMOD docs use the name value, so I used value.
  //property would be better but imho.
  const value = {};
  const result = eventInstance.getProperty(index, value);
  return new F32JSResult(result, value.val);
}
function Studio_EventInstance_SetTimelinePosition(eventInstance, position) {
  const result = eventInstance.setTimelinePosition(position);
  return new JSResult(result);
}
function Studio_EventInstance_GetTimelinePosition(eventInstance) {
  const position = {};
  const result = eventInstance.getTimelinePosition(position);
  return new I32JSResult(result, position.val);
}
function Studio_EventInstance_SetVolume(eventInstance, volume) {
  const result = eventInstance.setVolume(volume);
  return new JSResult(result);
}
function Studio_EventInstance_GetVolume(eventInstance) {
  const volume = {};
  const finalVolume = {};
  const result = eventInstance.getVolume(volume, finalVolume);
  return new F32F32JSResult(result, volume.val, finalVolume.val);
}
function Studio_EventInstance_IsVirtual(eventInstance) {
  const virtual = {};
  const result = eventInstance.isVirtual(virtual);
  return new BoolJSResult(result, virtual.val);
}
function Studio_EventInstance_GetParameterByName(eventInstance, name) {
  const value = {};
  const finalValue = {};
  const result = eventInstance.getParameterByName(name, value, finalValue);
  return new F32F32JSResult(result, value.val, finalValue.val);
}
function Studio_EventInstance_SetParameterByName(
  eventInstance,
  name,
  value,
  ignoreSeekSpeed,
) {
  const result = eventInstance.setParameterByName(
    name,
    value,
    ignoreSeekSpeed,
  );
  return new JSResult(result);
}
function Studio_EventInstance_Stop(eventInstance, mode) {
  const result = eventInstance.stop(mode);
  return new JSResult(result);
}
function Studio_EventInstance_SetPaused(eventInstance, paused) {
  const result = eventInstance.setPaused(paused);
  return new JSResult(result);
}
function Studio_EventInstance_GetPaused(eventInstance) {
  const paused = {};
  const result = eventInstance.getPaused(paused);
  return new BoolJSResult(result, paused.val);
}
function Studio_EventInstance_GetPlaybackState(eventInstance) {
  const state = {};
  const result = eventInstance.getPlaybackState(state);
  return new PlaybackStateJSResult(result, state.val);
}

// Bus

function Studio_Bus_SetMute(bus, mute) {
  const result = bus.setMute(mute);
  return new JSResult(result);
}
