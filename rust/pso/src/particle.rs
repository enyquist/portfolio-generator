use ndarray::Array1;
use rand::Rng;
use polars::frame::DataFrame;

use crate::utils::{AssetType, AssetConfig, TaxBracket};
use crate::optimizer::{objective_function};


#[derive(Debug, Clone)]
pub struct Particle {
    position: Array1<f64>,
    velocity: Array1<f64>,
    best_position: Array1<f64>,
    best_score: f64,
    asset_types: Vec<AssetType>,  // Indicates the type of asset each weight corresponds to
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

    // Setter for asset_types
    pub fn set_asset_types(&mut self, new_asset_types: Vec<AssetType>) {
        self.asset_types = new_asset_types;
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
        .map(|&is_etf| if is_etf { AssetType::Stock } else { AssetType::ETF })
        .collect();

    for _ in 0..num_particles {

        let mut position = Array1::<f64>::zeros(num_assets);
        let mut velocity = Array1::<f64>::zeros(num_assets);

        for i in 0..num_assets {
            let config = asset_configs.iter().find(|config| config.asset_type() == asset_types[i]).unwrap();
            position[i] = rng.gen_range(config.range().min()..config.range().max());
            velocity[i] = rng.gen_range(-0.1..0.1);
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
    inertia: f64,
    cognitive: f64,
    social: f64,
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

    for particle in particles.iter_mut() {
        for i in 0..particle.position.len() {
            let cognitive_component = cognitive * rng.gen::<f64>() * (particle.best_position[i] - particle.position[i]);
            let social_component = social * rng.gen::<f64>() * (global_best_position[i] - particle.position[i]);
            particle.velocity[i] = inertia * particle.velocity[i] + cognitive_component + social_component;
            particle.position[i] += particle.velocity[i];

            // Apply the correct bounds
            particle.position[i] = match particle.asset_types[i] {
                AssetType::Stock => particle.position[i].min(0.05).max(0.0),
                AssetType::ETF => particle.position[i].min(0.35).max(0.0),
            };
        }

        let score = objective_function(&particle, &df, min_div_growth, min_cagr, min_yield, required_income, initial_capital, div_preference, cagr_preference, yield_preference, salary, &qualified_brackets, &non_qualified_brackets);

        if score < *particle.best_score() {
            particle.set_best_position(particle.position().clone());
            particle.set_best_score(score);
        }
    }
}


pub fn normalize_and_adjust_weights(particles: &mut [Particle]) {
    for particle in particles.iter_mut() {
        // Drop weights below 1% by setting them to zero
        for i in 0..particle.position.len() {
            if particle.position[i] < 0.01 {
                particle.position[i] = 0.0;
            }
        }

        // Calculate the new total weight after dropping low weights
        let total_weight: f64 = particle.position.sum();

        // Normalize the remaining weights
        if total_weight != 0.0 {
            particle.position.mapv_inplace(|x| x / total_weight);
        }

        // Reapply constraints ensuring no weight goes below 1% for remaining assets
        for i in 0..particle.position.len() {
            if particle.position[i] != 0.0 {  // Only apply bounds to non-zero weights
                particle.position[i] = match particle.asset_types[i] {
                    AssetType::Stock => particle.position[i].min(0.05).max(0.01),
                    AssetType::ETF => particle.position[i].min(0.35).max(0.01),
                };
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::AssetRange;
    use polars::prelude::*;

    // Helper function to create AssetConfig
    fn create_asset_configs() -> Vec<AssetConfig> {
        let mut configs = Vec::new();
    
        let stock_range = AssetRange { min: 0.0, max: 0.05 };
        let stock_config = AssetConfig {
            asset_type: AssetType::Stock,
            range: stock_range,
        };
    
        let etf_range = AssetRange { min: 0.0, max: 0.35 };
        let etf_config = AssetConfig {
            asset_type: AssetType::ETF,
            range: etf_range,
        };
    
        // Add to configs vector
        configs.push(stock_config);
        configs.push(etf_config);
    
        configs
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
        let configs = create_asset_configs();
        let asset_types = vec![false, true]; // True for ETF, False for Stock

        let particles = initialize_particles(num_particles, num_assets, &asset_types, &configs);

        assert_eq!(particles.len(), num_particles);
        for particle in particles {
            assert_eq!(particle.position.len(), num_assets);
            for (i, &is_etf) in asset_types.iter().enumerate() {
                let range = if is_etf {
                    configs[0].range() // ETF range
                } else {
                    configs[1].range() // Stock range
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
        let asset_types = vec![true, false];
        let mut particles = initialize_particles(1, 2, &asset_types, &create_asset_configs());
        let global_best_position = Array1::from(vec![0.02, 0.1]);
        let dummy_df = create_test_dataframe();

        update_particles(
            &mut particles,
            &global_best_position,
            0.5, 0.3, 0.2, &dummy_df,
            0.1, 0.1, 0.05, 50000.0, 100000.0,
            0.33, 0.33, 0.33, 50000.0,
            &[],
            &[]
        );
    }

    #[test]
    fn test_normalize_and_adjust_weights() {
        let asset_types = vec![true, false, true];
        let mut particles = initialize_particles(1, 3, &asset_types, &create_asset_configs());
        // Artificially set weights for testing
        particles[0].position = Array1::from(vec![0.009, 0.009, 0.009]);
        normalize_and_adjust_weights(&mut particles);
        // Check if weights below 0.01 are set to zero
        assert_eq!(particles[0].position, Array1::from(vec![0.0, 0.0, 0.0]));
    }
}
