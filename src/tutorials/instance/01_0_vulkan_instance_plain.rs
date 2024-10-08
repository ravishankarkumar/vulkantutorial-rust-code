use ash::{vk, Entry, Instance};

fn main() {
    let entry = unsafe { Entry::load().expect("Failed to create entry.") };
    let instance = create_instance(&entry);

    println!("vulkan instance created");
}

fn create_instance(entry: &Entry) -> Instance {
    let app_info = vk::ApplicationInfo {
        api_version: vk::make_api_version(0, 1, 0, 0),
        ..Default::default()
    };
    let instance_create_info = vk::InstanceCreateInfo {
        p_application_info: &app_info,
        ..Default::default()
    };
    unsafe { entry.create_instance(&instance_create_info, None).unwrap() }
}
