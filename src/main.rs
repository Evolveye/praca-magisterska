mod simulation;
mod rendering;
mod world;
mod window_manager;

use simulation::Simulation;

fn main() {
    let mut simulation = Simulation::new().unwrap();
    simulation.run_window_event_loop();
    // let mut renderer = rendering::Renderer::new().unwrap();
    // world.render();
    // renderer.load_cube();
    // renderer.load_model_from_sources( "src/rendering/resources/barrel.obj", "src/rendering/resources/barrel.png" );

    // let fragment = world::visible_fragment::VisibleFragment::new();

    // let x = read_i16( "Podaj X" );
    // let y = read_i16( "Podaj Y" );
    // let z = read_i16( "Podaj Z" );

    // fragment.print_dimensions();
    // fragment.chunks[ 0 ][ 0 ].print_dimensions();
    // fragment.chunks[ 0 ][ 0 ].blocks[ 0 ][ 0 ][ 0 ].print();
    // fragment.chunks[ 0 ][ 0 ].blocks[ y as usize ][ x as usize ][ z as usize ].print();
}

// fn read_i16( label:&str ) -> i16 {
//     let mut num_str = String::new();

//     loop {
//         println!( "{label}" );

//         io::stdin()
//             .read_line(&mut num_str)
//             .expect("Failed to read line");

//         if let Ok(num) = num_str.trim().parse::<i16>() {
//             return num;
//         } else {
//             println!( "Your data: '{}'", num_str )
//         }
//     }
// }
