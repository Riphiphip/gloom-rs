extern crate nalgebra_glm as glm;
use std::convert::TryInto;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::{mem, os::raw::c_void, ptr};

use std::ffi::CString;

use std::collections::HashSet;

mod shader;
mod util;

use glutin::event::{
    DeviceEvent,
    ElementState::{Pressed, Released},
    Event, KeyboardInput,
    VirtualKeyCode::{self, *},
    WindowEvent,
};
use glutin::event_loop::ControlFlow;

const SCREEN_W: u32 = 800;
const SCREEN_H: u32 = 600;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //
// The names should be pretty self explanatory
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()

// == // Modify and complete the function below for the first task
// unsafe fn FUNCTION_NAME(ARGUMENT_NAME: &Vec<f32>, ARGUMENT_NAME: &Vec<u32>) -> u32 { }

unsafe fn setup_triangle_vao(verticies: &Vec<f32>, indicies: &Vec<u32>, colors:&Vec<f32>) -> u32 {
    let mut vao_id = 0;
    gl::GenVertexArrays(1, &mut vao_id);
    gl::BindVertexArray(vao_id);

    // Load verticies (vertex attrib 0)
    let mut vert_buffer_id = 0;
    gl::GenBuffers(1, &mut vert_buffer_id);
    gl::BindBuffer(gl::ARRAY_BUFFER, vert_buffer_id);

    let c_v_ptr = pointer_to_array(verticies);
    let c_v_size = byte_size_of_array(verticies);
    gl::BufferData(gl::ARRAY_BUFFER, c_v_size, c_v_ptr, gl::STATIC_DRAW);
    
    gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, 0 as *const c_void);
    gl::EnableVertexAttribArray(0);

    // Load vertex colors (vertex attrib 1)
    let mut color_buffer_id = 0;
    gl::GenBuffers(1, &mut color_buffer_id);
    gl::BindBuffer(gl::ARRAY_BUFFER, color_buffer_id);

    let c_c_ptr = pointer_to_array(colors);
    let c_c_size = byte_size_of_array(colors);
    gl::BufferData(gl::ARRAY_BUFFER, c_c_size, c_c_ptr, gl::STATIC_DRAW);
    
    gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, 0, 0 as *const c_void);
    gl::EnableVertexAttribArray(1);

    // Load indicies
    let mut element_buf_id = 0;
    gl::GenBuffers(1, &mut element_buf_id);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, element_buf_id);

    let index_v_size = byte_size_of_array(indicies);
    gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER,
        index_v_size,
        pointer_to_array(indicies),
        gl::STATIC_DRAW,
    );


    vao_id
}


fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(false)
        .with_inner_size(glutin::dpi::LogicalSize::new(SCREEN_W, SCREEN_H));
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers. This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!(
                "{}: {}",
                util::get_gl_string(gl::VENDOR),
                util::get_gl_string(gl::RENDERER)
            );
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!(
                "GLSL\t: {}",
                util::get_gl_string(gl::SHADING_LANGUAGE_VERSION)
            );
        }

        // == // Set up your VAO here
        let verticies: Vec<f32> = vec![
            // Triangle 0
             0.0,  1.0,  -1.5,
            -1.0, -1.0,  -1.5,
             1.0, -1.0,  -1.5,
             // Triangle 1
             0.0,  0.5,  -1.25,
            -0.5, -0.5,  -1.25,
             0.5, -0.5,  -1.25,
             // Triangle 2
             0.0,   0.25,  -1.0,
            -0.25, -0.25,  -1.0,
             0.25, -0.25,  -1.0,
        ];
        let indicies: Vec<u32> = vec![
            // Triangle 0    
            0, 1, 2,
            // Triangle 1
            3, 4, 5, 
            // Triangle 2
            6, 7, 8, 
        ];
        let colors: Vec<f32> = vec![
            // Triangle 0
            1.0, 0.0, 0.0, 0.5,
            1.0, 0.0, 0.0, 0.5,
            1.0, 0.0, 0.0, 0.5,
            // Triangle 1
            0.0, 1.0, 0.0, 0.5,
            0.0, 1.0, 0.0, 0.5,
            0.0, 1.0, 0.0, 0.5,
            // Triangle 2
            0.0, 0.0, 1.0, 0.5,
            0.0, 0.0, 1.0, 0.5,
            0.0, 0.0, 1.0, 0.5,
        ];
        let vao_id: u32 = unsafe { setup_triangle_vao(&verticies, &indicies, &colors) };

        // Basic usage of shader helper:
        // The example code below returns a shader object, which contains the field `.program_id`.
        // The snippet is not enough to do the assignment, and will need to be modified (outside of
        // just using the correct path), but it only needs to be called once
        //
        //     shader::ShaderBuilder::new()
        //        .attach_file("./path/to/shader.file")
        //        .link();
        let shader_program = unsafe {
            shader::ShaderBuilder::new()
                .attach_file("./shaders/simple.frag")
                .attach_file("./shaders/simple.vert")
                .link()
        };
        unsafe {
            let screen_dims_name = CString::new("screenDims").expect("Could not allocate c string");
            let screen_dims_uniform_loc =
                gl::GetUniformLocation(shader_program.program_id, screen_dims_name.as_ptr());
            gl::ProgramUniform2ui(
                shader_program.program_id,
                screen_dims_uniform_loc,
                SCREEN_W,
                SCREEN_H,
            );
        }

        // Used to demonstrate keyboard handling -- feel free to remove
        let mut _arbitrary_number = 0.0;
        
        let first_frame_time = std::time::Instant::now();
        let mut last_frame_time = first_frame_time;
        
        // Set up uniforms
        let time_uniform = shader::ShaderUniform::new(&shader_program, "iTime");
        let screen_dims_uniform = shader::ShaderUniform::new(&shader_program, "screenDims");
        let camera_uniform = shader::ShaderUniform::new(&shader_program, "camera");
        screen_dims_uniform.update2f(&[SCREEN_W as f32, SCREEN_H as f32]);
        
        let mut camera = glm::perspective(
            (SCREEN_W as f32) /(SCREEN_H as f32),
                120.0,
                1.0,
                100.0
        );
        camera_uniform.updatefmat4(&camera, false);

        let rot_amount = 0.01;

        let y_axis = glm::vec3::<f32>(0.0, 1.0, 0.0);
        let x_axis = glm::vec3::<f32>(1.0, 0.0, 0.0);
        let z_axis = glm::vec3::<f32>(0.0, 0.0, 1.0);

        let move_amount = 0.01;

        // The main rendering loop
        loop {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        VirtualKeyCode::A => {
                            _arbitrary_number += delta_time;
                            camera = glm::rotate::<f32>(&camera, -rot_amount, &y_axis);
                        }
                        VirtualKeyCode::D => {
                            _arbitrary_number -= delta_time;
                            camera = glm::rotate::<f32>(&camera, rot_amount, &y_axis);
                        }
                        VirtualKeyCode::W => {
                            camera = glm::rotate::<f32>(&camera, -rot_amount, &x_axis);
                        }
                        VirtualKeyCode::S => {
                            camera = glm::rotate::<f32>(&camera, rot_amount, &x_axis);
                        }
                        _ => {}
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {
                *delta = (0.0, 0.0);
            }


            time_uniform.update1f(first_frame_time.elapsed().as_secs_f32());
            camera_uniform.updatefmat4(&camera, false);

            unsafe {
                gl::ClearColor(0.0, 0.0, 0.0, 1.0); // moon raker, full opacity
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                // Issue the necessary commands to draw your scene here
                gl::BindVertexArray(vao_id);
                gl::EnableVertexArrayAttrib(vao_id, 0);
                shader_program.activate();
                gl::DrawElements(
                    gl::TRIANGLES,
                    (&indicies).len() as i32,
                    gl::UNSIGNED_INT,
                    0 as *const c_void,
                );
            }

            context.swap_buffers().unwrap();
        }
    });

    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events get handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: key_state,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        }
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle escape separately
                match keycode {
                    Escape => {
                        *control_flow = ControlFlow::Exit;
                    }
                    Q => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            }
            _ => {}
        }
    });
}
