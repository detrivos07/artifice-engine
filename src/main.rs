extern crate gl;
extern crate glfw;

mod io;

fn main() {
    let mut window = crate::io::GlfwWindow::new(800, 600, "Artfice Engine V0.0.0");

    // Define vertex data for a triangle
    let vertices: [f32; 9] = [
        0.0, 0.5, 0.0, // top
        -0.5, -0.5, 0.0, // bottom left
        0.5, -0.5, 0.0, // bottom right
    ];

    // Set up OpenGL objects
    let (vertex_array, vertex_buffer, shader_program) = unsafe {
        // Create a vertex array object
        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        // Create a vertex buffer object
        let mut vbo = 0;
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as isize,
            vertices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        // Configure vertex attributes
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (3 * std::mem::size_of::<f32>()) as i32,
            std::ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        // Create and compile the vertex shader
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let vertex_shader_source = std::ffi::CString::new(
            "#version 330 core
            layout (location = 0) in vec3 aPos;

            void main() {
                gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
            }",
        )
        .unwrap();
        gl::ShaderSource(
            vertex_shader,
            1,
            &vertex_shader_source.as_ptr(),
            std::ptr::null(),
        );
        gl::CompileShader(vertex_shader);
        check_shader_compilation(vertex_shader);

        // Create and compile the fragment shader
        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let fragment_shader_source = std::ffi::CString::new(
            "#version 330 core
            out vec4 FragColor;

            void main() {
                FragColor = vec4(1.0, 0.5, 0.2, 1.0);
            }",
        )
        .unwrap();
        gl::ShaderSource(
            fragment_shader,
            1,
            &fragment_shader_source.as_ptr(),
            std::ptr::null(),
        );
        gl::CompileShader(fragment_shader);
        check_shader_compilation(fragment_shader);

        // Create and link the shader program
        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
        check_program_linking(shader_program);

        // Delete the shaders as they're linked into the program and no longer needed
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        (vao, vbo, shader_program)
    };

    // Main render loop
    while !window.should_close() {
        window.update();

        // Render
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw the triangle
            gl::UseProgram(shader_program);
            gl::BindVertexArray(vertex_array);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }

    // Clean up
    unsafe {
        gl::DeleteVertexArrays(1, &vertex_array);
        gl::DeleteBuffers(1, &vertex_buffer);
        gl::DeleteProgram(shader_program);
    }
}

unsafe fn check_shader_compilation(shader: u32) {
    let mut success = 0;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    if success == 0 {
        let mut log_length = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_length);
        let mut log = Vec::with_capacity(log_length as usize);
        log.set_len(log_length as usize);
        gl::GetShaderInfoLog(
            shader,
            log_length,
            std::ptr::null_mut(),
            log.as_mut_ptr() as *mut i8,
        );
        let log_str = std::str::from_utf8(&log).unwrap_or("Unknown error");
        println!("Shader compilation failed: {}", log_str);
    }
}

unsafe fn check_program_linking(program: u32) {
    let mut success = 0;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
    if success == 0 {
        let mut log_length = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_length);
        let mut log = Vec::with_capacity(log_length as usize);
        log.set_len(log_length as usize);
        gl::GetProgramInfoLog(
            program,
            log_length,
            std::ptr::null_mut(),
            log.as_mut_ptr() as *mut i8,
        );
        let log_str = std::str::from_utf8(&log).unwrap_or("Unknown error");
        println!("Program linking failed: {}", log_str);
    }
}
