use std::{env, f64::consts::{E, PI}, process};

use graphics::{
    drawer::DrawerBuilder,
    light::{Light, LightProps},
    matrix::transform as tr,
    processes::{pipe_to_magick, wait_for_magick},
    vector::Vec3,
    PPMImg, RGB,
};

// # compilation:
// cargo run --release

fn main() {
    let args: Vec<String> = env::args().collect();

    let default_fname = "fakemotion-d3.8s2.gif";
    let (filename, _use_magick) = match args.len() {
        1 => (default_fname, true),
        2 => (args[1].as_str(), true),
        _ => {
            eprintln!("./program [output-name] [-no-m]");
            process::exit(1);
        }
    };

    let mut magick = pipe_to_magick(vec!["-delay", &format!("{}", 6), "ppm:-", filename]);

    let lights_for_around: Vec<Light> = vec![
        Light::Ambient(RGB::new(100, 100, 50)),
        // This should be yellow
        Light::Point {
            color: RGB::new(255, 234, 99),
            // color: RGB::new(255, 234, 99),
            // color: RGB::WHITE,
            location: Vec3(250., 250., 0.),
            fatt: |distance: f64| 50_000. / (distance * distance),
        },
    ];

    let illuminator0 = Light::Point {
        color: RGB::new(222, 205, 20),
        location: Vec3(0., 0., 0.),
        fatt: |d| 6000. / (d * d),
    };

    let illuminator1 = Light::Point {
        // color: RGB::WHITE,
            color: RGB::new(252, 219, 3),
        // position will be transformed later
        location: Vec3(0., 0., 0.),
        fatt: |_| 2.,
    };

    let lights_for_center: Vec<Light> = vec![Light::Ambient(RGB::new(255, 253, 237))];

    let lights_for_ilum0 = &[Light::Ambient(RGB::new(255, 100, 51))];

    let lights_for_ilum1 = &[Light::Ambient(RGB::new(252, 219, 3))];

    let center_props = LightProps {
        ka: Vec3(0.98, 0.98, 0.98),
        kd: Vec3(0.3425, 0.234, 0.23523),
        ks: Vec3(0.24, 0.24, 0.24),
        intensities: Vec3::ZEROS,
        ns: 10.,
    };

    let mut center_ring_props = LightProps::BRASS;
    center_ring_props.ks += -0.6;
    center_ring_props.kd += -0.3;
    // center_ring_props.ns -= 20.;

    let mut ilum0_props = LightProps::POLISHED_COPPER;
    ilum0_props.ka += 1.;
    ilum0_props.kd += 3.;
    ilum0_props.ks += 5.; //Vec3(0.780594, 0.423257, 0.2695701);
    ilum0_props.ns += 20.;

    let mut ilum1_props = LightProps::POLISHED_GOLD;
    ilum1_props.ka += 0.8;
    ilum1_props.kd += 1.;
    ilum1_props.ks += 2.;
    ilum1_props.ns += 10.;

    let mut drawer = DrawerBuilder::new(PPMImg::new(500, 500, 255))
        .with_writer(Box::new(magick.stdin.take().unwrap()))
        .with_lights(lights_for_around)
        .build();

    for rot in (0..360).into_iter().step_by(2) {
        // let mut stack: Vec<Matrix> = Vec::<Matrix>::new_stack();

        // moving to the center
        drawer.push_matrix();
        {
            drawer.transform_by(&tr::mv(250., 250., 0.));
            // drawer.transform_by(&tr::rotatex(rot as f64));

            // drawing center sphere, rotate on rot
            drawer.push_matrix();
            {
                drawer.transform_by(
                    &(tr::rotatex(rot as f64) * tr::rotatey(rot as f64) * tr::rotatez(rot as f64)),
                );

                let tmp_lights = drawer.env_lights;
                drawer.env_lights = lights_for_center.clone();
                drawer.add_sphere((0., 0., 0.), 40., Some(&center_props));
                drawer.env_lights = tmp_lights;
            }
            drawer.pop_matrix();

            // draw the torus around the sphere, rotate on rot
            drawer.push_matrix();
            {
                drawer.transform_by(&(tr::rotatez(45.) * tr::rotatey(rot as f64)));
                drawer.add_torus((0., 0., 0.), 10., 70., Some(&center_ring_props));
            }
            drawer.pop_matrix();

            // move away from center, draw first orbit
            drawer.push_matrix();
            {
                // remember: transform_top needs to take the transformation in the opposite direction
                drawer.transform_by(&tr::rotatez(rot as f64)); // <- var here
                drawer.transform_by(&tr::mv(150., 0., 0.));

                drawer.push_matrix();
                {
                    drawer.transform_by(&tr::rotatex(rot as f64));
                    drawer.transform_by(&tr::rotatey(rot as f64));

                    drawer.env_lights.extend_from_slice(lights_for_ilum0);
                    drawer.add_sphere((0., 0., 0.), 30., Some(&ilum0_props));
                    for _ in 0..lights_for_ilum0.len() {
                        drawer.env_lights.pop();
                    }
                }
                drawer.pop_matrix();

                // Lights for the first group of satellites
                let mut moving = illuminator0;
                moving.transform_by(drawer.get_top_matrix());
                drawer.env_lights.push(moving);


                // the first two satellites will rotate a bit
                drawer.transform_by(&tr::rotatex(rot as f64 * 3.)); // <- var here
                drawer.transform_by(&tr::rotatez((rot as f64 * std::f64::consts::PI / (36.)).sin() * 30.));
                
                // draw 1st satellite
                drawer.push_matrix();
                {
                    drawer.transform_by(&tr::mv(0., 80., 0.));
                    // drawer.fg_color = light_yellow;
                    drawer.add_sphere((0., 0., 0.), 20., Some(&LightProps::POLISHED_SILVER));

                    // drawer.transform_by(&);
                    drawer.transform_by(
                        &(tr::rotatey(rot as f64 * 4.)
                            * tr::rotatex(rot as f64 * 4.)
                            * tr::rotatez(-45.)),
                    );
                    // drawer.fg_color = brown;
                    drawer.add_torus((0., 0., 0.), 5., 40., Some(&LightProps::GOLD));
                }
                drawer.pop_matrix();

                // 2nd satellite
                drawer.push_matrix();
                {
                    drawer.transform_by(&tr::mv(0., -80., 0.));
                    drawer.add_sphere((0., 0., 0.), 20., Some(&LightProps::TURQUOISE));
                }
                drawer.pop_matrix();

                drawer.env_lights.pop();
            }
            drawer.pop_matrix();

            drawer.push_matrix();
            {
                drawer.transform_by(&tr::rotatez(rot as f64)); // <- var here
                drawer.transform_by(&tr::mv(-200., 0., 0.));

                drawer.env_lights.extend_from_slice(lights_for_ilum1);
                drawer.add_sphere((0., 0., 0.), 30., Some(&ilum1_props));
                for _ in 0..lights_for_ilum1.len() {
                    drawer.env_lights.pop();
                }

                // the moving light should be transformed here
                let mut moving = illuminator1;
                moving.transform_by(drawer.get_top_matrix());
                drawer.env_lights.push(moving);

                let fun =  10. * E.powf(-(((rot - 180) as f64 / (30. * PI)).powi(2)))-0.60624944523;

                drawer.transform_by(&tr::rotatez(-3. * rot as f64));

                let distance = 80. + (rot as f64).sin() * 3. * fun;
                drawer.push_matrix();
                {
                    drawer.transform_by(&tr::mv(distance, 0., 0.));

                    drawer.add_sphere((0., 0., 0.), 20., Some(&LightProps::JADE));
                }
                drawer.pop_matrix();

                drawer.push_matrix();
                {
                    // drawer.transform_by(&tr::rotatez(-rot as f64 * 3.));
                    drawer.transform_by(&tr::mv(-distance, 0., 0.));
                    drawer.add_sphere((0., 0., 0.), 20., Some(&LightProps::PEARL));
                }
                drawer.pop_matrix();

                drawer.env_lights.pop();
            }
            drawer.pop_matrix();

            // moving light removed
            // lights_for_around.pop();
        }
        drawer.pop_matrix();

        drawer.flush().expect("Error writing img data");

        drawer.clear();
    }

    drawer.finish().expect("Error writing img data");

    println!("Waiting for convert/magick to exit...");
    println!("convert/magick {}", wait_for_magick(magick));
    println!("File name: {}", filename);
}
