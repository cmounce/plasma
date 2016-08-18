extern crate rand;

use self::rand::Rng;

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
            data.push(rng.gen());
        }

        Gene { data: data }
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
            let gene = if rng.gen() { self.genes[i].clone() } else { other.genes[i].clone() };
            child.genes.push(gene);
        }
        // TODO: add mutation

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
    use super::Chromosome;
    use super::Genome;

    #[test]
    fn test_gene_rand() {
        let g1 = Gene::rand(8);
        let g2 = Gene::rand(8);
        assert!(g1 != g2);
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
        let num_genes: usize = 16;
        let a = Chromosome::rand(num_genes, 8);
        let b = Chromosome::rand(num_genes, 8);
        let c = a.breed(&b);
        assert!(c.genes.len() == num_genes);
        for i in 0..num_genes {
            assert!(c.genes[i] == a.genes[i] || c.genes[i] == b.genes[i]);
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
