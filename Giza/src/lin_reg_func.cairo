use orion::operators::tensor::{Tensor, TensorTrait, FP16x16Tensor};
use orion::numbers::{FP16x16, FixedTrait};

/// Predicts the y values using the provided x values and computed beta and intercept.
fn predict_y_values(
    betas: Tensor<FP16x16>, x_values: Tensor<FP16x16>, intercept: Tensor<FP16x16>
) -> Tensor<FP16x16> {

    // create a tensor to hold all the y_pred values
    let mut y_pred_shape = array::ArrayTrait::new();
    y_pred_shape.append(x_values.data.len());

    let mut y_pred_vals = array::ArrayTrait::new();

    let mut i: u32 = 0;
    loop {
        if i >=x_values.data.len() {
            break ();
        }
        // (*x_values.data.at(i)).print();
        // TODO figure out how to elementwise multiply a tensor here using Orion.
        // If not add another for loop to do the elementwise multiplication.
        loop{
            if j >= betas.data.length {
                break ();
            }
            let predicted_value = betas.data.at(j) * *x_values.data.at(indices: array![i, j].span()) + intercept;
            y_pred_vals.append(predicted_value);
            j += 1;
        };
       
        y_pred_vals.append(predicted_value);
        i += 1;
    };

    let y_pred_tensor = TensorTrait::<FP16x16>::new(y_pred_shape.span(), y_pred_vals.span());

    return y_pred_tensor;
}


/// Calculates the mean squared error between the true y values and the predicted y values.
fn compute_mse(y_values: Tensor<FP16x16>, y_pred_values: Tensor<FP16x16>) -> FP16x16 {
    let mut squared_diff_shape = array::ArrayTrait::new();
    squared_diff_shape.append(y_values.data.len());

    let mut squared_diff_vals = array::ArrayTrait::new();

    let mut i: u32 = 0;
    loop {
        if i >= y_values.data.len() {
            break ();
        }
        let diff = *y_values.data.at(i) - *y_pred_values.data.at(i);
        let squared_diff = diff * diff;
        squared_diff_vals.append(squared_diff);
        i += 1;
    };

    let squared_diff_tensor = TensorTrait::<FP16x16>::new(
        squared_diff_shape.span(), squared_diff_vals.span()
    );

    let mse = calculate_mean(squared_diff_tensor);

    return mse;
}

/// Calculates the R squared score.
fn calculate_r_score(y_values: Tensor<FP16x16>, y_pred_values: Tensor<FP16x16>) -> FP16x16 {
    let mean_y_value = calculate_mean(y_values);

    // creating the appropriate tensor shapes and empty arrays to populate values into
    let mut squared_diff_shape = array::ArrayTrait::new();
    squared_diff_shape.append(y_values.data.len());
    let mut squared_diff_vals = array::ArrayTrait::new();

    let mut squared_mean_diff_shape = array::ArrayTrait::new();
    squared_mean_diff_shape.append(y_values.data.len());
    let mut squared_mean_diff_vals = array::ArrayTrait::new();

    let mut i: u32 = 0;
    loop {
        if i >= y_values.data.len() {
            break ();
        }
        let diff_pred = *y_values.data.at(i) - *y_pred_values.data.at(i);
        let squared_diff = diff_pred * diff_pred;
        squared_diff_vals.append(squared_diff);

        let diff_mean = *y_values.data.at(i) - mean_y_value;
        let squared_mean_diff = diff_mean * diff_mean;
        squared_mean_diff_vals.append(squared_mean_diff);
        i += 1;
    };

    let squared_diff_tensor = TensorTrait::<FP16x16>::new(
        squared_diff_shape.span(), squared_diff_vals.span()
    );
    let squared_mean_diff_tensor = TensorTrait::<FP16x16>::new(
        squared_mean_diff_shape.span(), squared_mean_diff_vals.span()
    );

    let sum_squared_diff = squared_diff_tensor.cumsum(0, Option::None(()), Option::None(()));
    let sum_squared_mean_diff = squared_mean_diff_tensor
        .cumsum(0, Option::None(()), Option::None(()));

    let r_score = FixedTrait::new_unscaled(1, false)
        - *sum_squared_diff.data.at(y_values.data.len() - 1)
            / *sum_squared_mean_diff.data.at(y_values.data.len() - 1);

    return r_score;
}