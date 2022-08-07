use crate::vulkan::Context;

use std::ffi::OsStr;
use std::fs::File;
use std::{io::Read, path::Path};

use ash::{prelude::*, vk};

use cstr::cstr;
use shaderc;

pub type Kind = shaderc::ShaderKind;
fn shader_kind_to_vk_stage(kind: Kind) -> vk::ShaderStageFlags {
    match kind {
        Kind::Vertex => vk::ShaderStageFlags::VERTEX,
        Kind::Fragment => vk::ShaderStageFlags::FRAGMENT,
        _ => todo!(),
    }
}

pub struct Shader<'a> {
    pub module: vk::ShaderModule,
    pub path: &'a Path,
    pub kind: Kind,
}

impl Shader<'_> {
    #[doc = "# Errors"]
    #[doc = "Errors if internal ash functions fail"]
    pub fn new<'a>(ctx: &Context, path: &'a Path) -> VkResult<Shader<'a>> {
        let mut file = match File::open(path) {
            Err(e) => panic!("Couldn't open shader file {} because {}", path.display(), e),
            Ok(file) => file,
        };

        let mut shader_code: Vec<u8> = Vec::new();
        if let Err(e) = file.read_to_end(&mut shader_code) {
            panic!(
                "Failed to read shader file {} because {}",
                path.display(),
                e
            );
        }

        let shader_code = Shader::compile_shader(path, &shader_code);
        let module_info = vk::ShaderModuleCreateInfo::builder().code(&shader_code);
        let module = unsafe { ctx.device.create_shader_module(&module_info, None)? };

        println!("Compiled shader {}", path.display());

        Ok(Shader {
            module,
            path,
            kind: Shader::get_kind(path),
        })
    }

    #[must_use]
    pub fn get_stage(&self) -> vk::PipelineShaderStageCreateInfoBuilder {
        vk::PipelineShaderStageCreateInfo::builder()
            .stage(shader_kind_to_vk_stage(self.kind))
            .name(cstr!("main"))
            .module(self.module)
    }

    fn compile_shader(path: &Path, code: &[u8]) -> Vec<u32> {
        if path.extension() == Some(OsStr::new("spv")) {
            return code.iter().map(|a| u32::from(*a)).collect();
        }

        let compiler = match shaderc::Compiler::new() {
            None => panic!("ShaderC compiler initialisation failed"),
            Some(compiler) => compiler,
        };

        let filename = match path.file_name() {
            None => panic!("Shader file {} has no file name", path.display()),
            Some(filename) => match filename.to_str() {
                None => panic!("Converting filename to &str failed"),
                Some(filename) => filename,
            },
        };

        let shader_code = match std::str::from_utf8(code) {
            Err(e) => panic!("Unable to convert shader code to string because {}", e),
            Ok(code) => code,
        };

        return match compiler.compile_into_spirv(
            shader_code,
            Shader::get_kind(path),
            filename,
            "main",
            None,
        ) {
            Err(e) => panic!("Failed to compile shader because {}", e),
            Ok(code) => code.as_binary().to_vec(),
        };
    }

    fn get_kind(path: &Path) -> Kind {
        let mut p = path;

        if path.extension() == Some(OsStr::new("spv")) {
            p = Path::new(match path.file_stem() {
                None => panic!("File {} has no stem", path.display()),
                Some(stem) => stem,
            });
        };

        match p.extension() {
            None => panic!("Shader file {} has no extension", p.display()),
            Some(ext) => match ext.to_str() {
                Some("vert") => Kind::Vertex,
                Some("frag") => Kind::Fragment,
                _ => panic!("Invalid extension on shader file {}", p.display()),
            },
        }
    }
}
