use std::thread;
use std::path::Path;
use std::fs::File;
use std::io::BufWriter;

#[derive(Copy,Clone)]
struct Complex
{
    real : f64,
    complex : f64
}

impl Complex
{
    fn squared(&self) -> Complex
    {
        let mut val = Complex
        {
            real : 0.0,
            complex : 0.0
        };

        val.real = self.real.powf(2.0) - self.complex.powf(2.0);
        val.complex = 2.0*self.real*self.complex;

        return val
    }

    fn modulus_no_square_root (&self) -> f64
    {
        return (self.real.powf(2.0) + self.complex.powf(2.0)).sqrt();
    }
}

use std::ops::Add;

impl Add for Complex
{
    type Output = Self;

    fn add (self, other : Self) -> Self
    {
        Self {real : self.real + other.real, complex : self.complex + other.complex}
    }
}

fn assign_color_for_pixel (h : f64, w : f64, height : f64, width : f64, iterations : u32) -> u8
{

    let num = Complex{
        real : 3.0/width*w -1.5,
        complex : 3.0/height*h -1.5,
    };
    
    let how_covergent = convergence_test (num, iterations);

    return (how_covergent*256.0) as u8
}

fn convergence_test (number : Complex, iterations : u32) -> f64
{
    let mut z = Complex
    {
        real : 0.0,
        complex : 0.0
    };

    let mut i = 0;

    while i < iterations
    {
        z = z.squared() + number;
        i +=1;

        if z.modulus_no_square_root() > 4.4
        {
            return i as f64/iterations as f64;
        }
    }

    return 1.0;
}

fn print_mandlebrot_set (width : u32, height : u32, vec : Vec<u8>)
{
    let path = Path::new("image.png");
    let file = File::create(path).unwrap();

    let ref mut w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(&vec[..]).unwrap(); // Save
}

fn generate_mandlebrot_set (width : u32, height : u32, iterations : u32) -> Vec<u8>
{
    let thread_count_y = 2;
    let thread_count_x = 2;

    let chunkx = width/thread_count_x;
    let chunky = height/thread_count_y;

    let data = vec![0u8; (width*height) as usize];

    let lock = std::sync::Arc::new(std::sync::Mutex::new(data));

    let mut handles = Vec::new();

    for i in 0..thread_count_x
    {
        for j in 0..thread_count_y
        {
            let startx = chunkx*i;
            let endx = startx + chunkx;

            let starty = chunky*j;
            let endy = starty + chunky;

            let clone = lock.clone();
            let handle = thread::spawn(move || 
            {
                for h in starty..endy
                {
                    for w in startx..endx 
                    {
                        let color = assign_color_for_pixel (h as f64,w as f64, height as f64, width as f64, iterations);
                        
                        let mut v = clone.lock().unwrap();

                        v[(w + width*h) as usize] = color;
                    }

                }
            });

            handles.push(handle);
        }
    }
     
    let mut i = 0;
    let size = handles.len();
    for h in handles
    {
        h.join().unwrap();

        println! ("{} % done!", ((i + 1)*100) as f32/size as f32);
        i +=1;
    }

    let data = std::sync::Arc::try_unwrap(lock).expect("Lock still has multiple owners");

    return data.into_inner().expect("Mutex cannot be locked");
}


fn main() {

    let width = 40000;
    let height = 40000;
    let iterationdepth = 150;

    let mandle_vector = generate_mandlebrot_set(width, height, iterationdepth);

    print_mandlebrot_set(width, height, mandle_vector);

}