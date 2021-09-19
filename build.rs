use std::collections::HashSet;
use std::env::{var, set_var};
use std::error::Error;
use std::fs::{ReadDir, read_dir};
use std::path::PathBuf;
use prost_build::{Config, ServiceGenerator};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=protobuf");
    println!("cargo:rerun-if-env-changed=PROTOC");
    println!("cargo:rerun-if-env-changed=PROTOC_INCLUDE");

    let mut files = vec![];
    collect_files(&mut files, read_dir("./protobuf")?)?;

    if var("PROTOC").is_err() && var("TARGET").map(|target| target.contains("musl")).unwrap_or_default() {
        // Don't use provided binaries on musl platforms. Use the system protoc instead.
        set_var("PROTOC", which::which("protoc")?);
    }

    fn collect_files(files: &mut Vec<PathBuf>, dir: ReadDir) -> Result<(), Box<dyn Error>> {
        for f in dir {
            let f = f?;
            let ty = f.file_type()?;
            let name = f.file_name();
            if ty.is_dir() {
                collect_files(files, read_dir(f.path())?)?;
            } else if ty.is_file() && !name.to_string_lossy().starts_with("legacy") {
                files.push(f.path());
            }
        }
        Ok(())
    }

    Config::new().service_generator(Box::new(FlowServiceGenerator)).compile_protos(&files, &["protobuf/"])?;

    Ok(())
}

pub struct FlowServiceGenerator;

impl ServiceGenerator for FlowServiceGenerator {
    fn generate(&mut self, service: prost_build::Service, buf: &mut String) {
        let service_name = service.proto_name;
        if service_name != "AccessAPI" { return }
        let mut tys = HashSet::new();
        for method in service.methods {
            method.comments.append_with_indent(0, buf);
            let input_ty = method.input_type.split('.').last().unwrap().to_owned();
            buf.push_str(&format!(
"impl crate::requests::FlowRequest<{output_ty}> for crate::protobuf::access::{input_ty} {{
    const PATH: &'static str = \"/flow.access.{service_name}/{method_name}\";
}}", 
                input_ty = input_ty,
                output_ty = method.output_type.split('.').last().unwrap(),
                service_name = service_name,
                method_name = method.proto_name,
            ));
            tys.insert(input_ty);
        }
        for ty in tys {
            buf.push_str(&format!(
                "impl crate::requests::private::Sealed for crate::protobuf::access::{input_ty} {{}}", 
                input_ty = ty,
            ))
        }
    }
}