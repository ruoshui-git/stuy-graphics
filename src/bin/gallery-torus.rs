use graphics::{
    drawer::DrawerBuilder,
    light::{fatt, Light},
    matrix::transform,
    processes::pipe_to_magick,
    vector::Vec3, PPMImg, RGB,
};

fn main() {
    let mut magick = pipe_to_magick(vec!["ppm:-", "img.gif"]);

    let mut magick_in = magick.stdin.take().unwrap();

    let lights = vec![
        Light::Ambient(RGB {
            red: 100,
            green: 100,
            blue: 50,
        }),
        Light::Point {
            color: RGB {
                red: 252,
                green: 219,
                blue: 3,
            },
            location: Vec3(250., 400., 0.),
            fatt: fatt::no_effect,
        },
        Light::Point {
            color: RGB {
                red: 44,
                green: 156,
                blue: 73,
            },
            location: Vec3(250., 100., 0.),
            fatt: fatt::no_effect,
        },
        Light::Point {
            color: RGB {
                red: 66, 
                green: 147, 
                blue: 245,
            },
            location: Vec3(250., 250., 0.),
            fatt: fatt::no_effect,
        }
    ];

    let mut drawer = DrawerBuilder::new(PPMImg::new(500, 500, 250))
        .with_lights(lights)
        .build();
    drawer.transform_by(&transform::mv(250., 250., 0.));
    for rot in (0..360).step_by(5) {
        drawer.push_matrix();
        drawer.transform_by(&transform::rotatex(rot as f64));
        drawer.add_torus((0., 0., 0.), 30., 100., None);

        drawer
            .write_to_buf(&mut magick_in)
            .expect("error writing to imagemagick");

        drawer.clear();
        drawer.pop_matrix();
    }
    drop(magick_in);

    println!("Waiting for convert/magick to exit...");
    let output = magick.wait().expect("Failed to wait on convert/magick");
    println!("convert/magick {}", output);
}
