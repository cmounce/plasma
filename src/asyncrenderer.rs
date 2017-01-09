use genetics::Genome;
use renderer::{Image, PlasmaRenderer};
use settings::RenderingSettings;
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, RecvError};

pub struct AsyncRenderer {
    request_tx: Sender<Request>,
    response_rx: Receiver<Response>,
    last_request_id: u32,
    genome: Option<Genome>,
    genome_set: bool
}

struct Request {
    request_id: u32,
    genome: Option<Genome>,
    width: usize,
    height: usize,
    time: f32
}

struct Response {
    image: Image,
    request_id: u32
}

impl AsyncRenderer {
    pub fn new(settings: &RenderingSettings) -> AsyncRenderer {
        let (request_tx, request_rx) = mpsc::channel();
        let (response_tx, response_rx) = mpsc::channel();
        let settings_clone = settings.clone();
        thread::spawn(|| {
            AsyncRenderer::thread(request_rx, response_tx, settings_clone);
        });

        AsyncRenderer {
            request_tx: request_tx,
            response_rx: response_rx,
            last_request_id: 0,
            genome: None,
            genome_set: false
        }
    }

    fn next_request_id(&mut self) -> u32 {
        self.last_request_id = self.last_request_id.wrapping_add(1);
        self.last_request_id
    }

    pub fn set_genome(&mut self, genome: &Genome) {
        self.genome = Some(genome.clone());
        self.genome_set = true;
        self.next_request_id(); // Increment request ID to invalidate previous requests
    }

    pub fn render(&mut self, width: usize, height: usize, time: f32) {
        assert!(self.genome_set, "Must call set_genome() before calling render()");
        let request = Request {
            request_id: self.next_request_id(),
            genome: self.genome.take(),
            width: width,
            height: height,
            time: time
        };
        self.request_tx.send(request).unwrap();
    }

    pub fn get_image(&mut self) -> Option<Image> {
        while let Ok(response) = self.response_rx.try_recv() {
            if response.request_id == self.last_request_id {
                return Some(response.image);
            }
        }
        None
    }

    fn thread(rx: Receiver<Request>, tx: Sender<Response>, settings: RenderingSettings) {
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
            let response = Response {
                request_id: request.request_id,
                image: image
            };
            tx.send(response).unwrap();
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

    /*
     *  Helper functions
     */

    fn dummy_settings() -> RenderingSettings {
        RenderingSettings {
            dithering: false,
            frames_per_second: 16.0,
            loop_duration: 60.0,
            palette_size: None,
            width: 32,
            height: 32
        }
    }

    fn rand_genome() -> Genome {
        Genome {
            pattern: Chromosome::rand(NUM_FORMULA_GENES, FORMULA_GENE_SIZE),
            color: Chromosome::rand(NUM_COLOR_GENES, CONTROL_POINT_GENE_SIZE)
        }
    }

    fn wait_for_image(renderer: &mut AsyncRenderer) -> Image {
        for _ in 0..100 {
            if let Some(image) = renderer.get_image() {
                return image;
            }
            sleep(Duration::from_millis(5));
        }
        panic!("Never got image from AsyncRenderer");
    }

    /*
     *  Tests
     */

    #[test]
    fn test_asyncrenderer_singlerender() {
        // Make a request
        let genome = rand_genome();
        let mut ar = AsyncRenderer::new(&dummy_settings());
        ar.set_genome(&genome);
        ar.render(32, 32, 0.0);

        // Assert that image is not available right away, but that we eventually get it
        assert!(ar.get_image().is_none());
        let image1 = wait_for_image(&mut ar);

        // Compare image with regular Renderer
        let mut r = PlasmaRenderer::new(&genome);
        let mut image2 = Image::new(32, 32);
        r.render(&mut image2, 0.0);
        assert_eq!(image1.pixel_data, image2.pixel_data);
    }

    #[test]
    fn test_asyncrenderer_cancellation() {
        // Make a small request
        let genome = rand_genome();
        let mut ar = AsyncRenderer::new(&dummy_settings());
        ar.set_genome(&genome);
        ar.render(32, 32, 0.0);

        // A moment later, make another small request
        sleep(Duration::from_millis(200));
        ar.render(32, 32, 0.5);

        // Assert that the result from the first request has been cleared out
        assert!(ar.get_image().is_none());

        // Assert that we eventually get a result for the second request
        let actual = wait_for_image(&mut ar);
        let mut r = PlasmaRenderer::new(&genome);
        let mut expected = Image::new(32, 32);
        r.render(&mut expected, 0.5);
        assert_eq!(expected.pixel_data, actual.pixel_data);
    }
}
