use ash::{vk, Entry, Instance, ext::debug_utils};
use std::{
  ffi::{CStr, CString},
  os::raw::{c_char, c_void},
};

#[cfg(debug_assertions)]
pub const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
pub const ENABLE_VALIDATION_LAYERS: bool = false;

const REQUIRED_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

unsafe extern "system" fn vulkan_debug_callback(
    flag: vk::DebugUtilsMessageSeverityFlagsEXT,
    typ: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    use vk::DebugUtilsMessageSeverityFlagsEXT as Flag;

    let message = CStr::from_ptr((*p_callback_data).p_message);
    match flag {
        Flag::VERBOSE => log::debug!("{:?} - {:?}", typ, message),
        Flag::INFO => log::info!("{:?} - {:?}", typ, message),
        Flag::WARNING => log::warn!("{:?} - {:?}", typ, message),
        _ => log::error!("{:?} - {:?}", typ, message),
    }
    vk::FALSE
}

/// Setup the debug message if validation layers are enabled.
pub fn setup_debug_messenger(
  entry: &Entry,
  instance: &Instance,
) -> Option<(debug_utils::Instance, vk::DebugUtilsMessengerEXT)> {
  if !ENABLE_VALIDATION_LAYERS {
      return None;
  }

  let create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
      .flags(vk::DebugUtilsMessengerCreateFlagsEXT::empty())
      .message_severity(
          vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
              | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
              | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
      )
      .message_type(
          vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
              | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
              | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
      )
      .pfn_user_callback(Some(vulkan_debug_callback));
  let debug_utils = debug_utils::Instance::new(entry, instance);
  let debug_utils_messenger = unsafe {
      debug_utils
          .create_debug_utils_messenger(&create_info, None)
          .unwrap()
  };

  Some((debug_utils, debug_utils_messenger))
}

/// Get the pointers to the validation layers names.
/// Also return the corresponding `CString` to avoid dangling pointers.
pub fn get_layer_names_and_pointers() -> (Vec<CString>, Vec<*const c_char>) {
  let layer_names = REQUIRED_LAYERS
      .iter()
      .map(|name| CString::new(*name).unwrap())
      .collect::<Vec<_>>();
  let layer_names_ptrs = layer_names
      .iter()
      .map(|name| name.as_ptr())
      .collect::<Vec<_>>();
  (layer_names, layer_names_ptrs)
}

/// Check if the required validation set in `REQUIRED_LAYERS`
/// are supported by the Vulkan instance.
///
/// # Panics
///
/// Panic if at least one on the layer is not supported.
pub fn check_validation_layer_support(entry: &Entry) {
  let supported_layers = unsafe { entry.enumerate_instance_layer_properties().unwrap() };
  for required in REQUIRED_LAYERS.iter() {
      let found = supported_layers.iter().any(|layer| {
          let name = unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) };
          let name = name.to_str().expect("Failed to get layer name pointer");
          required == &name
      });

      if !found {
          panic!("Validation layer not supported: {}", required);
      }
  }
}


fn main() {
    let entry = unsafe { Entry::load().expect("Failed to create entry.") };
    let instance = create_instance(&entry);

    println!("vulkan instance created");
}

fn create_instance(entry: &Entry) -> Instance {
    let app_name = CString::new("Vulkan Application").unwrap();
    let engine_name = CString::new("No Engine").unwrap();
    let app_info = vk::ApplicationInfo::default()
        .application_name(app_name.as_c_str())
        .application_version(vk::make_api_version(0, 0, 1, 0))
        .engine_name(engine_name.as_c_str())
        .engine_version(vk::make_api_version(0, 0, 1, 0))
        .api_version(vk::make_api_version(0, 1, 2, 0));

    let mut extension_names: Vec<*const i8> = Vec::new();
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        extension_names.push(ash::khr::portability_enumeration::NAME.as_ptr());
        // Enabling this extension is a requirement when using `VK_KHR_portability_subset`
        // extension_names.push(ash::khr::get_physical_device_properties2::NAME.as_ptr());
    }

    let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
        vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
    } else {
        vk::InstanceCreateFlags::default()
    };

    let (_layer_names, layer_names_ptrs) = get_layer_names_and_pointers();
    let mut instance_create_info = vk::InstanceCreateInfo::default()
        .application_info(&app_info)
        .enabled_extension_names(&extension_names)
        .flags(create_flags);
    if ENABLE_VALIDATION_LAYERS {
        check_validation_layer_support(entry);
        instance_create_info = instance_create_info.enabled_layer_names(&layer_names_ptrs);
    }

    unsafe { entry.create_instance(&instance_create_info, None).unwrap() }
}
