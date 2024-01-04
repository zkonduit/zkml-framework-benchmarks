use array::ArrayTrait;
use orion::operators::tensor::{FP16x16Tensor, TensorTrait, Tensor};
use orion::numbers::{FixedTrait, FP16x16, FP16x16Impl};

fn X_values() -> Tensor<FP16x16> {
    // Define the shape of the tensor (4 samples, 2 features)
    let mut shape = ArrayTrait::new();
    shape.append(4); // Number of samples
    shape.append(2); // Number of features per sample

    // Define the data in fixed-point representation
    let mut data = ArrayTrait::new();
    
    // Sample 1
    data.append(FixedTrait::new(1, false)); // 1.0
    data.append(FixedTrait::new(1, false)); // 1.0
    
    // Sample 2
    data.append(FixedTrait::new(1, false)); // 1.0
    data.append(FixedTrait::new(2, false)); // 2.0
    
    // Sample 3
    data.append(FixedTrait::new(2, false)); // 2.0
    data.append(FixedTrait::new(2, false)); // 2.0
    
    // Sample 4
    data.append(FixedTrait::new(2, false)); // 2.0
    data.append(FixedTrait::new(3, false)); // 3.0
    
    // Construct the tensor
    let tensor = TensorTrait::<FP16x16>::new(shape.span(), data.span());

    return tensor;
}
