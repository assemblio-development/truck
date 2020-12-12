mod app;
use app::*;
use std::f64::consts::PI;
use truck_modeling::*;
use truck_platform::*;
use truck_rendimpl::*;
use wgpu::*;

const N: usize = 5;
const BACKGROUND: [f64; 4] = [45.0 / 255.0, 36.0 / 255.0, 42.0 / 255.0, 1.0];
const BOXCOLOR: [f64; 4] = [208.0 / 255.0, 176.0 / 255.0, 107.0 / 255.0, 1.0];

struct MyApp {
    scene: Scene,
    instances: Vec<ShapeInstance>,
    matrices: Vec<Matrix4>,
}

impl App for MyApp {
    fn init(handler: &DeviceHandler) -> MyApp {
        let side_length = (N + 1) as f64 * 1.5;
        let camera_dist = side_length / 2.0 / (PI / 8.0).tan();
        let a = side_length / 2.0;
        let b = camera_dist / 2.0;
        let scene_desc = SceneDescriptor {
            camera: Camera::perspective_camera(
                Matrix4::from_translation(camera_dist * Vector3::unit_z()),
                Rad(PI / 4.0),
                0.1,
                100.0,
            ),
            lights: vec![
                Light {
                    position: Point3::new(-a, -a, b),
                    color: Vector3::new(0.5, 0.5, 0.5),
                    light_type: LightType::Point,
                },
                Light {
                    position: Point3::new(-a, a, b),
                    color: Vector3::new(0.5, 0.5, 0.5),
                    light_type: LightType::Point,
                },
                Light {
                    position: Point3::new(a, -a, b),
                    color: Vector3::new(0.5, 0.5, 0.5),
                    light_type: LightType::Point,
                },
                Light {
                    position: Point3::new(a, a, b),
                    color: Vector3::new(0.5, 0.5, 0.5),
                    light_type: LightType::Point,
                },
            ],
            background: Color {
                r: BACKGROUND[0],
                g: BACKGROUND[1],
                b: BACKGROUND[2],
                a: BACKGROUND[3],
            },
            ..Default::default()
        };
        let mut scene = Scene::new(handler.clone(), &scene_desc);
        let v = builder::vertex(Point3::new(-0.5, -0.5, -0.5));
        let e = builder::tsweep(&v, Vector3::unit_x());
        let f = builder::tsweep(&e, Vector3::unit_y());
        let cube = builder::tsweep(&f, Vector3::unit_z());
        let instance = scene.create_instance(&cube, &Default::default());
        let mut matrices = Vec::new();
        let instances: Vec<_> = (0..N)
            .flat_map(move |i| (0..N).map(move |j| (i, j)))
            .map(|(i, j)| {
                let mut instance = instance.clone();
                let (s, t) = (i as f64 / (N - 1) as f64, j as f64 / (N - 1) as f64);
                let matrix = Matrix4::from_translation(Vector3::new(
                    1.5 * (i + 1) as f64 - side_length / 2.0,
                    1.5 * (j + 1) as f64 - side_length / 2.0,
                    0.0,
                ));
                matrices.push(matrix);
                *instance.descriptor_mut() = InstanceDescriptor {
                    matrix,
                    material: Material {
                        albedo: Vector4::from(BOXCOLOR),
                        reflectance: s,
                        roughness: t,
                        ambient_ratio: 0.02,
                    },
                    ..Default::default()
                };
                instance
            })
            .collect();
        instances.iter().for_each(|shape| {
            scene.add_objects(&shape.render_faces());
        });
        MyApp {
            scene,
            instances,
            matrices,
        }
    }
    fn update(&mut self, _: &DeviceHandler) {
        let time = self.scene.elapsed().as_secs_f64();
        for (i, shape) in self.instances.iter_mut().enumerate() {
            let axis = if i % 2 == 0 {
                (-1.0_f64).powi(i as i32 / 2) * Vector3::unit_y()
            } else {
                -(-1.0_f64).powi(i as i32 / 2) * Vector3::unit_x()
            };
            shape.descriptor_mut().matrix =
                self.matrices[i] * Matrix4::from_axis_angle(axis, Rad(time * PI / 2.0));
            self.scene.update_bind_groups(&shape.render_faces());
        }
    }
    fn render(&mut self, frame: &SwapChainFrame) { self.scene.render_scene(&frame.output.view); }
}

fn main() { MyApp::run() }