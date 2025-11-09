use ark_ff::Field;

fn main() {
    println!("Hello, world!");
}

pub fn rotation_matrix<F: Field>(c0: &[F], negacyclic: bool) -> Vec<Vec<F>> {
    let n = c0.len();
    assert!(n > 0);
    let mut m = vec![vec![F::ZERO; n]; n];

    for (i, row) in m.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            let idx = ((i as isize - j as isize).rem_euclid(n as isize)) as usize;
            let sign = if negacyclic && i < j { -F::ONE } else { F::ONE };
            *cell = sign * c0[idx];
        }
    }
    m
}

pub fn setup<F: Field>(n: usize, m: usize, seed: &str) -> Vec<Vec<F>> {
    // create a block-matrix
    let a = Vec::with_capacity(n*m);
    for _ in 0..m / n {
        let base = // create a random vector of size n from the seed.
        let a_i = rotation_matrix(&base, true);
        // extend the a matrix with the a_i matrix
    }
    a
}

pub fn commit<F: Field>(a: Vec<Vec<F>>, z: Vec<F>) -> Vec<F> {
    assert!(/* assert the size of matrix and the vector for the mulplicatin*/);
    
    // todo: Multiply the a matrix and the z vector and return the result vector of size n
}

// todo, test the rotation matrix.
