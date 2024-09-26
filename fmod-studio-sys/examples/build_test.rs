// SPDX-FileCopyrightText: 2024 Julia DeMille <me@jdemille.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::ptr;

use fmod_studio_sys::*;
use fmod_sys::FMOD_SYSTEM;
fn main() {
    let mut system: *mut FMOD_SYSTEM = ptr::null_mut();
    let res = unsafe {
        fmod_sys::functions::FMOD_System_Create(&mut system, fmod_sys::FMOD_VERSION as u32)
    };
    if res == fmod_sys::FMOD_RESULT::FMOD_OK {
        unsafe { fmod_sys::functions::FMOD_System_Close(system) };
    }
}
