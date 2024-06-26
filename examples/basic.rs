//! A basic example of NEAT with this crate. Enable the `crossover` feature for it to use crossover reproduction

use neat::*;
use rand::prelude::*;

#[derive(PartialEq, Clone, Debug, DivisionReproduction, RandomlyMutable)]
#[cfg_attr(feature = "crossover", derive(CrossoverReproduction))]
struct AgentDNA {
    network: NeuralNetworkTopology<2, 4>,
}

impl Prunable for AgentDNA {}

impl GenerateRandom for AgentDNA {
    fn gen_random(rng: &mut impl rand::Rng) -> Self {
        Self {
            network: NeuralNetworkTopology::new(0.01, 3, rng),
        }
    }
}

#[derive(Debug)]
struct Agent {
    network: NeuralNetwork<2, 4>,
}

impl From<&AgentDNA> for Agent {
    fn from(value: &AgentDNA) -> Self {
        Self {
            network: (&value.network).into(),
        }
    }
}

fn fitness(dna: &AgentDNA) -> f32 {
    let agent = Agent::from(dna);

    let mut fitness = 0.;
    let mut rng = rand::thread_rng();

    for _ in 0..10 {
        // 10 games

        // set up game
        let mut agent_pos: (i32, i32) = (rng.gen_range(0..10), rng.gen_range(0..10));
        let mut food_pos: (i32, i32) = (rng.gen_range(0..10), rng.gen_range(0..10));

        while food_pos == agent_pos {
            food_pos = (rng.gen_range(0..10), rng.gen_range(0..10));
        }

        let mut step = 0;

        loop {
            // perform actions in game
            let action = agent.network.predict([
                (food_pos.0 - agent_pos.0) as f32,
                (food_pos.1 - agent_pos.1) as f32,
            ]);
            let action = action.iter().max_index();

            match action {
                0 => agent_pos.0 += 1,
                1 => agent_pos.0 -= 1,
                2 => agent_pos.1 += 1,
                _ => agent_pos.1 -= 1,
            }

            step += 1;

            if agent_pos == food_pos {
                fitness += 10.;
                break; // new game
            } else {
                // lose fitness for being slow and far away
                fitness -=
                    (food_pos.0 - agent_pos.0 + food_pos.1 - agent_pos.1).abs() as f32 * 0.001;
            }

            // 50 steps per game
            if step == 50 {
                break;
            }
        }
    }

    fitness
}

fn main() {
    #[cfg(not(feature = "rayon"))]
    let mut rng = rand::thread_rng();

    let mut sim = GeneticSim::new(
        #[cfg(not(feature = "rayon"))]
        Vec::gen_random(&mut rng, 100),
        #[cfg(feature = "rayon")]
        Vec::gen_random(100),
        fitness,
        #[cfg(not(feature = "crossover"))]
        division_pruning_nextgen,
        #[cfg(feature = "crossover")]
        crossover_pruning_nextgen,
    );

    for _ in 0..100 {
        sim.next_generation();
    }

    #[cfg(not(feature = "serde"))]
    let mut fits: Vec<_> = sim.genomes.iter().map(fitness).collect();

    #[cfg(feature = "serde")]
    let mut fits: Vec<_> = sim.genomes.iter().map(|e| (e, fitness(e))).collect();

    #[cfg(not(feature = "serde"))]
    fits.sort_by(|a, b| a.partial_cmp(&b).unwrap());

    #[cfg(feature = "serde")]
    fits.sort_by(|(_, a), (_, b)| a.partial_cmp(&b).unwrap());

    dbg!(&fits);

    #[cfg(feature = "serde")]
    {
        let intermediate = NNTSerde::from(&fits[0].0.network);
        let serialized = serde_json::to_string(&intermediate).unwrap();
        println!("{}", serialized);
    }
}
