use genetics::Genome;
use renderer::{Image, PlasmaRenderer};
use settings::RenderingSettings;
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, RecvError};

pub struct AsyncRenderer {
    request_tx: Sender<Request>,
    image_rx: Receiver<Image>,
    genome: Option<Genome>,
    genome_set: bool
}

struct Request {
    genome: Option<Genome>,
    width: usize,
    height: usize,
    time: f32
}

impl AsyncRenderer {
    pub fn new(settings: &RenderingSettings) -> AsyncRenderer {
        let (request_tx, request_rx) = mpsc::channel();
        let (image_tx, image_rx) = mpsc::channel();
        let settings_clone = settings.clone();
        thread::spawn(|| {
            AsyncRenderer::thread(request_rx, image_tx, settings_clone);
        });

        AsyncRenderer {
            request_tx: request_tx,
            image_rx: image_rx,
            genome: None,
            genome_set: false
        }
    }

    pub fn set_genome(&mut self, genome: &Genome) {
        self.genome = Some(genome.clone());
        self.genome_set = true;
    }

    pub fn render(&mut self, width: usize, height: usize, time: f32) {
        assert!(self.genome_set, "Must call set_genome() before calling render()");
        let request = Request {
            genome: self.genome.take(),
            width: width,
            height: height,
            time: time
        };
        self.request_tx.send(request).unwrap();
    }

    pub fn get_image(&mut self) -> Option<Image> {
        self.image_rx.try_recv().ok()
    }

    fn thread(rx: Receiver<Request>, tx: Sender<Image>, settings: RenderingSettings) {
        let mut renderer = None;
        loop {
            // Wait for a new request
            let mut request = match rx.recv() {
                Ok(request) => request,
                Err(RecvError) => return
            };

            // Fast-forward through backlog (if any) to get to the latest request
            while let Ok(req) = rx.try_recv() {
                let old_genome = request.genome;
                request = req;
                // Use previous genome if none specified
                request.genome = request.genome.or(old_genome);
            }

            // Render frame
            if let Some(genome) = request.genome {
                // If genome has changed since last render, rebuild renderer
                renderer = Some(PlasmaRenderer::new(&genome));
            }
            let mut image = Image::new(request.width, request.height);
            renderer.as_mut().unwrap().render(&mut image, request.time);
            tx.send(image).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use colormapper::{CONTROL_POINT_GENE_SIZE, NUM_COLOR_GENES};
    use formulas::{FORMULA_GENE_SIZE, NUM_FORMULA_GENES};
    use genetics::{Chromosome, Genome};
    use renderer::{Image, PlasmaRenderer};
    use settings::RenderingSettings;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_asyncrenderer_happycase() {
        // Get set up
        let settings = RenderingSettings {
            dithering: false,
            frames_per_second: 16.0,
            loop_duration: 60.0,
            palette_size: None,
            width: 32,
            height: 32
        };
        let genome = Genome {
            pattern: Chromosome::rand(NUM_FORMULA_GENES, FORMULA_GENE_SIZE),
            color: Chromosome::rand(NUM_COLOR_GENES, CONTROL_POINT_GENE_SIZE)
        };

        // Actually use AsyncRenderer
        let mut ar = AsyncRenderer::new(&settings);
        ar.set_genome(&genome);
        ar.render(32, 32, 0.0);

        // Poll for image
        let mut result = ar.get_image();
        assert!(result.is_none());
        for _ in 0..100 {
            sleep(Duration::from_millis(5));
            result = ar.get_image();
            if result.is_some() {
                break;
            }
        }
        let image = result.expect("Never got image from AsyncRenderer");

        // Compare image with regular Renderer
        let mut r = PlasmaRenderer::new(&genome);
        let mut image2 = Image::new(32, 32);
        r.render(&mut image2, 0.0);
        assert_eq!(image.pixel_data, image2.pixel_data);
    }
}
