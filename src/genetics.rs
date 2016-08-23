extern crate rand;

use self::rand::Rng;
use self::rand::distributions::{Exp, IndependentSample, Normal};

/*
 * Definitions for genes, chromosomes, and genomes.
 *
 * This module doesn't know the plasma's rules about how a gene behaves.
 * It only handles gene-level mixing and byte-level mutation.
 */

const MUTATION_RATE: f64 = 0.01;
const MUTATION_STD_DEV: f64 = 32.0;

#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Gene {
    pub data: Vec<u8>
}

struct Chromosome {
    genes: Vec<Gene>
}

struct Genome {
    pattern: Chromosome,
    color: Chromosome
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

    fn mutating_clone(&self) -> Gene {
        let mut rng = rand::thread_rng();
        let exp = Exp::new(MUTATION_RATE);
        let normal = Normal::new(0.0, MUTATION_STD_DEV);
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
            // TODO: test distribution of mutations?
            let old_value = gene.data[index];
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
            gene.data[index] = new_value;
        }
        gene
    }
}

impl Chromosome {
    fn rand(num_genes: usize, gene_size: usize) -> Chromosome {
        let mut c = Chromosome { genes: vec![] };
        for _ in 0..num_genes {
            c.genes.push(Gene::rand(gene_size));
        }
        c
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
    fn breed(&self, other: &Genome) -> Genome {
        Genome {
            pattern: self.pattern.breed(&other.pattern),
            color: self.color.breed(&other.color)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::Gene;
    use super::Genome;
    use super::Chromosome;
    use super::MUTATION_RATE;

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
}
