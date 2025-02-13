use gl;
use std::{ffi::CString, path::Path, ptr, str};

pub struct Shader {
    pub program_id: u32,
}

pub struct ShaderBuilder {
    program_id: u32,
    shaders: Vec<u32>,
}

pub struct ShaderUniform {
    location: i32,
    program_id: u32,
}

#[allow(dead_code)]
pub enum ShaderType {
    Vertex,
    Fragment,
    TessellationControl,
    TessellationEvaluation,
    Geometry,
}

impl Shader {
    // Make sure the shader is active before calling this
    pub unsafe fn get_uniform_location(&self, name: &str) -> i32 {
        let name_cstr = CString::new(name).expect("CString::new failed");
        gl::GetUniformLocation(self.program_id, name_cstr.as_ptr())
    }

    pub unsafe fn activate(&self) {
        gl::UseProgram(self.program_id);
    }
}

impl Into<gl::types::GLenum> for ShaderType {
    fn into(self) -> gl::types::GLenum {
        match self {
            ShaderType::Vertex => gl::VERTEX_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
            ShaderType::TessellationControl => gl::TESS_CONTROL_SHADER,
            ShaderType::TessellationEvaluation => gl::TESS_EVALUATION_SHADER,
            ShaderType::Geometry => gl::GEOMETRY_SHADER,
        }
    }
}

impl ShaderType {
    fn from_ext(ext: &std::ffi::OsStr) -> Result<ShaderType, String> {
        match ext.to_str().expect("Failed to read extension") {
            "vert" => Ok(ShaderType::Vertex),
            "frag" => Ok(ShaderType::Fragment),
            "tcs" => Ok(ShaderType::TessellationControl),
            "tes" => Ok(ShaderType::TessellationEvaluation),
            "geom" => Ok(ShaderType::Geometry),
            e => Err(e.to_string()),
        }
    }
}

impl ShaderBuilder {
    pub unsafe fn new() -> ShaderBuilder {
        ShaderBuilder {
            program_id: gl::CreateProgram(),
            shaders: vec![],
        }
    }

    pub unsafe fn attach_file(self, shader_path: &str) -> ShaderBuilder {
        let path = Path::new(shader_path);
        if let Some(extension) = path.extension() {
            let shader_type =
                ShaderType::from_ext(extension).expect("Failed to parse file extension.");
            let shader_src = std::fs::read_to_string(path)
                .expect(&format!("Failed to read shader source. {}", shader_path));
            self.compile_shader(&shader_src, shader_type)
        } else {
            panic!(
                "Failed to read extension of file with path: {}",
                shader_path
            );
        }
    }

    pub unsafe fn compile_shader(
        mut self,
        shader_src: &str,
        shader_type: ShaderType,
    ) -> ShaderBuilder {
        let shader = gl::CreateShader(shader_type.into());
        let c_str_shader = CString::new(shader_src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str_shader.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        if !self.check_shader_errors(shader) {
            panic!("Shader failed to compile.");
        }

        self.shaders.push(shader);

        self
    }

    unsafe fn check_shader_errors(&self, shader_id: u32) -> bool {
        let mut success = i32::from(gl::FALSE);
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1);
        gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut success);
        if success != i32::from(gl::TRUE) {
            gl::GetShaderInfoLog(
                shader_id,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut gl::types::GLchar,
            );
            println!(
                "ERROR::Shader Compilation Failed!\n{}",
                String::from_utf8_lossy(&info_log)
            );
            return false;
        }
        true
    }

    unsafe fn check_linker_errors(&self) -> bool {
        let mut success = i32::from(gl::FALSE);
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1);
        gl::GetProgramiv(self.program_id, gl::LINK_STATUS, &mut success);
        if success != i32::from(gl::TRUE) {
            gl::GetProgramInfoLog(
                self.program_id,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut gl::types::GLchar,
            );
            println!(
                "ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}",
                String::from_utf8_lossy(&info_log)
            );
            return false;
        }
        true
    }

    #[must_use = "The shader program is useless if not stored in a variable."]
    pub unsafe fn link(self) -> Shader {
        for &shader in &self.shaders {
            gl::AttachShader(self.program_id, shader);
        }
        gl::LinkProgram(self.program_id);

        // todo:: use this to make safer abstraction
        self.check_linker_errors();

        for &shader in &self.shaders {
            gl::DeleteShader(shader);
        }

        Shader {
            program_id: self.program_id,
        }
    }
}

impl ShaderUniform {
    pub fn new(program: &Shader, uniform_name: &str) -> ShaderUniform {
        let uniform_string = CString::new(uniform_name)
            .expect("Could not convert uniform name to c_string");

        let uniform_loc =
            unsafe { gl::GetUniformLocation(program.program_id, uniform_string.as_ptr()) };
        ShaderUniform {
            program_id: program.program_id,
            location: uniform_loc,
        }
    }

    pub fn update1f(&self, value: f32) {
        unsafe { gl::ProgramUniform1f(self.program_id, self.location, value) };
    }

    pub fn update2f(&self, value: &[f32; 2]) {
        let v0 = value[0];
        let v1 = value[1];
        unsafe { gl::ProgramUniform2f(self.program_id, self.location, v0, v1) };
    }

    pub fn update3f(&self, value: &[f32; 3]) {
        let v0 = value[0];
        let v1 = value[1];
        let v2 = value[2];
        unsafe { gl::ProgramUniform3f(self.program_id, self.location, v0, v1, v2) };
    }

    pub fn update4f(&self, value: &[f32; 4]) {
        let v0 = value[0];
        let v1 = value[1];
        let v2 = value[2];
        let v3 = value[3];
        unsafe { gl::ProgramUniform4f(self.program_id, self.location, v0, v1, v2, v3) };
    }

    pub fn updatefmat2(&self, value: &glm::Mat2, transpose: bool){
        let mat_ptr = value.as_ptr();
        unsafe {
            gl::ProgramUniformMatrix2fv(
                self.program_id,
                self.location,
                1,
                transpose as u8,
                mat_ptr
            );
        }
    }

    pub fn updatefmat4(&self, value: &glm::Mat4, transpose: bool){
        let mat_ptr = value.as_ptr();
        unsafe {
            gl::ProgramUniformMatrix4fv(
                self.program_id,
                self.location,
                1,
                transpose as u8,
                mat_ptr
            );
        }
    }

}
