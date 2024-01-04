use array::ArrayTrait;
use orion::operators::tensor::{FP16x16Tensor, TensorTrait, Tensor};
use orion::numbers::{FixedTrait, FP16x16, FP16x16Impl};

fn Y_values() -> Tensor<FP16x16> {
    // Define the shape of the tensor (4 samples, 1 feature)
    let mut shape = ArrayTrait::new();
    shape.append(4); // Number of samples

    // Define the data in fixed-point representation
    let mut data = ArrayTrait::new();
    
    // Append each value from the Python Y array
    data.append(FixedTrait::new(6, false)); // 3
    data.append(FixedTrait::new(8, false)); // 6
    data.append(FixedTrait::new(9, false)); // 7
    data.append(FixedTrait::new(11, false)); // 8
    
    // Construct the tensor
    let tensor = TensorTrait::<FP16x16>::new(shape.span(), data.span());

    return tensor;
}