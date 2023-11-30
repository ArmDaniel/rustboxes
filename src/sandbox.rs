use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;
use crate::{Config, FolderMapper};
use xml::writer::{EmitterConfig, EventWriter, XmlEvent};

const WINDOWS_SANDBOX_CONFIG_FILE_SUFFIX: &str = ".wsb";

pub(crate) struct OfflineSandbox { pub(crate) config: Config, }
pub(crate) struct OnlineSandbox {
    pub(crate) config: Config,
    pub(crate) launch_new_instance: bool,
}
pub(crate) struct OfflineSession {
    pub(crate) sandbox : OfflineSandbox
}

impl OfflineSession {
    pub(crate) fn run(&self){
        let config_file = generate_config_file(&self.sandbox.config);

        let mut temp_file = NamedTempFile::new().expect("Failed to create temporary file.");
        let mut config_file_path = PathBuf::from(temp_file.path());
        config_file_path.set_extension(WINDOWS_SANDBOX_CONFIG_FILE_SUFFIX);

        File::create(&config_file_path)
            .and_then(|mut file| {
                file.write_all(config_file.as_bytes())
            })
            .expect("Failed to write to config file");

        OfflineSandbox::start_sandbox(&self.sandbox, config_file_path.to_str().unwrap());
    }
}

fn get_boolean_text(value: bool) -> &'static str{
    if value { "Default "} else { "Disabled" }
}
fn format_folder_mappers(folder_mappers: &Vec<FolderMapper>, writer: &mut EventWriter<&mut Vec<u8>>) {
    writer.write(XmlEvent::start_element("MappedFolders")).unwrap();
    for folder_mapper in folder_mappers {
        writer.write(XmlEvent::start_element("MappedFolder")).unwrap();

        writer.write(XmlEvent::start_element("HostFolder")).unwrap();
        writer.write(XmlEvent::characters(&folder_mapper.path)).unwrap();
        writer.write(XmlEvent::end_element()).unwrap(); // HostFolder

        writer.write(XmlEvent::start_element("ReadOnly")).unwrap();
        let read_only_text = if folder_mapper.read_only { "true" } else { "false" };
        writer.write(XmlEvent::characters(read_only_text)).unwrap();
        writer.write(XmlEvent::end_element()).unwrap(); // ReadOnly

        writer.write(XmlEvent::end_element()).unwrap(); // MappedFolder
    }
    writer.write(XmlEvent::end_element()).unwrap(); // MappedFolders
}

fn generate_config_file(config: &Config) -> String {
    let mut buffer = Vec::new();
    let mut writer = EmitterConfig::new().perform_indent(true).create_writer(&mut buffer);

    writer.write(XmlEvent::start_element("Configuration")).unwrap();

    writer.write(XmlEvent::start_element("VGpu")).unwrap();
    writer.write(XmlEvent::characters(get_boolean_text(config.virtual_gpu))).unwrap();
    writer.write(XmlEvent::end_element()).unwrap(); // VGpu

    writer.write(XmlEvent::start_element("Networking")).unwrap();
    writer.write(XmlEvent::characters(get_boolean_text(config.networking))).unwrap();
    writer.write(XmlEvent::end_element()).unwrap(); // Networking

    writer.write(XmlEvent::start_element("LogonCommand")).unwrap();
    writer.write(XmlEvent::start_element("Command")).unwrap();
    writer.write(XmlEvent::characters(&config.logon_script)).unwrap();
    writer.write(XmlEvent::end_element()).unwrap(); // Command
    writer.write(XmlEvent::end_element()).unwrap(); // LogonCommand

    format_folder_mappers(&config.folder_mappers, &mut writer);

    writer.write(XmlEvent::end_element()).unwrap(); // Configuration

    String::from_utf8(buffer).unwrap()
}

impl OfflineSandbox {
    pub(crate) fn start_sandbox(&self,config_file_path: &str){

        Command::new("cmd")
            .args(&["/C","start",config_file_path])
            .spawn()
            .expect("Failed to start sandbox");
    }
}
