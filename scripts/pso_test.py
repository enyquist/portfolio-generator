# third party libraries
import rust_pso

# Parameters for PSO
num_particles = 30
num_dimensions = 4
num_iterations = 100

# Call the PSO function
best_solution = rust_pso.pso(num_particles, num_dimensions, num_iterations)
print("Best solution (weights):", best_solution)
