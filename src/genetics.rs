extern crate rand;

use self::rand::Rng;
use self::rand::distributions::{Exp, IndependentSample};

/*
# Design
- Two chromosomes: one for pattern, one for color
- Color gene: position + color + is_enabled
- Pattern gene: just a value (index into chromosome determines which equation it goes into)
- Mutation: add a random number
- Recombination: for each gene in chromosome, randomly pick A's or B's copy

# Implementation
- A Genome is a collection of chromosomes.
- A chromosome is a Vec of Genes.
- A Gene can be either:
    - A ColorGene
    - A PatternGene
## Thoughts
- Maybe serialization should be on a Chromosome struct
    - e.g., Chromosome<ColorGene> would implement to_bytes() and from_bytes()
    - ColorGene would implement to_bytes() and from_bytes()
    - types passed around would be Iterators
- Implement the Genes first. Everything else is boilerplate.
- Maybe Genes should be generic: just a Vec<u8>
    Reasoning: genetics.rs shouldn't know Plasma's rules about how a gene behaves
        - genetics.rs only handles gene-level mixing and byte-level mutation.
        - Plasma code only handles converting genes to f32 values with special properties
*/

const MUTATION_RATE: f64 = 0.01;

#[derive(Clone,Debug,Eq,PartialEq)]
struct Gene {
    data: Vec<u8>
}

struct Chromosome {
    genes: Vec<Gene>
}

struct Genome {
    pattern: Chromosome,
    color: Chromosome
}

impl Gene {
    fn rand(num_bytes: usize) -> Gene {
        let mut rng = rand::thread_rng();
        let mut data = vec![];
        for _ in 0..num_bytes {
            data.push(rng.gen()); // TODO: is there a shorter way to do this?
        }

        Gene { data: data }
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
            let index = mutation_position.round() as usize;
            if index >= gene.data.len() {
                break;
            }
            // Replace one byte of the gene
            gene.data[index] = rng.gen();
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

    #[test]
    fn test_gene_mutating_clone() {
        // TODO: Add another test that tests many small genes?
        let gene_size = 5000.0;
        let g1 = Gene::rand(gene_size as usize);
        let g2 = g1.mutating_clone();
        let num_mutations = g1.hamming(&g2) as f64;
        let expected_mutations = gene_size*MUTATION_RATE;
        let variance = gene_size*MUTATION_RATE*(1.0 - MUTATION_RATE);
        let std_dev = variance.sqrt();
        let lower_bound = expected_mutations - std_dev*4.0;
        let upper_bound = expected_mutations + std_dev*4.0;
        assert!(lower_bound >= 1.0); // Make sure our bounds detect at least one mutation
        assert!(upper_bound < gene_size); // and that the lower bound isn't too high
        assert!(lower_bound < num_mutations && num_mutations < upper_bound);
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
