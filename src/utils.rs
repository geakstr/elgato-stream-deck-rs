use core_foundation::string::CFStringRef;
use core_foundation::{
    array::{CFArray, CFArrayRef},
    base::{CFType, TCFType},
    dictionary::{CFDictionaryRef, CFMutableDictionary},
    string::CFString,
};
use core_foundation_sys::base::OSStatus;
use std::os::raw::c_void;
use std::process::Command;

pub fn open_app(app: &str) {
    let _ = Command::new("open")
        .arg("-a")
        .arg(app)
        .status()
        .expect("failed to open app");
}

pub fn select_keyboard_layout(input_source_id: &str) -> Result<(), String> {
    unsafe {
        let cf_input_source_id = CFString::new(input_source_id);
        let mut dict = CFMutableDictionary::new();
        dict.set(
            k_tisproperty_input_source_id(),
            cf_input_source_id.into_CFType(),
        );
        let sources_ref = TISCreateInputSourceList(dict.as_concrete_TypeRef(), false);
        if sources_ref.is_null() {
            return Err("No input sources found.".into());
        }
        let sources = CFArray::<CFType>::wrap_under_create_rule(sources_ref);
        if sources.is_empty() {
            return Err(format!(
                "No matching input source for ID '{}'",
                input_source_id
            ));
        }
        let source = sources
            .get(0)
            .ok_or_else(|| "Empty source array?".to_string())?;
        let source_ref = source.as_concrete_TypeRef() as TISInputSourceRef;
        let status = TISSelectInputSource(source_ref);
        if status != 0 {
            return Err(format!(
                "TISSelectInputSource failed with status {}",
                status
            ));
        }
    }
    Ok(())
}

#[allow(dead_code)]
pub fn list_all_keyboard_layouts() {
    println!("Listing all input sources...");

    unsafe {
        // Get all input sources by passing NULL for properties
        let arr_ref = TISCreateInputSourceList(std::ptr::null(), false);
        if arr_ref.is_null() {
            eprintln!("Failed to retrieve input sources array.");
            return;
        }

        // Convert raw CFArrayRef into high-level CFArray
        let cfarray = CFArray::<*const c_void>::wrap_under_create_rule(arr_ref as *mut _);

        println!("Number of sources: {}", cfarray.len());
        for i in 0..cfarray.len() {
            // Each element is a `*const c_void`; cast to `TISInputSourceRef`
            let raw_ptr = *cfarray.get(i).unwrap();
            let source_ref = raw_ptr as TISInputSourceRef;

            // Grab a few properties:
            let input_source_id = tis_get_string_property(
                source_ref,
                k_tisproperty_input_source_id().as_concrete_TypeRef(),
            );
            let input_source_type = tis_get_string_property(
                source_ref,
                k_tisproperty_input_source_type().as_concrete_TypeRef(),
            );
            let localized_name = tis_get_string_property(
                source_ref,
                k_tisproperty_localized_name().as_concrete_TypeRef(),
            );

            println!("----");
            println!("Index: {}", i);
            println!(
                "  ID:   {}",
                input_source_id
                    .map(|s| s.to_string())
                    .unwrap_or("(none)".into())
            );
            println!(
                "  Type: {}",
                input_source_type
                    .map(|s| s.to_string())
                    .unwrap_or("(none)".into())
            );
            println!(
                "  Name: {}",
                localized_name
                    .map(|s| s.to_string())
                    .unwrap_or("(none)".into())
            );
        }
    }
}

#[repr(C)]
struct __TISInputSource {
    _private: [u8; 0],
}

type TISInputSourceRef = *mut __TISInputSource;

#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    fn TISCreateInputSourceList(
        properties: CFDictionaryRef,
        includeAllInstalled: bool,
    ) -> CFArrayRef;
    fn TISSelectInputSource(inputSource: TISInputSourceRef) -> OSStatus;
    fn TISGetInputSourceProperty(
        inputSource: TISInputSourceRef,
        propertyKey: CFStringRef,
    ) -> *const c_void;
}

fn k_tisproperty_input_source_id() -> CFString {
    CFString::from_static_string("TISPropertyInputSourceID")
}

fn k_tisproperty_input_source_type() -> CFString {
    CFString::from_static_string("TISPropertyInputSourceType")
}

fn k_tisproperty_localized_name() -> CFString {
    CFString::from_static_string("TISPropertyLocalizedName")
}

fn tis_get_string_property(
    input_source: TISInputSourceRef,
    property_key: CFStringRef,
) -> Option<CFString> {
    unsafe {
        let ptr = TISGetInputSourceProperty(input_source, property_key);
        if ptr.is_null() {
            return None;
        }
        let cf_str_ref = ptr as CFStringRef;
        Some(CFString::wrap_under_get_rule(cf_str_ref))
    }
}
