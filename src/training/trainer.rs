use crate::network::layer::Layer;
use crate::network::activation;
use crate::data::dataset::Sample;
use crate::utils::math::shuffle_dataset;
use crate::metrics::accuracy::evaluate;
use ndarray::Array1;

pub fn forward_pass(layers: &mut [Layer], inputs: &Array1<f32>) {
    let total_layers = layers.len();

    for (i, neuron) in layers[0].neurons.iter_mut().enumerate() {
        neuron.raw_value = inputs[i];
        neuron.activated_value = inputs[i];
    }

    for l in 1..total_layers {
        let prev_activations = layers[l - 1].activated_values();

        let is_not_output = l < (total_layers - 1);

        for neuron in &mut layers[l].neurons {
            let weighted_sum = neuron.weights.dot(&prev_activations) + neuron.bias;
            neuron.raw_value = weighted_sum;
            neuron.activated_value = if is_not_output {
                activation::sigmoid(weighted_sum)
            } else {
                weighted_sum // softmax will be applied later
            };
        }
    }

    // apply softmax now
    let output_index = total_layers - 1;
    let raw_outputs: Array1<f32> = layers[output_index]
        .neurons
        .iter()
        .map(|n| n.raw_value)
        .collect();
    let softmax_values = activation::softmax(&raw_outputs);

    for (neuron, &val) in layers[output_index].neurons.iter_mut().zip(softmax_values.iter()) {
        neuron.activated_value = val;
    }
}

pub fn back_propagate(layers: &mut [Layer], targets: &Array1<f32>, learning_rate: f32) {
    let output_index = layers.len() - 1;

    for (i, neuron) in layers[output_index].neurons.iter_mut().enumerate() {
        neuron.delta = neuron.activated_value - targets[i];
    }

    for l in (1..output_index).rev() {
        let next_layer_deltas: Vec<f32> = layers[l + 1].neurons.iter().map(|n| n.delta).collect();
        let next_layer_weights: Vec<Vec<f32>> = layers[l + 1].neurons.iter().map(|n| n.weights.to_vec()).collect();

        for (j, neuron) in layers[l].neurons.iter_mut().enumerate() {
            let sum: f32 = next_layer_deltas.iter().zip(next_layer_weights.iter()).map(|(delta, weights)| delta * weights[j]).sum();
            neuron.delta = sum * activation::sigmoid_derivative(neuron.raw_value);
        }
    }

    for l in 1..layers.len() {
        let prev_activations = layers[l - 1].activated_values();

        for neuron in &mut layers[l].neurons {
            neuron.bias -= learning_rate * neuron.delta;

            let gradient = &prev_activations * neuron.delta;
            neuron.weights = &neuron.weights - &(gradient * learning_rate);
        }
    }
}

fn calculate_loss(layers: &[Layer], targets: &Array1<f32>) -> f32 {
    let output_index = layers.len() - 1;
    layers[output_index]
        .neurons
        .iter()
        .zip(targets.iter())
        .map(|(neuron, &target)| -target * (neuron.activated_value + 1e-12).ln())
        .sum()
}

pub fn train(
    layers: &mut [Layer],
    training_set: &[Sample],
    epochs: usize,
    learning_rate: f32,
    test_set: &[Sample],
) {
    for epoch in 0..epochs {
        let mut shuffled = training_set.to_vec();
        shuffle_dataset(&mut shuffled);

        let mut total_loss = 0.0;
        for sample in shuffled.iter() {
            forward_pass(layers, &sample.inputs);
            back_propagate(layers, &sample.target, learning_rate);
            total_loss += calculate_loss(layers, &sample.target);
        }

        let train_acc = evaluate(layers, training_set);
        let test_acc = evaluate(layers, test_set);

        println!(
            "Epoch {}: Loss = {:.4}, Train Acc = {:.2}%, Test Acc = {:.2}%",
            epoch + 1,
            total_loss / training_set.len() as f32,
            train_acc,
            test_acc
        );
    }
}