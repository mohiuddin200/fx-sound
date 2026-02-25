// Simple test to verify LADSPA plugin loads correctly

use std::ffi::CString;
use std::os::raw::c_char;

#[link(name = "dl")]
extern "C" {
    fn dlopen(filename: *const c_char, flag: i32) -> *mut std::os::raw::c_void;
    fn dlsym(handle: *mut std::os::raw::c_void, symbol: *const c_char) -> *mut std::os::raw::c_void;
}

const RTLD_NOW: i32 = 2;

#[repr(C)]
struct LadspaDescriptor {
    unique_id: u64,
    label: *mut c_char,
    properties: i32,
    name: *mut c_char,
    maker: *mut c_char,
    copyright: *mut c_char,
    port_count: u64,
    port_descriptors: *mut i32,
    port_names: *mut *mut c_char,
    port_range_hints: *mut std::os::raw::c_void,
    implementation_data: *mut std::os::raw::c_void,
    instantiate: *const std::os::raw::c_void,
    connect_port: *const std::os::raw::c_void,
    activate: *const std::os::raw::c_void,
    run: *const std::os::raw::c_void,
    run_adding: *const std::os::raw::c_void,
    set_run_adding_gain: *const std::os::raw::c_void,
    deactivate: *const std::os::raw::c_void,
    cleanup: *const std::os::raw::c_void,
}

type LadspaDescriptorFn = unsafe extern "C" fn(u64) -> *mut LadspaDescriptor;

#[test]
fn test_ladspa_plugin_loads() {
    let path = "/home/mohiuddin/.ladspa/libfxsonic_dsp.so";
    let cpath = CString::new(path).expect("Invalid path");

    unsafe {
        let handle = dlopen(cpath.as_ptr(), RTLD_NOW);
        assert!(!handle.is_null(), "Failed to load library");

        let sym_name = CString::new("ladspa_descriptor").expect("Invalid symbol");
        let ladspa_descriptor: LadspaDescriptorFn = std::mem::transmute(dlsym(handle, sym_name.as_ptr()));
        assert!((ladspa_descriptor as *const () as *mut ()).is_null() == false, "Failed to find ladspa_descriptor");

        let desc = ladspa_descriptor(0);
        assert!(!desc.is_null(), "Failed to get descriptor for index 0");

        let d = &*desc;
        assert_eq!(d.unique_id, 424242, "Unexpected unique ID");

        let label = std::ffi::CStr::from_ptr(d.label).to_string_lossy();
        assert_eq!(label, "fxsonic_enhancer", "Unexpected label");

        let name = std::ffi::CStr::from_ptr(d.name).to_string_lossy();
        assert!(name.contains("FxSonic"), "Unexpected name: {}", name);

        let prop_hard_rt = 0x4;
        assert!(d.properties & prop_hard_rt != 0, "Plugin is not HARD_RT_CAPABLE");
    }
}

#[test]
fn test_ladspa_descriptor_returns_null_for_invalid_index() {
    let path = "/home/mohiuddin/.ladspa/libfxsonic_dsp.so";
    let cpath = CString::new(path).expect("Invalid path");

    unsafe {
        let handle = dlopen(cpath.as_ptr(), RTLD_NOW);
        assert!(!handle.is_null(), "Failed to load library");

        let sym_name = CString::new("ladspa_descriptor").expect("Invalid symbol");
        let ladspa_descriptor: LadspaDescriptorFn = std::mem::transmute(dlsym(handle, sym_name.as_ptr()));

        let desc = ladspa_descriptor(999);
        assert!(desc.is_null(), "Expected null for invalid index");
    }
}
