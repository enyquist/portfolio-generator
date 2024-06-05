use ndarray::Array1;
use rand::Rng;
use polars::frame::DataFrame;
use std::f64::consts::E;

use crate::utils::{AssetType, AssetConfig, TaxBracket};
use crate::optimizer::{objective_function};


#[derive(Debug, Clone)]
pub struct Particle {
    pub position: Array1<f64>,
    pub velocity: Array1<f64>,
    pub best_position: Array1<f64>,
    pub best_score: f64,
    pub asset_types: Vec<AssetType>,  // Indicates the type of asset each weight corresponds to
}

impl Particle {
    // Getter for position
    pub fn position(&self) -> &Array1<f64> {
        &self.position
    }

    // Setter for position
    pub fn set_position(&mut self, new_position: Array1<f64>) {
        self.position = new_position;
    }

    // Getter for best_position
    pub fn best_position(&self) -> &Array1<f64> {
        &self.best_position
    }

    // Setter for best_position
    pub fn set_best_position(&mut self, new_best_position: Array1<f64>) {
        self.best_position = new_best_position;
    }

    // Getter for best_score
    pub fn best_score(&self) -> &f64 {
        &self.best_score
    }

    // Setter for best_score
    pub fn set_best_score(&mut self, new_best_score: f64) {
        self.best_score = new_best_score;
    }
}


pub fn initialize_particles(
    num_particles: usize,
    num_assets: usize,
    etf_flags: &[bool],  // Indicates the type of asset each weight corresponds to
    asset_configs: &[AssetConfig],  // List of asset types and their ranges
) -> Vec<Particle> {
    let mut rng = rand::thread_rng();
    let mut particles = Vec::with_capacity(num_particles);

    // Preprocess the bool flags into AssetTypes
    let asset_types: Vec<AssetType> = etf_flags.iter()
        .map(|&is_etf| if is_etf { AssetType::ETF } else { AssetType::Stock })
        .collect();

    for _ in 0..num_particles {

        let mut position = Array1::<f64>::zeros(num_assets);
        let mut velocity = Array1::<f64>::zeros(num_assets);

        // Generate initial positions and velocities
        for i in 0..num_assets {
            let config = asset_configs.iter().find(|config| config.asset_type() == asset_types[i]).unwrap();
            position[i] = rng.gen_range(config.range().min()..config.range().max());
            velocity[i] = rng.gen_range(-0.1..0.1);
        }

        // Normalize positions so that their sum equals 1.0
        let total_weight: f64 = position.sum();
        if total_weight > 0.0 {
            position.mapv_inplace(|x| x / total_weight);
        }

        // Ensure individual weight constraints are not violated
        for i in 0..num_assets {
            let config = asset_configs.iter().find(|config| config.asset_type() == asset_types[i]).unwrap();
            position[i] = position[i].clamp(config.range().min(), config.range().max());
        }

        particles.push(Particle {
            position: position.clone(),
            velocity,
            best_position: position,
            best_score: f64::INFINITY,
            asset_types: asset_types.clone(),
        });
    }

    particles
}


pub fn update_particles(
    particles: &mut [Particle],
    global_best_position: &Array1<f64>,
    initial_inertia: f64,
    decay_rate: f64,
    cognitive: f64,
    social: f64,
    iteration: usize,
    df: &DataFrame,
    min_div_growth: f64,
    min_cagr: f64,
    min_yield: f64,
    required_income: f64,
    initial_capital: f64,
    div_preference: f64,
    cagr_preference: f64,
    yield_preference: f64,
    salary: f64,
    qualified_brackets: &[TaxBracket],
    non_qualified_brackets: &[TaxBracket],
) {
    let mut rng = rand::thread_rng();
    // let inertia = initial_inertia * (1.0 - iteration as f64 / max_iterations as f64); // Decrease inertia over time
    let inertia = initial_inertia * E.powf(-decay_rate * iteration as f64);


    for particle in particles.iter_mut() {
        // Update velocity and position
        for i in 0..particle.position.len() {
            let cognitive_component = cognitive * rng.gen::<f64>() * (particle.best_position[i] - particle.position[i]);
            let social_component = social * rng.gen::<f64>() * (global_best_position[i] - particle.position[i]);
            particle.velocity[i] = inertia * particle.velocity[i] + cognitive_component + social_component;
            particle.position[i] += particle.velocity[i];
        }

        // Normalize positions so their sum equals 1.0
        let total_weight: f64 = particle.position.sum();
        if total_weight > 0.0 {
            particle.position.mapv_inplace(|x| x / total_weight);
        }

        // Clamp positions to ensure they are within bounds
        for i in 0..particle.position.len() {
            particle.position[i] = match particle.asset_types[i] {
                AssetType::Stock => particle.position[i].min(0.05).max(0.00),
                AssetType::ETF => particle.position[i].min(0.35).max(0.00),
            };
        }

        // Re-evaluate objective function and update best state if necessary
        let score = objective_function(&particle, &df, min_div_growth, min_cagr, min_yield, required_income, initial_capital, div_preference, cagr_preference, yield_preference, salary, &qualified_brackets, &non_qualified_brackets);

        if score < *particle.best_score() {
            particle.set_best_position(particle.position().clone());
            particle.set_best_score(score);
        }
    }
}


pub fn normalize_and_adjust_weights(particles: &mut [Particle]) {
    for particle in particles.iter_mut() {
        let mut weight_to_redistribute = 0.0;

        // Drop weights below 0.01 by setting them to zero and calculate redistribution amount
        for weight in particle.position.iter_mut() {
            if *weight < 0.01 {
                weight_to_redistribute += *weight;
                *weight = 0.0;
            }
        }

        // Calculate the amount each weight can increase, ignoring those under 0.01
        let potential_increase: Vec<f64> = particle.position.iter().enumerate().map(|(i, &w)| {
            if w >= 0.01 {
                let bounds = match particle.asset_types[i] {
                    AssetType::Stock => 0.05,
                    AssetType::ETF => 0.35,
                };
                bounds - w // Calculate increase potential only if weight is within the valid range
            } else {
                0.0 // No increase potential for weights below the threshold
            }
        }).collect();

        let total_potential_increase: f64 = potential_increase.iter().sum();

        // Redistribute the dropped weight proportionally
        if total_potential_increase > 0.0 && weight_to_redistribute > 0.0 {
            for (i, weight) in particle.position.iter_mut().enumerate() {
                if *weight >= 0.01 {
                    let increase = (potential_increase[i] / total_potential_increase) * weight_to_redistribute;
                    *weight += increase;
                }
            }
        }

        // Final normalization if necessary
        let corrected_total: f64 = particle.position.iter().sum();

        if corrected_total != 1.0 {
            for weight in particle.position.iter_mut() {
                if *weight > 0.01 {
                    *weight /= corrected_total;
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::AssetRange;
    use polars::prelude::*;

    // Helper function to create AssetConfig based on a vector of bools
    fn create_asset_configs(asset_types: &[bool]) -> Vec<AssetConfig> {
        let stock_range = AssetRange { min: 0.00, max: 0.05 };
        let etf_range = AssetRange { min: 0.00, max: 0.35 };

        asset_types.iter().map(|&is_etf| {
            if !is_etf {
                AssetConfig {
                    asset_type: AssetType::ETF,
                    range: etf_range,
                }
            } else {
                AssetConfig {
                    asset_type: AssetType::Stock,
                    range: stock_range,
                }
            }
        }).collect()
    }

    // Helper functions to create a DataFrame
    fn create_test_dataframe() -> DataFrame {
        let qualified = BooleanChunked::from_slice("Qualified", &[true, false]);
        let yields = Float64Chunked::from_slice("Yield", &[0.02, 0.03]);
        let div_growths = Float64Chunked::from_slice("5 Yr Dividend Growth", &[0.05, 0.06]);
        let cagr_growths = Float64Chunked::from_slice("5 Yr CAGR", &[0.07, 0.08]);
        let expense_ratios = Float64Chunked::from_slice("Expense Ratio", &[0.01, 0.02]);
        let is_etf = Float64Chunked::from_slice("ETF", &[0.0, 1.0]);
        let sector1 = Float64Chunked::from_slice("Sector 1", &[0.3, 0.5]);
        let sector2 = Float64Chunked::from_slice("Sector 2", &[0.2, 0.4]);
        DataFrame::new(vec![
            qualified.into_series(),
            yields.into_series(),
            div_growths.into_series(),
            cagr_growths.into_series(),
            expense_ratios.into_series(),
            is_etf.into_series(),
            sector1.into_series(),
            sector2.into_series(),
        ]).unwrap()
    }

    #[test]
    fn test_initialize_particles() {
        let num_assets = 2;
        let num_particles = 10;
        let asset_types = vec![true, false]; // True for ETF, False for Stock
        let configs = create_asset_configs(&asset_types);

        let particles = initialize_particles(num_particles, num_assets, &asset_types, &configs);

        assert_eq!(particles.len(), num_particles);
        for particle in particles {
            assert_eq!(particle.position.len(), num_assets);
            for (i, &is_etf) in asset_types.iter().enumerate() {
                let range = if is_etf {
                    configs[1].range() // ETF range
                } else {
                    configs[0].range() // Stock range
                };
                assert!(
                    particle.position[i] >= range.min() && particle.position[i] <= range.max(),
                    "Particle position out of expected range: position[{}] = {}, range = ({}, {})",
                    i, particle.position[i], range.min(), range.max()
                );
            }
        }
    }

    #[test]
    fn test_update_particles() {
        let num_assets = 2;
        let num_particles = 10;
        let asset_types = vec![true, false];  // True for ETF, False for Stock
        let configs = create_asset_configs(&asset_types);
        let mut particles = initialize_particles(num_particles, num_assets, &asset_types, &configs);
        let global_best_position = Array1::from(vec![0.02, 0.1]);
        let dummy_df = create_test_dataframe();

        update_particles(
            &mut particles,
            &global_best_position,
            0.5, 0.1, 0.3, 0.2, 1, &dummy_df,
            0.1, 0.1, 0.05, 50000.0, 100000.0,
            0.33, 0.33, 0.33, 50000.0,
            &[],
            &[]
        );

        // Check that particles obey the constraints
        for particle in particles {
            // Check that there are the same number of positions as assets
            assert_eq!(particle.position.len(), num_assets);

            // Check that each position is within the expected range
            for (i, &is_etf) in asset_types.iter().enumerate() {
                let range = if is_etf {
                    configs[1].range() // ETF range
                } else {
                    configs[0].range() // Stock range
                };
                assert!(
                    particle.position[i] >= range.min() && particle.position[i] <= range.max(),
                    "Particle position out of expected range: position[{}] = {}, range = ({}, {})",
                    i, particle.position[i], range.min(), range.max()
                );
            }

            // Check that the total weight of the positions is 1.0
            // let total_weight: f64 = particle.position().iter().sum();
            // dbg!(total_weight);
            // assert!((total_weight - 1.0).abs() < 1e-8);
        }
    }

    #[test]
    fn test_normalize_and_adjust_weights_redistribute() {
        let num_particles = 1;
        let num_assets = 5;
        let asset_types = vec![true, false, true, false, true];  // Assume true for ETF, false for Stock
        let asset_configs = vec![
            AssetConfig { asset_type: AssetType::ETF, range: AssetRange { min: 0.01, max: 0.35 } },
            AssetConfig { asset_type: AssetType::Stock, range: AssetRange { min: 0.01, max: 0.05 } },
            AssetConfig { asset_type: AssetType::ETF, range: AssetRange { min: 0.01, max: 0.35 } },
            AssetConfig { asset_type: AssetType::Stock, range: AssetRange { min: 0.01, max: 0.05 } },
            AssetConfig { asset_type: AssetType::ETF, range: AssetRange { min: 0.01, max: 0.35 } },
        ];
    
        let mut particles = initialize_particles(num_particles, num_assets, &asset_types, &asset_configs);

        particles[0].position = Array1::from(vec![0.35, 0.005, 0.35, 0.05, 0.245]);  // Intentionally set total weight above 1.0 to see normalization
    
        normalize_and_adjust_weights(&mut particles);

        // Ensure the total weight is 1 or very close, considering float inaccuracies
        let total_weight: f64 = particles[0].position.sum();
        assert!((total_weight - 1.0).abs() < 1e-8);
    
        // Ensure no weight exceeds its max
        for i in 0..num_assets {
            assert!(particles[0].position[i] <= asset_configs[i].range.max);
        }

        // Ensure no weight is less than 0.01
        for i in 0..num_assets {
            if i == 1 {
                assert_eq!(particles[0].position[i], 0.0);
            } else {
                assert!(particles[0].position[i] >= 0.01);
            }
        }
    }
}
