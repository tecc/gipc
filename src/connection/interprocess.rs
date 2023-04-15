
macro_rules! name_onto {
    (await $func:path; $name:expr, $global:expr) => {
        name_onto!($func, ., await; $name, $global)
    };
    ($func:path $(, $suffix:tt )*; $name:expr, $global:expr) => {{
        use interprocess::local_socket::NameTypeSupport::{self, *};
        use std::path::PathBuf;
        let name = $name.as_ref();
        let global = $global;
        match NameTypeSupport::query() {
            Both | OnlyNamespaced => {
                $func(format!("@{}-gipc.sock", name))$( $suffix )*
            }
            OnlyPaths => {
                let path = if global {
                    #[cfg(not(target_family = "unix"))]
                    panic!("Non-Linux operating systems do not support global named sockets.");
                    #[cfg(target_family = "unix")]
                    PathBuf::from(
                        format!("/run/{}.sock", name)
                    )
                } else {
                    dirs::runtime_dir()
                        .or(dirs::data_local_dir())
                        .map(|v| v.join(format!("{}.sock", name)))
                        .expect("No valid path can be used")
                };
                $func(path)$( $suffix )*
            }
        }
    }};
}
pub(crate) use name_onto;