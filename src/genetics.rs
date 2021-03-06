extern crate rand;
extern crate rustc_serialize;

use std::collections::VecDeque;
use self::rand::Rng;
use self::rand::distributions::{Exp, IndependentSample, Normal};
use self::rustc_serialize::base64::{ToBase64, FromBase64, URL_SAFE};

/*
 * Definitions for genes, chromosomes, and genomes.
 *
 * This module doesn't know the plasma's rules about how a gene behaves.
 * It only handles gene-level mixing and byte-level mutation.
 *
 * Organization:
 * - A Genome represents everything about a plasma
 * - A Chromosome represents a certain aspect of a plasma (e.g., its color scheme)
 * - A Gene represents a further smaller component (e.g., that the color scheme contains red)
 * - Genes are byte vectors.
 */

const MUTATION_RATE: f64 = 0.03;
const MUTATION_STD_DEV: f64 = 32.0;

#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Gene {
    pub data: Vec<u8>
}

#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Chromosome {
    pub genes: Vec<Gene>
}

#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Genome {
    pub pattern: Chromosome,
    pub color: Chromosome
}

pub struct Population {
    genomes: VecDeque<Genome>,
    max_size: usize
}

trait Mutate {
    fn mutate(&self) -> Self;
}

impl Mutate for u8 {
    fn mutate(&self) -> u8 {
        let mut rng = rand::thread_rng();
        let normal = Normal::new(0.0, MUTATION_STD_DEV);

        let old_value = *self;
        let mut new_value = old_value;
        while new_value == old_value {
            let delta = normal.ind_sample(&mut rng).round();
            if delta >= -255.0 && delta <= 255.0 {
                new_value = if delta >= 0.0 {
                    old_value.saturating_add(delta as u8)
                } else {
                    old_value.saturating_sub(delta.abs() as u8)
                }
            }
        }

        new_value
    }
}

impl Gene {
    pub fn rand(num_bytes: usize) -> Gene {
        let mut rng = rand::thread_rng();
        let mut data = vec![];
        for _ in 0..num_bytes {
            data.push(rng.gen());
        }

        Gene { data: data }
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.data.clone()
    }

    fn from_bytes(bytes: &[u8]) -> Gene {
        Gene { data: bytes.to_vec() }
    }

    fn mutating_clone(&self) -> Gene {
        let mut rng = rand::thread_rng();
        let exp = Exp::new(MUTATION_RATE);
        let mut mutation_position = 0.0;
        // Start with a non-mutated version of self
        let mut gene = self.clone();
        loop {
            // Calculate distance to next mutation
            mutation_position += exp.ind_sample(&mut rng);
            let index = mutation_position.floor() as usize;
            if index >= gene.data.len() {
                break;
            }
            // Replace one byte of the gene
            gene.data[index] = gene.data[index].mutate();
        }
        gene
    }
}

impl Chromosome {
    pub fn rand(num_genes: usize, gene_size: usize) -> Chromosome {
        let mut c = Chromosome { genes: vec![] };
        for _ in 0..num_genes {
            c.genes.push(Gene::rand(gene_size));
        }
        c
    }

    fn to_bytes(&self) -> Vec<u8> {
        if self.genes.len() == 0 {
            return vec![0];
        }
        let gene_size = self.genes[0].data.len();
        let num_genes = self.genes.len();
        assert!(gene_size < 16);
        assert!(num_genes < 16);
        let header = ((gene_size & 0xF) << 4 | num_genes & 0xF) as u8;
        let mut result = vec![header];
        for gene in &self.genes {
            assert_eq!(gene_size, gene.data.len());
            let mut bytes = gene.to_bytes();
            result.append(&mut bytes);
        }
        result
    }

    fn from_mut_slice(slice: &mut &[u8]) -> Result<Chromosome, &'static str> {
        if slice.len() < 1 {
            return Err("Chromosome header is missing");
        }
        let header = slice[0];
        *slice = &slice[1..];
        let gene_size = ((header >> 4) & 0xF) as usize;
        let num_genes = (header & 0xF) as usize;
        let expected_len = gene_size*num_genes;
        if slice.len() < expected_len {
            return Err("Unexpected end of chromosome");
        }
        let mut genes = vec![];
        for _ in 0..num_genes {
            genes.push(Gene::from_bytes(&slice[0..gene_size]));
            *slice = &slice[gene_size..];
        }
        Ok(Chromosome { genes: genes })
    }

    fn breed(&self, other: &Chromosome) -> Chromosome {
        assert!(self.genes.len() == other.genes.len());
        let mut rng = rand::thread_rng();
        let mut child = Chromosome { genes: vec![] };
        for i in 0..self.genes.len() {
            let gene = if rng.gen() {
                self.genes[i].mutating_clone()
            } else {
                other.genes[i].mutating_clone()
            };
            child.genes.push(gene);
        }
        child
    }
}

impl Genome {
    pub fn breed(&self, other: &Genome) -> Genome {
        Genome {
            pattern: self.pattern.breed(&other.pattern),
            color: self.color.breed(&other.color)
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut result = self.pattern.to_bytes();
        result.append(&mut self.color.to_bytes());
        result
    }

    fn from_bytes(bytes: &[u8]) -> Result<Genome, &'static str> {
        let mut slice = &bytes[..];
        let pattern = try!(Chromosome::from_mut_slice(&mut slice));
        let color = try!(Chromosome::from_mut_slice(&mut slice));
        if !slice.is_empty() {
            return Err("Unexpected bytes at end of genome");
        }
        Ok(Genome { pattern: pattern, color: color })
    }

    pub fn to_base64(&self) -> String {
        let bytes = self.to_bytes();
        bytes.to_base64(URL_SAFE)
    }

    pub fn from_base64(data: &str) -> Result<Genome, &'static str> {
        if let Ok(bytes) = data.from_base64() {
            Genome::from_bytes(&bytes)
        } else {
            Err("Couldn't decode genome string")
        }
    }
}

impl Population {
    pub fn new(max_size: usize) -> Population {
        Population {
            genomes: VecDeque::with_capacity(max_size),
            max_size: max_size
        }
    }

    pub fn add(&mut self, genome: Genome) {
        self.genomes.push_back(genome);
        if self.genomes.len() > self.max_size {
            self.genomes.pop_front();
        }
    }

    pub fn get_pair(&self) -> Option<(&Genome, &Genome)> {
        let num_genomes = self.genomes.len();
        if num_genomes == 0 {
            None
        } else if num_genomes == 1 {
            // Only one genome: return it twice
            Some((self.genomes.get(0).unwrap(), self.genomes.get(0).unwrap()))
        } else {
            // Pick two different genomes
            let mut rng = rand::thread_rng();
            let index1 = rng.gen_range(0, num_genomes);
            let index2_raw = rng.gen_range(0, num_genomes - 1);
            let index2 = if index2_raw >= index1 { index2_raw + 1 } else { index2_raw };
            Some((self.genomes.get(index1).unwrap(), self.genomes.get(index2).unwrap()))
        }
    }

    pub fn breed(&self) -> Genome {
        let (a, b) = self.get_pair().expect("Couldn't get breeding pair");
        a.breed(&b)
    }
}

#[cfg(test)]
mod tests {
    use super::Mutate;
    use super::Gene;
    use super::Genome;
    use super::Chromosome;
    use super::Population;
    use super::MUTATION_RATE;
    use super::MUTATION_STD_DEV;
    use genetics::rustc_serialize::base64::{ToBase64, URL_SAFE};

    impl Gene {
        // Test helper -- used for detecting mutation
        fn hamming(&self, other: &Gene) -> usize {
            assert!(self.data.len() == other.data.len());
            let mut hamming = 0;
            for i in 0..self.data.len() {
                if self.data[i] != other.data[i] {
                    hamming += 1;
                }
            }
            hamming
        }
    }

    #[test]
    // Make sure that mutate() always returns a different number
    fn test_u8_mutate() {
        for _ in 0..2000 {
            assert!(0 != 0.mutate());
            assert!(128 != 128.mutate());
            assert!(255 != 255.mutate());
        }
    }

    #[test]
    // Make sure that nearby bytes are more likely to be chosen
    fn test_u8_mutate_distribution() {
        let num_mutations = 100;
        let mut sum = 0;
        for _ in 0..num_mutations {
            sum += 0.mutate() as u64;
        }
        let mean = (sum as f64)/(num_mutations as f64);
        assert!(mean < MUTATION_STD_DEV); // about 68% of mutations will be less than this
    }

    #[test]
    fn test_gene_rand() {
        let g1 = Gene::rand(8);
        let g2 = Gene::rand(8);
        assert!(g1 != g2);
    }

    // Calculates how many mutations would be too few or too many,
    // given num_cloned_bytes and MUTATION_RATE.
    fn calculate_mutation_bounds(num_cloned_bytes: usize) -> (usize, usize) {
        let n = num_cloned_bytes as f64;
        let expected_mutations = n*MUTATION_RATE;
        let variance = n*MUTATION_RATE*(1.0 - MUTATION_RATE);
        let std_dev = variance.sqrt();
        let lower_bound = (expected_mutations - std_dev*4.0).round() as usize;
        let upper_bound = (expected_mutations + std_dev*4.0).round() as usize;
        assert!(lower_bound > 0); // Make sure it's possible to fail the test by having
        assert!(upper_bound < num_cloned_bytes); // too few or too many mutations
        (lower_bound, upper_bound)
    }

    #[test]
    fn test_gene_mutating_clone() {
        let gene_size = 5000;
        let g1 = Gene::rand(gene_size);
        let g2 = g1.mutating_clone();
        let num_mutations = g1.hamming(&g2);
        let (lower_bound, upper_bound) = calculate_mutation_bounds(gene_size);
        assert!(lower_bound < num_mutations);
        assert!(num_mutations < upper_bound);
    }

    #[test]
    fn test_gene_mutating_clone_small() {
        let mut g = Gene::rand(1);
        let num_clones = 10000;
        let mut num_mutations = 0;
        for _ in 0..num_clones {
            let clone = g.mutating_clone();
            if g.hamming(&clone) > 0 {
                num_mutations += 1;
            }
            g = clone;
        }
        let (lower, upper) = calculate_mutation_bounds(num_clones);
        assert!(lower < num_mutations);
        assert!(num_mutations < upper);
    }

    #[test]
    fn test_chromosome_rand() {
        let num_genes = 8;
        let c = Chromosome::rand(num_genes, 8);
        assert!(c.genes.len() == num_genes);
        for i in 1..num_genes {
            assert!(c.genes[i] != c.genes[i - 1]);
        }
    }

    #[test]
    fn test_chromosome_breed() {
        let num_genes = 16;
        let gene_size = 16;
        let a = Chromosome::rand(num_genes, gene_size);
        let b = Chromosome::rand(num_genes, gene_size);
        let c = a.breed(&b);
        assert!(c.genes.len() == num_genes);
        for i in 0..num_genes {
            // Assert that a majority of this gene's bytes come from one of the parents.
            // Not all of them may match due to mutation.
            let a_distance = a.genes[i].hamming(&c.genes[i]);
            let b_distance = b.genes[i].hamming(&c.genes[i]);
            assert!(a_distance < gene_size/2 || b_distance < gene_size/2);
        }
    }

    #[test]
    fn test_genome_breed() {
        let a = Genome {
            color: Chromosome::rand(1, 2),
            pattern: Chromosome::rand(3, 4)
        };
        let b = Genome {
            color: Chromosome::rand(1, 2),
            pattern: Chromosome::rand(3, 4)
        };
        let c = a.breed(&b);
        assert!(c.color.genes.len() == 1);
        assert!(c.pattern.genes.len() == 3);
    }

    #[test]
    fn test_genome_to_base64() {
        let g = Genome {
            pattern: Chromosome { genes: vec![] },
            color: Chromosome {
                genes: vec![
                    Gene { data: vec![2, 3, 5, 7] },
                    Gene { data: vec![11, 13, 17, 19] }
                ]
            }
        };
        let bytes = vec![
            0, // Pattern header: empty Chromosome
            (4 << 4) | 2, // Color header: 2 genes, 4-byte each
            2, 3, 5, 7, // Gene 1
            11, 13, 17, 19 // Gene 2
        ];
        assert_eq!(g.to_base64(), bytes.to_base64(URL_SAFE));
    }

    #[test]
    fn test_genome_from_base64() {
        for gene_size in 0..15 {
            for num_genes in 0..15 {
                let g1 = Genome {
                    pattern: Chromosome::rand(num_genes, gene_size),
                    color: Chromosome::rand(num_genes, gene_size)
                };
                let s = g1.to_base64();
                if let Ok(g2) = Genome::from_base64(&s) {
                    assert_eq!(g1, g2,
                        "Bad deserialization with size = {}, number = {}", gene_size, num_genes
                    );
                } else {
                    panic!("Couldn't deserialize size = {}, number = {}", gene_size, num_genes);
                }
            }
        }
    }

    #[test]
    fn test_genome_from_base64_bad_data() {
        assert!(Genome::from_base64("").is_err());
        assert!(Genome::from_base64("!@#$%^&*()").is_err());
        assert!(Genome::from_base64(&vec![0].to_base64(URL_SAFE)).is_err());
    }

    #[test]
    fn test_population() {
        // Test get_pair() with 0 genomes
        let max_genomes = 5;
        let mut p = Population::new(max_genomes);
        assert_eq!(p.get_pair().is_some(), false);

        // Test with 1 genome
        let g = Genome {
            color: Chromosome::rand(4, 4),
            pattern: Chromosome::rand(4, 4)
        };
        p.add(g.clone());
        assert_eq!(p.get_pair().is_some(), true);

        // Test with 2 genomes
        p.add(g.clone());
        assert_eq!(p.get_pair().is_some(), true);

        // Fill Population past its limit of max_genomes
        for _ in 0..max_genomes {
            let g = Genome {
                color: Chromosome::rand(4, 4),
                pattern: Chromosome::rand(4, 4)
            };
            p.add(g);
        }
        for _ in 0..100 {
            let (g1, g2) = p.get_pair().unwrap();
            assert!(*g1 != *g2); // Make sure we got two different genomes
            assert!(*g1 != g && *g2 != g); // Make sure original genomes were flushed out
        }
    }
}
