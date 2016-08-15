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
*/

const GENE_SIZE: usize = 16;

#[derive(Debug)]
struct Genome {
    color: Vec<Gene>,
    shape: Vec<Gene>
}

#[derive(Debug,Eq,PartialEq)]
struct Gene {
    data: Vec<u8>
}

impl Gene {
    fn rand() -> Gene {
        let mut rng = rand::thread_rng();
        let mut data = vec![];
        for _ in 0..GENE_SIZE {
            data.push(rng.gen());
        }

        Gene { data: data }
    }

    fn mutate(&mut self) {

    }
}

#[cfg(test)]
mod tests {
    use super::Gene;

    #[test]
    fn test_gene_rand() {
        let g1 = Gene::rand();
        let g2 = Gene::rand();
        assert!(g1 != g2);
        println!("{:?}", g1);
        println!("{:?}", g2);
        //assert!(false);
    }
}
