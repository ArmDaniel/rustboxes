mod sandbox;


use wmi::{COMLibrary, Variant, WMIConnection};
use std::collections::HashMap;
use crate::sandbox::{Config, OfflineSandbox, OfflineSession, OnlineSandbox};

const FEATURE_WINDOWS_SANDBOX: &str =
"Containers-DisposableClientVM";
const FEATURE_QUERY: &str =
    "SELECT * FROM Win32_OptionalFeature";
const ENABLED_STATE: u32 = 1;




fn check_feature(os: &HashMap<String, Variant>) {
    match (os.get("Name"), os.get("InstallState")) {
        (Some(Variant::String(name)), state) if name == FEATURE_WINDOWS_SANDBOX => {
            if let Some(Variant::UI4(install_state)) = state {
                if *install_state == ENABLED_STATE {
                    println!("{} feature is installed and enabled.", FEATURE_WINDOWS_SANDBOX);
                } else {
                    println!("{} feature is not enabled.", FEATURE_WINDOWS_SANDBOX);
                }
            }
        }
        _ => {},
    }
}

fn com_wmi_connection() -> Result<(), wmi::WMIError>
{
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con)?;

    let results : Vec<HashMap<String, Variant>> = wmi_con.raw_query(FEATURE_QUERY).unwrap();

    Ok(for os in results {
        check_feature(&os);
    })
}
fn main() {

    com_wmi_connection().expect("Failed to fetch WSB info.");

    let config = Config::default();

    if !config.networking
    {
        let sdb = OfflineSandbox { config };
        let offline_ses = OfflineSession {  sandbox: sdb };
        offline_ses.run();
    }

}
