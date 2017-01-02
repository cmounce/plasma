use genetics::Genome;
use renderer::{Image, PlasmaRenderer};
use settings::RenderingSettings;
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

pub struct AsyncRenderer {
    request_tx: Sender<RenderRequest>,
    image_rx: Receiver<Image>,
    genome_set: bool
}

enum RenderRequest {
    SetGenome(Genome),
    Render { width: usize, height: usize, time: f32 }
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
            genome_set: false
        }
    }

    pub fn set_genome(&mut self, genome: &Genome) {
        let request = RenderRequest::SetGenome(genome.clone());
        self.request_tx.send(request).unwrap();
        self.genome_set = true;
    }

    pub fn render(&mut self, width: usize, height: usize, time: f32) {
        assert!(self.genome_set, "Must call set_genome() before calling render()");
        let request = RenderRequest::Render { width: width, height: height, time: time };
        self.request_tx.send(request).unwrap();
    }

    pub fn get_image(&mut self) -> Option<Image> {
        self.image_rx.try_recv().ok()
    }

    fn thread(rx: Receiver<RenderRequest>, tx: Sender<Image>, settings: RenderingSettings) {
        let mut renderer = None;
        while let Ok(request) = rx.recv() {
            match request {
                RenderRequest::SetGenome(genome) => {
                    renderer = Some(PlasmaRenderer::new(&genome));
                },
                RenderRequest::Render{width, height, time} => {
                    let mut image = Image::new(width, height);
                    renderer.as_mut().unwrap().render(&mut image, time);
                    tx.send(image).unwrap();
                }
            };
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
