use graphics::{
    drawer::DrawerBuilder,
    light::{fatt, Light, LightProps},
    matrix::transform as tr,
    processes::{pipe_to_magick, wait_for_magick},
    vector::Vec3,
    PPMImg, RGB,
};

// # compilation:
// cargo run --release

fn main() {
    let mut magick = pipe_to_magick(vec![
        "-delay",
        &format!("{}", 3.8),
        "ppm:-",
        "fakemotion-d3.8s2.gif",
    ]);

    let lights_for_around: Vec<Light> = vec![
        Light::Ambient(RGB {
            red: 100,
            green: 100,
            blue: 50,
        }),
        // This should be yellow
        Light::Point {
            color: RGB {
                red: 252,
                green: 219,
                blue: 3,
            },
            location: Vec3(250., 250., 0.),
            fatt: fatt::no_effect,
        },
    ];

    let lights_for_center: Vec<Light> = vec![Light::Ambient(RGB {
        red: 255,
        green: 253,
        blue: 237,
    })];

    // colors!
    // let default_fg = drawer.get_fg_color();
    let light_yellow = RGB::new(245, 236, 66);
    // let blue = RGB::new(66, 135, 245);
    let magenta = RGB::new(239, 66, 245);
    // let purple = RGB::new(209, 66, 245);
    let brown = RGB::new(212, 143, 78);

    let center_props = LightProps {
        ka: Vec3(255., 255., 255.),
        kd: Vec3(0.3425, 0.234, 0.23523),
        ks: Vec3(0.24, 0.24, 0.24),
        intensities: Vec3::ZEROS,
        ns: 10.,
    };

    let mut drawer = DrawerBuilder::new(PPMImg::new(500, 500, 255))
        .with_writer(Box::new(magick.stdin.take().unwrap()))
        .with_lights(lights_for_around.clone())
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

                drawer.env_lights = lights_for_center.clone();
                drawer.add_sphere((0., 0., 0.), 40., Some(&center_props));
                drawer.env_lights = lights_for_around.clone();
            }
            drawer.pop_matrix();

            // draw the torus around the sphere, rotate on rot
            drawer.push_matrix();
            {
                drawer.transform_by(&(tr::rotatez(45.) * tr::rotatey(rot as f64)));
                drawer.add_torus((0., 0., 0.), 10., 70., Some(&LightProps::BRASS));
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
                    drawer.fg_color = magenta;
                    drawer.add_sphere((0., 0., 0.), 30., Some(&LightProps::POLISHED_COPPER));
                }
                drawer.pop_matrix();

                // draw 1st satellite
                drawer.push_matrix();
                {
                    drawer.transform_by(&tr::rotatex(rot as f64 * 3.)); // <- var here
                    drawer.transform_by(&tr::mv(0., 80., 0.));
                    drawer.fg_color = light_yellow;
                    drawer.add_sphere((0., 0., 0.), 20., Some(&LightProps::POLISHED_SILVER));

                    // drawer.transform_by(&);
                    drawer.transform_by(
                        &(tr::rotatey(rot as f64 * 4.)
                            * tr::rotatex(rot as f64 * 4.)
                            * tr::rotatez(-45.)),
                    );
                    drawer.fg_color = brown;
                    drawer.add_torus((0., 0., 0.), 5., 40., Some(&LightProps::GOLD));
                }
                drawer.pop_matrix();

                // 2nd satellite
                drawer.push_matrix();
                {
                    drawer.transform_by(&tr::rotatex(rot as f64 * 3.));
                    drawer.transform_by(&tr::mv(0., -80., 0.));
                    drawer.add_sphere((0., 0., 0.), 20., Some(&LightProps::BRASS));
                }
                drawer.pop_matrix();
            }
            drawer.pop_matrix();

            drawer.push_matrix();
            {
                drawer.transform_by(&tr::rotatez(rot as f64)); // <- var here
                drawer.transform_by(&tr::mv(-200., 0., 0.));

                drawer.add_sphere((0., 0., 0.), 30., Some(&LightProps::POLISHED_GOLD));

                drawer.push_matrix();
                {
                    drawer.transform_by(&tr::rotatez(-rot as f64 * 3.));
                    drawer.transform_by(&tr::mv(80., 0., 0.));

                    drawer.add_sphere((0., 0., 0.), 20., Some(&LightProps::SILVER));
                }
                drawer.pop_matrix();

                drawer.push_matrix();
                {
                    drawer.transform_by(&tr::rotatez(-rot as f64 * 3.));
                    drawer.transform_by(&tr::mv(-80., 0., 0.));

                    drawer.add_sphere((0., 0., 0.), 20., None);
                }
                drawer.pop_matrix();
            }
            drawer.pop_matrix();
        }
        drawer.pop_matrix();

        drawer.flush().expect("Error writing img data");

        drawer.clear();
    }

    drawer.finish().expect("Error writing img data");

    println!("Waiting for convert/magick to exit...");
    println!("convert/magick {}", wait_for_magick(magick));
}
