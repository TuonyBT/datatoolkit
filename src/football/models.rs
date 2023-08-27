use std::collections::{BTreeMap};
use ndarray::{Array, Array2, Axis};


pub fn linear(ress: &BTreeMap::<i64, (Vec<(String, usize)>, Vec<f64>)>) -> () {

    let x_vec = ress.iter().map(|(k, _v)| *k as f64).collect::<Vec<f64>>();
    let yh_vec = ress.iter().map(|(_k, v)| v.1[0] * 100.0).collect::<Vec<f64>>();
    let yd_vec = ress.iter().map(|(_k, v)| v.1[1] * 100.0).collect::<Vec<f64>>();
    let ya_vec = ress.iter().map(|(_k, v)| v.1[2] * 100.0).collect::<Vec<f64>>();

    let x_arr = Array::from_vec(x_vec,);
    let yh_arr = Array::from_vec(yh_vec,);
    let yd_arr = Array::from_vec(yd_vec,);
    let ya_arr = Array::from_vec(ya_vec,);


    let x_bar= x_arr.mean().unwrap();
    let x_err = &x_arr - x_bar;
    let xe_dot_xe = x_err.dot(&x_err);

    let yh_bar= yh_arr.mean().unwrap();
    let yh_err = &yh_arr - yh_bar;
    let xe_dot_yhe = x_err.dot(&yh_err);
    let mh = &xe_dot_yhe / &xe_dot_xe;
    let ch = &yh_bar - &mh * x_bar;

    let yd_bar= yd_arr.mean().unwrap();
    let yd_err = &yd_arr - yd_bar;
    let xe_dot_yde = x_err.dot(&yd_err);
    let md = &xe_dot_yde / &xe_dot_xe;
    let cd= &yd_bar - &md * x_bar;

    let ya_bar= ya_arr.mean().unwrap();
    let ya_err = &ya_arr - ya_bar;
    let xe_dot_yae = x_err.dot(&ya_err);
    let ma = &xe_dot_yae / &xe_dot_xe;
    let ca= &ya_bar - &ma * x_bar;

    println!("Home wins linear fit y = {:?} *x + {:?}", mh, ch);
    println!("Draws linear fit y = {:?} *x + {:?}", md, cd);
    println!("Away wins linear fit y = {:?} *x + {:?}", ma, ca);
}


pub fn linear_gd<'a>(fv: &'a Vec<Vec<f64>>, rvs: &'a BTreeMap::<&String, Vec<f64>>)
    -> (BTreeMap::<&'a String, (Array<f64, ndarray::Dim<[usize; 2]>>, Array<f64, ndarray::Dim<[usize; 2]>>, f64)>,
     Array<f64, ndarray::Dim<[usize; 1]>>) {

// Declare vector where we will collect results
    let mut fitted_models = BTreeMap::<&String, (Array<f64, ndarray::Dim<[usize; 2]>>, Array<f64, ndarray::Dim<[usize; 2]>>, f64)>::new();

// Define the features we want to model and put them into the X (i.e. features) matrix
    let m = fv.len();

    let theta_len = fv[0].len();
    let features_vec = fv.into_iter().flatten().map(|&x| x).collect::<Vec<f64>>();
    let mut x_arr = Array2::from_shape_vec((m, theta_len), features_vec).unwrap();
    let x_mu = x_arr.mean_axis(Axis(0)).unwrap();
    let x_sig = x_arr.std_axis(Axis(0), 0.0);
    let x_sig_recip = x_sig.map(|sd| if sd == &0.0 {1.0}
                                     else {1.0 / *sd});
    let mu_des = Array::from_shape_vec((theta_len, theta_len), vec![0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]).unwrap();
    x_arr.scaled_add(-0.0, &mu_des.dot(&x_mu));
    x_arr.zip_mut_with(&x_sig_recip, |x, s| *x *= s);

//  Control parameters for the optimisation. Switches allows further control over features modelled: redundant if regularising
    let alpha = 0.1;
    let n_iterations = 1000;

//  Extract the observations that correspond to these features for the response we want to model
    for (obs, y_vec) in rvs.iter() {
        let y_arr = Array::from_shape_vec((m, 1), y_vec.clone()).unwrap();
//  Initialise hypothesis matrix (if we have standardised the features then 0.0 should be OK to start
        let theta = Array::from_shape_vec((theta_len, 1), vec![0.0; theta_len]).unwrap();
//  Switches allows further control over features modelled: redundant if regularising
        let switches = match obs.as_str() {
                       "Home Win" => Array::from_shape_vec((theta_len, theta_len), vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]).unwrap(),
                       _ => Array::from_shape_vec((theta_len, theta_len), vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]).unwrap(),
                       };


//  Run the optimisation and return the model parameters and fitted values
//        fitted_models.insert(obs, grad_desc(&x_arr, &y_arr, theta, &switches, &alpha, n_iterations));

        fitted_models.insert(obs, grad_desc(&x_arr, &y_arr, theta, &switches, &alpha, n_iterations));

//      in case we want to compare with theta from a non-standardised set of features
//        let mut theta_trans = fitted_models[obs].1.clone().column(0).to_owned();
//        theta_trans.zip_mut_with(&x_sig_recip, |x, s| *x *= s);
//        println!("{:?}", x_sig_recip);

    }
    (fitted_models, x_sig_recip)
}

// Gradient descent using linear regression

pub fn grad_desc(x: &Array2<f64>,
                y: &Array<f64, ndarray::Dim<[usize; 2]>>,
                mut th: Array<f64, ndarray::Dim<[usize; 2]>>,
                sw: &Array<f64, ndarray::Dim<[usize; 2]>>,
                al: &f64,
                n: i32) -> (Array<f64, ndarray::Dim<[usize; 2]>>, Array<f64, ndarray::Dim<[usize; 2]>>, f64) {

    let m = y.len();
    let y_mu = y.mean_axis(Axis(0)).unwrap();
    let y_err = y.map(|&yi| *(yi - &y_mu).get(0).unwrap()); //.iter().collect::<Array<f64, ndarray::Dim<[usize; 2]>>>();
    let y_var = y_err.t().dot(&y_err).sum();
    let mut mj: f64 = 0.0;

    for _iter in 0..n {
        let h_theta = x.dot(&th);
        let err = &h_theta - y;
        mj = (err.t().dot(&err)).sum();
        let j_grad = x.t().dot(&err) / m as f64;

        th.scaled_add(-al, &sw.dot(&j_grad));
    }
    (x.dot(&th), th, 1.0 - mj / y_var)
}

pub fn logistic_gd<'a>(fv: &'a Vec<Vec<f64>>, rvs: &'a BTreeMap::<&String, Vec<f64>>)
    -> (BTreeMap::<&'a String, (Array<f64, ndarray::Dim<[usize; 2]>>, Array<f64, ndarray::Dim<[usize; 2]>>, f64)>,
     Array<f64, ndarray::Dim<[usize; 1]>>) {

// Declare vector where we will collect results
    let mut fitted_models = BTreeMap::<&String, (Array<f64, ndarray::Dim<[usize; 2]>>, Array<f64, ndarray::Dim<[usize; 2]>>, f64)>::new();

// Define the features we want to model and put them into the X (i.e. features) matrix
    let m = fv.len();
    let theta_len = fv[0].len();
    let features_vec = fv.into_iter().flatten().map(|&x| x).collect::<Vec<f64>>();
    let mut x_arr = Array2::from_shape_vec((m, theta_len), features_vec).unwrap();
    let x_mu = x_arr.mean_axis(Axis(0)).unwrap();
    let x_sig = x_arr.std_axis(Axis(0), 0.0);
    let x_sig_recip = x_sig.map(|sd| if sd == &0.0 {1.0}
                                     else {1.0 / *sd});
    let mu_des = Array::from_shape_vec((theta_len, theta_len), vec![0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]).unwrap();
    x_arr.scaled_add(-0.0, &mu_des.dot(&x_mu));
    x_arr.zip_mut_with(&x_sig_recip, |x, s| *x *= s);

//  Control parameters for the optimisation. Switches allows further control over features modelled: redundant if regularising
    let alpha = 0.1;
    let n_iterations = 1000;

//  Extract the observations that correspond to these features for the response we want to model
    for (obs, y_vec) in rvs.iter() {
        let y_arr = Array::from_shape_vec((m, 1), y_vec.clone()).unwrap();
//  Initialise hypothesis matrix (if we have standardised the features then 0.0 should be OK to start
        let theta = Array::from_shape_vec((theta_len, 1), vec![0.0; theta_len]).unwrap();
//  Switches allows further control over features modelled: redundant if regularising
        let switches = match obs.as_str() {
                       "Home Win" => Array::from_shape_vec((theta_len, theta_len), vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]).unwrap(),
                       _ => Array::from_shape_vec((theta_len, theta_len), vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]).unwrap(),
                       };


//  Run the optimisation and return the model parameters and fitted values
//        fitted_models.insert(obs, grad_desc(&x_arr, &y_arr, theta, &switches, &alpha, n_iterations));


//        if obs == &&"Home Win".to_string() {
        if true {
            fitted_models.insert(obs, logistic_desc(&x_arr, &y_arr, theta, &switches, &alpha, n_iterations));
        }

//      in case we want to compare with theta from a non-standardised set of features
//        let mut theta_trans = fitted_models[obs].1.clone().column(0).to_owned();
//        theta_trans.zip_mut_with(&x_sig_recip, |x, s| *x *= s);
//        println!("{:?}", x_sig_recip);

    }
    (fitted_models, x_sig_recip)
}


// Gradient descent using logistic regression

pub fn logistic_desc(x: &Array2<f64>,
                y: &Array<f64, ndarray::Dim<[usize; 2]>>,
                mut th: Array<f64, ndarray::Dim<[usize; 2]>>,
                sw: &Array<f64, ndarray::Dim<[usize; 2]>>,
                al: &f64,
                n: i32) -> (Array<f64, ndarray::Dim<[usize; 2]>>, Array<f64, ndarray::Dim<[usize; 2]>>, f64) {

    let m = y.len();
    let y_mu = y.mean_axis(Axis(0)).unwrap();
    let y_err = y.map(|&yi| *(yi - &y_mu).get(0).unwrap()); //.iter().collect::<Array<f64, ndarray::Dim<[usize; 2]>>>();
    let y_var = y_err.t().dot(&y_err).sum();
    let mut mj: f64 = 0.0;

    for iter in 0..n {
        let h_theta = x.dot(&th).map(|x| 1.0 / (1.0 + (-x).exp()));

        let err = &h_theta - y;
        mj = (err.t().dot(&err)).sum();
        let j_grad = x.t().dot(&err) / m as f64;

        th.scaled_add(-al, &sw.dot(&j_grad));

    }
    (x.dot(&th), th, 1.0 - mj / y_var)
}
